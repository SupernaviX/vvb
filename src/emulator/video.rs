use crate::emulator::storage::Storage;
use anyhow::Result;
use std::sync::{mpsc, Arc, Mutex};

pub const VB_WIDTH: usize = 384;
pub const VB_HEIGHT: usize = 224;
pub const FRAME_SIZE: usize = VB_WIDTH * VB_HEIGHT;

const DPSTTS: usize = 0x0005f820;
const DPCTRL: usize = 0x0005f822;

// brightness control registers
const BRTA: usize = 0x0005f824;
const BRTB: usize = 0x0005f826;
const BRTC: usize = 0x0005f828;

const XPSTTS: usize = 0x0005f840;
const XPCTRL: usize = 0x0005f842;

enum FrameBuffer {
    Left0,
    Right0,
    Left1,
    Right1,
}

#[derive(Copy, Clone)]
pub enum Eye {
    Left,
    Right,
}
pub type EyeBuffer = Vec<u8>;

pub struct Frame {
    pub eye: Eye,
    pub buffer: Arc<Mutex<EyeBuffer>>,
}

pub type FrameChannel = mpsc::Receiver<Frame>;

pub struct Video {
    cycle: u64,
    frame_channel: Option<mpsc::Sender<Frame>>,
    buffers: [Arc<Mutex<EyeBuffer>>; 2],
}
impl Video {
    pub fn new() -> Video {
        Video {
            cycle: 0,
            frame_channel: None,
            buffers: [
                Arc::new(Mutex::new(vec![0; FRAME_SIZE])),
                Arc::new(Mutex::new(vec![0; FRAME_SIZE])),
            ],
        }
    }

    pub fn init(&mut self, storage: &mut Storage) {
        self.cycle = 0;
        self.set_flags(storage, DPCTRL, 0x00c0);
        storage.write_halfword(DPSTTS, 0x00c0);
    }

    pub fn run(&mut self, storage: &mut Storage, until_cycle: u64) -> Result<()> {
        let mut curr_ms = self.cycle / 20000;
        let next_ms = until_cycle / 20000;
        while curr_ms != next_ms {
            curr_ms = curr_ms + 1;
            let (current_eye, current_fb) = match curr_ms % 40 / 10 {
                0 => (Eye::Left, FrameBuffer::Left0),
                1 => (Eye::Right, FrameBuffer::Right0),
                2 => (Eye::Left, FrameBuffer::Left1),
                _ => (Eye::Right, FrameBuffer::Right1),
            };
            const FRAME_CLOCK_FLAG: i16 = 0x0080;
            let (display_flags, drawing_flags) = match current_fb {
                FrameBuffer::Left0 => (0x0004, 0x0008),
                FrameBuffer::Right0 => (0x0008, 0x0008),
                FrameBuffer::Left1 => (0x0010, 0x0004),
                FrameBuffer::Right1 => (0x0020, 0x0004),
            };
            match curr_ms % 20 {
                0 => self.set_flags(storage, DPCTRL, FRAME_CLOCK_FLAG),
                3 | 13 => {
                    self.set_flags(storage, DPCTRL, display_flags);
                    self.set_flags(storage, XPCTRL, drawing_flags);
                }
                5 | 15 => {
                    self.build_frame(storage, current_fb)?;
                    self.send_frame(current_eye)?;
                }
                8 | 18 => {
                    self.clear_flags(storage, DPCTRL, display_flags);
                    self.clear_flags(storage, XPCTRL, drawing_flags);
                }
                10 => self.clear_flags(storage, DPCTRL, FRAME_CLOCK_FLAG),
                _ => (),
            };
        }
        self.cycle = until_cycle;
        storage.write_halfword(DPSTTS, storage.read_halfword(DPCTRL));
        storage.write_halfword(XPSTTS, storage.read_halfword(XPCTRL));
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

    fn build_frame(&self, storage: &Storage, buffer: FrameBuffer) -> Result<()> {
        // colors to render
        let color0 = 0u8; // always black
        let color1 = storage.read_halfword(BRTA) as u8 * 2;
        let color2 = storage.read_halfword(BRTB) as u8 * 2;
        let color3 = color1 + color2 + storage.read_halfword(BRTC) as u8 * 2;
        let colors = [color0, color1, color2, color3];

        let (buf_index, buf_address) = match buffer {
            FrameBuffer::Left0 => (0, 0x00000000),
            FrameBuffer::Right0 => (1, 0x00010000),
            FrameBuffer::Left1 => (0, 0x00008000),
            FrameBuffer::Right1 => (1, 0x00018000),
        };
        let eye_buffer = &mut self.buffers[buf_index]
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

    fn set_flags(&self, storage: &mut Storage, address: usize, flags: i16) {
        let flags = storage.read_halfword(address) | flags;
        storage.write_halfword(address, flags);
    }

    fn clear_flags(&self, storage: &mut Storage, address: usize, flags: i16) {
        let flags = storage.read_halfword(address) ^ flags;
        storage.write_halfword(address, flags);
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::storage::Storage;
    use crate::emulator::video::{Video, DPSTTS, XPSTTS};

    fn ms_to_cycles(ms: u64) -> u64 {
        ms * 20000
    }

    #[test]
    fn can_emulate_two_frames() {
        let mut video = Video::new();
        let mut storage = Storage::new();

        video.init(&mut storage);
        assert_eq!(storage.read_halfword(DPSTTS), 0x00c0);

        video.run(&mut storage, ms_to_cycles(3)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x00c4);
        assert_eq!(storage.read_halfword(XPSTTS), 0x0008);

        video.run(&mut storage, ms_to_cycles(8)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x00c0);
        assert_eq!(storage.read_halfword(XPSTTS), 0x0000);

        video.run(&mut storage, ms_to_cycles(10)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x0040);

        video.run(&mut storage, ms_to_cycles(13)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x0048);
        assert_eq!(storage.read_halfword(XPSTTS), 0x0008);

        video.run(&mut storage, ms_to_cycles(18)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x0040);
        assert_eq!(storage.read_halfword(XPSTTS), 0x0000);

        video.run(&mut storage, ms_to_cycles(20)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x00c0);

        video.run(&mut storage, ms_to_cycles(23)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x00d0);
        assert_eq!(storage.read_halfword(XPSTTS), 0x0004);

        video.run(&mut storage, ms_to_cycles(28)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x00c0);
        assert_eq!(storage.read_halfword(XPSTTS), 0x0000);

        video.run(&mut storage, ms_to_cycles(30)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x0040);

        video.run(&mut storage, ms_to_cycles(33)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x0060);
        assert_eq!(storage.read_halfword(XPSTTS), 0x0004);

        video.run(&mut storage, ms_to_cycles(38)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x0040);
        assert_eq!(storage.read_halfword(XPSTTS), 0x0000);

        video.run(&mut storage, ms_to_cycles(40)).unwrap();
        assert_eq!(storage.read_halfword(DPSTTS), 0x00c0);
    }
}
