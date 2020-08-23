use crate::emulator::storage::Storage;
use anyhow::Result;
use std::sync::{mpsc, Arc, Mutex};

pub const VB_WIDTH: usize = 384;
pub const VB_HEIGHT: usize = 224;
pub const FRAME_SIZE: usize = VB_WIDTH * VB_HEIGHT;

const DPSTTS: usize = 0x0005f820;
const DPCTRL: usize = 0x0005f822;

// flags for DPSTTS/DPCTRL
const FCLK: i16 = 0x0080;
const SCANRDY: i16 = 0x0040;
const R1BSY: i16 = 0x0020;
const L1BSY: i16 = 0x0010;
const R0BSY: i16 = 0x0008;
const L0BSY: i16 = 0x0004;

// brightness control registers
const BRTA: usize = 0x0005f824;
const BRTB: usize = 0x0005f826;
const BRTC: usize = 0x0005f828;

const XPSTTS: usize = 0x0005f840;
const XPCTRL: usize = 0x0005f842;

// flags for XPSTTS/XPCTRL
const F1BSY: i16 = 0x0008;
const F0BSY: i16 = 0x0004;
const XPEN: i16 = 0x0002;

const BKCOL: usize = 0x0005f870;

#[derive(Copy, Clone, Debug)]
enum Buffer {
    Buffer0,
    Buffer1,
}
impl Buffer {
    pub fn toggle(&self) -> Self {
        match self {
            Buffer0 => Buffer1,
            Buffer1 => Buffer0,
        }
    }
}
use Buffer::{Buffer0, Buffer1};

#[derive(Copy, Clone, Debug)]
pub enum Eye {
    Left,
    Right,
}
use Eye::{Left, Right};

pub type EyeBuffer = Vec<u8>;

pub struct Frame {
    pub eye: Eye,
    pub buffer: Arc<Mutex<EyeBuffer>>,
}

pub type FrameChannel = mpsc::Receiver<Frame>;

pub struct Video {
    cycle: u64,
    drawing: bool,
    display_buffer: Buffer,
    frame_channel: Option<mpsc::Sender<Frame>>,
    buffers: [Arc<Mutex<EyeBuffer>>; 2],
}
impl Video {
    pub fn new() -> Video {
        Video {
            cycle: 0,
            drawing: false,
            display_buffer: Buffer0,
            frame_channel: None,
            buffers: [
                Arc::new(Mutex::new(vec![0; FRAME_SIZE])),
                Arc::new(Mutex::new(vec![0; FRAME_SIZE])),
            ],
        }
    }

    pub fn init(&mut self, storage: &mut Storage) {
        self.cycle = 0;
        storage.write_halfword(DPCTRL, FCLK | SCANRDY);
        storage.write_halfword(DPSTTS, FCLK | SCANRDY);
    }

    pub fn run(&mut self, storage: &mut Storage, until_cycle: u64) -> Result<()> {
        let mut dpctrl = storage.read_halfword(DPCTRL);
        let mut xpctrl = storage.read_halfword(XPCTRL);

        let mut curr_ms = self.cycle / 20000;
        let next_ms = until_cycle / 20000;
        while curr_ms != next_ms {
            curr_ms = curr_ms + 1;
            self.cycle += curr_ms * 20000;

            match curr_ms % 20 {
                0 => {
                    // Frame clock up
                    dpctrl |= FCLK;

                    // If we're starting a display frame, toggle whether we're drawing
                    // TODO: this should be the start of a game frame
                    self.drawing = (xpctrl & XPEN) != 0;

                    if self.drawing {
                        // "Start drawing" on whichever buffer was displayed before
                        xpctrl |= match self.display_buffer {
                            Buffer0 => F0BSY,
                            Buffer1 => F1BSY,
                        };

                        // Switch to displaying the other buffer
                        self.display_buffer = self.display_buffer.toggle();
                    }
                }
                3 => {
                    // "Start displaying" left eye
                    dpctrl |= match self.display_buffer {
                        Buffer0 => L0BSY,
                        Buffer1 => L1BSY,
                    };

                    if self.drawing {
                        // Actually draw on the background buffer
                        self.draw(storage);
                    }
                }
                5 => {
                    // Actually display the left eye
                    self.build_frame(storage, Left)?;
                    self.send_frame(Left)?;

                    // "Stop drawing" on background buffer
                    xpctrl &= !(F0BSY | F1BSY);
                }
                8 => {
                    // "Stop displaying" left eye
                    dpctrl &= !(L0BSY | L1BSY);
                }
                10 => {
                    // Frame clock down
                    dpctrl ^= FCLK;
                }
                13 => {
                    // "Start displaying" right eye
                    dpctrl |= match self.display_buffer {
                        Buffer0 => R0BSY,
                        Buffer1 => R1BSY,
                    };
                }
                15 => {
                    // Actually display the right eye
                    self.build_frame(storage, Right)?;
                    self.send_frame(Right)?;
                }
                18 => {
                    // "Stop displaying" right eye,
                    dpctrl &= !(R0BSY | R1BSY);
                }
                _ => (),
            };
        }
        self.cycle = until_cycle;
        storage.write_halfword(DPCTRL, dpctrl);
        storage.write_halfword(DPSTTS, dpctrl);
        storage.write_halfword(XPCTRL, xpctrl);
        storage.write_halfword(XPSTTS, xpctrl);
        Ok(())
    }

    pub fn get_frame_channel(&mut self) -> FrameChannel {
        let (tx, rx) = mpsc::channel();
        self.frame_channel = Some(tx);
        rx
    }

    pub fn load_frame(&self, eye: Eye, image: &[u8]) {
        let mut buffer = self.buffers[eye as usize]
            .lock()
            .expect("Buffer lock was poisoned!");
        // Input data is RGBA, only copy the R
        for (place, data) in buffer.iter_mut().zip(image.iter().step_by(4)) {
            *place = *data;
        }
    }

    pub fn send_frame(&self, eye: Eye) -> Result<()> {
        if let Some(channel) = self.frame_channel.as_ref() {
            let buffer = &self.buffers[eye as usize];
            let frame = Frame {
                eye,
                buffer: Arc::clone(buffer),
            };
            channel.send(frame)?;
        }
        Ok(())
    }

    fn build_frame(&self, storage: &Storage, eye: Eye) -> Result<()> {
        // colors to render
        let color0 = 0u8; // always black
        let color1 = storage.read_halfword(BRTA) as u8 * 2;
        let color2 = storage.read_halfword(BRTB) as u8 * 2;
        let color3 = color1 + color2 + storage.read_halfword(BRTC) as u8 * 2;
        let colors = [color0, color1, color2, color3];

        let buf_address = self.get_buffer_address(eye, self.display_buffer);
        let eye_buffer = &mut self.buffers[eye as usize]
            .lock()
            .expect("Buffer lock was poisoned!");

        for (col, col_offset) in (0..(384 * 64)).step_by(64).enumerate() {
            for (row_offset, top_row) in (0..224).step_by(4).enumerate().step_by(2) {
                let address = buf_address + col_offset + row_offset;
                let pixels = storage.read_halfword(address) as u16;
                for (row, pixel) in (0..16).step_by(2).map(|i| (pixels >> i) & 0b11).enumerate() {
                    let index = col + (top_row + row) * 384;
                    eye_buffer[index] = colors[pixel as usize];
                }
            }
        }
        Ok(())
    }

    // Perform the drawing procedure, writing to whichever framebuffer is inactive
    fn draw(&self, storage: &mut Storage) {
        let buffer = self.display_buffer.toggle();
        let left_buf_address = self.get_buffer_address(Left, buffer);
        let right_buf_address = self.get_buffer_address(Right, buffer);

        // Clear both frames to BKCOL
        let bkcol = storage.read_halfword(BKCOL) & 0x03;
        let fill = (0..16)
            .step_by(2)
            .map(|shift| bkcol << shift)
            .fold(0, |a, b| a | b);
        for buf_address in [left_buf_address, right_buf_address].iter() {
            for col_offset in (0..384 * 64).step_by(64) {
                for row_offset in 0..56 {
                    let address = buf_address + col_offset + row_offset;
                    storage.write_halfword(address, fill);
                }
            }
        }

        // TODO: actually... draw things
    }

    fn get_buffer_address(&self, eye: Eye, buffer: Buffer) -> usize {
        match (eye, buffer) {
            (Left, Buffer0) => 0x00000000,
            (Right, Buffer0) => 0x00010000,
            (Left, Buffer1) => 0x00008000,
            (Right, Buffer1) => 0x00018000,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::storage::Storage;
    use crate::emulator::video::Video;
    use crate::emulator::video::{DPSTTS, FCLK, L0BSY, L1BSY, R0BSY, R1BSY, SCANRDY};
    use crate::emulator::video::{F0BSY, F1BSY, XPCTRL, XPEN, XPSTTS};

    fn ms_to_cycles(ms: u64) -> u64 {
        ms * 20000
    }

    #[test]
    fn can_emulate_two_frames_of_dpstts_while_drawing_is_off() {
        let mut video = Video::new();
        let mut storage = Storage::new();

        video.init(&mut storage);
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);

        video.run(&mut storage, ms_to_cycles(3)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK | L0BSY);

        video.run(&mut storage, ms_to_cycles(8)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);

        video.run(&mut storage, ms_to_cycles(10)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY);

        video.run(&mut storage, ms_to_cycles(13)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | R0BSY);

        video.run(&mut storage, ms_to_cycles(18)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY);

        video.run(&mut storage, ms_to_cycles(20)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);

        video.run(&mut storage, ms_to_cycles(23)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK | L0BSY);

        video.run(&mut storage, ms_to_cycles(28)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);

        video.run(&mut storage, ms_to_cycles(30)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY);

        video.run(&mut storage, ms_to_cycles(33)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | R0BSY);

        video.run(&mut storage, ms_to_cycles(38)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY);

        video.run(&mut storage, ms_to_cycles(40)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);
    }

    #[test]
    fn can_emulate_two_frames_of_dpstts_while_drawing_is_on() {
        let mut video = Video::new();
        let mut storage = Storage::new();

        video.init(&mut storage);
        storage.write_halfword(XPCTRL, XPEN);
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);

        video.run(&mut storage, ms_to_cycles(3)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK | L0BSY);

        video.run(&mut storage, ms_to_cycles(8)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);

        video.run(&mut storage, ms_to_cycles(10)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY);

        video.run(&mut storage, ms_to_cycles(13)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | R0BSY);

        video.run(&mut storage, ms_to_cycles(18)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY);

        video.run(&mut storage, ms_to_cycles(20)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);

        video.run(&mut storage, ms_to_cycles(23)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK | L1BSY);

        video.run(&mut storage, ms_to_cycles(28)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);

        video.run(&mut storage, ms_to_cycles(30)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY);

        video.run(&mut storage, ms_to_cycles(33)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | R1BSY);

        video.run(&mut storage, ms_to_cycles(38)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY);

        video.run(&mut storage, ms_to_cycles(40)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), SCANRDY | FCLK);
    }

    #[test]
    fn can_emulate_two_frames_of_xpstts_while_drawing_is_on() {
        let mut video = Video::new();
        let mut storage = Storage::new();

        video.init(&mut storage);
        // turn on drawing
        storage.write_halfword(XPCTRL, XPEN);

        // start 2 frames in, because that's the first time we see a rising FCLK
        video.run(&mut storage, ms_to_cycles(40)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN | F1BSY);

        video.run(&mut storage, ms_to_cycles(45)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN);

        video.run(&mut storage, ms_to_cycles(50)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN);

        video.run(&mut storage, ms_to_cycles(55)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN);

        video.run(&mut storage, ms_to_cycles(60)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN | F0BSY);

        video.run(&mut storage, ms_to_cycles(65)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN);

        video.run(&mut storage, ms_to_cycles(70)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN);

        video.run(&mut storage, ms_to_cycles(75)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN);

        video.run(&mut storage, ms_to_cycles(80)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN | F1BSY);
    }

    #[test]
    fn can_turn_off_xpstts_midframe() {
        let mut video = Video::new();
        let mut storage = Storage::new();

        video.init(&mut storage);

        // turn on drawing 2 frames in, because that's the first time we see a rising FCLK
        video.run(&mut storage, ms_to_cycles(39)).unwrap();
        storage.write_halfword(XPCTRL, XPEN);
        video.run(&mut storage, ms_to_cycles(40)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN | F0BSY);

        // turn off drawing
        storage.write_halfword(XPCTRL, F0BSY);
        video.run(&mut storage, ms_to_cycles(42)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), F0BSY);

        video.run(&mut storage, ms_to_cycles(45)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), 0);

        video.run(&mut storage, ms_to_cycles(60)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), 0);
    }

    #[test]
    fn can_turn_on_xpstts_midframe() {
        let mut video = Video::new();
        let mut storage = Storage::new();

        video.init(&mut storage);

        // start >2 frames in, because that's the first time we see a rising FCLK
        video.run(&mut storage, ms_to_cycles(41)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), 0);

        // turn on drawing
        storage.write_halfword(XPCTRL, XPEN);
        video.run(&mut storage, ms_to_cycles(42)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN);

        video.run(&mut storage, ms_to_cycles(45)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN);

        video.run(&mut storage, ms_to_cycles(60)).unwrap();
        assert_eq!(storage.read_halfword(XPSTTS), XPEN | F0BSY);
    }
}
