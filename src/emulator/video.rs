use crate::emulator::memory::Memory;
use anyhow::Result;
use log::error;
use std::sync::{mpsc, Arc, Mutex};

mod drawing;

pub const VB_WIDTH: usize = 384;
pub const VB_HEIGHT: usize = 224;
pub const FRAME_SIZE: usize = VB_WIDTH * VB_HEIGHT;

const INTPND: usize = 0x0005f800;
const INTENB: usize = 0x0005f802;
const INTCLR: usize = 0x0005f804;

// flags for the interrupt registers
const FRAMESTART: i16 = 0x0010;
const DP_INTERRUPTS: i16 = FRAMESTART;

const DPSTTS: usize = 0x0005f820;
const DPCTRL: usize = 0x0005f822;

// flags for DPSTTS/DPCTRL
const FCLK: i16 = 0x0080;
const SCANRDY: i16 = 0x0040;
const R1BSY: i16 = 0x0020;
const L1BSY: i16 = 0x0010;
const R0BSY: i16 = 0x0008;
const L0BSY: i16 = 0x0004;
const DISP: i16 = 0x0002;
const DPRST: i16 = 0x0001;
const DP_READONLY_MASK: i16 = FCLK | SCANRDY | R1BSY | L1BSY | R0BSY | L0BSY;

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
use crate::emulator::cpu::Interrupt;
use crate::emulator::video::drawing::DrawingProcess;
use Eye::{Left, Right};

pub type EyeBuffer = Vec<u8>;

pub struct Frame {
    pub eye: Eye,
    pub buffer: Arc<Mutex<EyeBuffer>>,
}

pub type FrameChannel = mpsc::Receiver<Frame>;

pub struct Video {
    cycle: u64,
    displaying: bool,
    drawing: bool,
    xp_module: DrawingProcess,
    dpctrl_flags: i16,
    pending_interrupts: i16,
    enabled_interrupts: i16,
    display_buffer: Buffer,
    frame_channel: Option<mpsc::Sender<Frame>>,
    buffers: [Arc<Mutex<EyeBuffer>>; 2],
}
impl Video {
    pub fn new() -> Video {
        Video {
            cycle: 0,
            displaying: false,
            drawing: false,
            xp_module: DrawingProcess::new(),
            dpctrl_flags: SCANRDY,
            pending_interrupts: 0,
            enabled_interrupts: 0,
            display_buffer: Buffer0,
            frame_channel: None,
            buffers: [
                Arc::new(Mutex::new(vec![0; FRAME_SIZE])),
                Arc::new(Mutex::new(vec![0; FRAME_SIZE])),
            ],
        }
    }

    pub fn init(&mut self, memory: &mut Memory) {
        self.cycle = 0;
        self.displaying = false;
        self.drawing = false;
        self.dpctrl_flags = SCANRDY;
        self.pending_interrupts = 0;
        self.enabled_interrupts = 0;
        self.display_buffer = Buffer0;
        memory.write_halfword(DPCTRL, self.dpctrl_flags);
        memory.write_halfword(DPSTTS, self.dpctrl_flags);
        memory.write_halfword(INTPND, self.pending_interrupts);
        memory.write_halfword(INTENB, self.enabled_interrupts);
    }

    pub fn next_event(&self) -> u64 {
        ((self.cycle / 20000) + 1) * 20000
    }

    pub fn active_interrupt(&self) -> Option<Interrupt> {
        if (self.enabled_interrupts & self.pending_interrupts) != 0 {
            return Some(Interrupt {
                code: 0xfe40,
                level: 4,
                handler: 0xfffffe40,
            });
        }
        None
    }

    pub fn process_event(&mut self, memory: &mut Memory, address: usize) {
        if address == DPCTRL {
            let mut dpctrl = memory.read_halfword(DPCTRL);

            let displaying = (dpctrl & DISP) != 0;
            let resetting = (dpctrl & DPRST) != 0;
            if !displaying || resetting {
                self.displaying = false;
                self.dpctrl_flags = SCANRDY;
            }
            if resetting {
                self.pending_interrupts &= !DP_INTERRUPTS;
                self.enabled_interrupts &= !DP_INTERRUPTS;
            }

            // Don't let the program overwrite the readonly flags
            dpctrl &= !DP_READONLY_MASK;
            dpctrl |= self.dpctrl_flags;
            memory.write_halfword(DPCTRL, dpctrl);
            memory.write_halfword(DPSTTS, dpctrl & !DPRST);
            memory.write_halfword(INTPND, self.pending_interrupts);
            memory.write_halfword(INTENB, self.enabled_interrupts);
        }

        if address == INTENB {
            self.enabled_interrupts = memory.read_halfword(INTENB);
            if (self.enabled_interrupts & !FRAMESTART) != 0 {
                error!("Unsupported interrupt! 0x{:04x}", self.enabled_interrupts);
                panic!();
            }
        }

        if address == INTCLR {
            self.pending_interrupts &= !memory.read_halfword(INTCLR);
            memory.write_halfword(INTPND, self.pending_interrupts);
        }
    }

    pub fn run(&mut self, memory: &mut Memory, target_cycle: u64) -> Result<()> {
        let mut dpctrl = memory.read_halfword(DPCTRL);
        let mut xpctrl = memory.read_halfword(XPCTRL);

        let mut curr_ms = self.cycle / 20000;
        let next_ms = target_cycle / 20000;
        while curr_ms < next_ms {
            curr_ms = curr_ms + 1;
            self.cycle += curr_ms * 20000;

            match curr_ms % 20 {
                0 => {
                    // If we're starting a display frame, check what's enabled
                    // TODO: only start drawing at the start of a game frame
                    self.displaying = (dpctrl & DISP) != 0 && (dpctrl & DPRST) == 0;
                    self.drawing = (xpctrl & XPEN) != 0;

                    // Frame clock up
                    if self.displaying {
                        self.dpctrl_flags |= FCLK;
                        self.pending_interrupts |= FRAMESTART;
                        memory.write_halfword(INTPND, self.pending_interrupts);
                    }

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
                    if self.displaying {
                        self.dpctrl_flags |= match self.display_buffer {
                            Buffer0 => L0BSY,
                            Buffer1 => L1BSY,
                        };
                    }

                    if self.drawing {
                        // Actually draw on the background buffer
                        self.draw(memory)?;
                    }
                }
                5 => {
                    if self.displaying {
                        // Actually display the left eye
                        self.build_frame(memory, Left);
                        self.send_frame(Left)?;
                    }

                    // "Stop drawing" on background buffer
                    xpctrl &= !(F0BSY | F1BSY);
                }
                8 => {
                    // "Stop displaying" left eye
                    self.dpctrl_flags &= !(L0BSY | L1BSY);
                }
                10 => {
                    if self.displaying {
                        // Frame clock down
                        self.dpctrl_flags &= !FCLK;
                    }
                }
                13 => {
                    if self.displaying {
                        // "Start displaying" right eye
                        self.dpctrl_flags |= match self.display_buffer {
                            Buffer0 => R0BSY,
                            Buffer1 => R1BSY,
                        };
                    }
                }
                15 => {
                    if self.displaying {
                        // Actually display the right eye
                        self.build_frame(memory, Right);
                        self.send_frame(Right)?;
                    }
                }
                18 => {
                    // "Stop displaying" right eye,
                    self.dpctrl_flags &= !(R0BSY | R1BSY);
                }
                _ => (),
            };
        }
        self.cycle = target_cycle;
        dpctrl &= !DP_READONLY_MASK;
        dpctrl |= self.dpctrl_flags;
        memory.write_halfword(DPCTRL, dpctrl);
        memory.write_halfword(DPSTTS, dpctrl & !DPRST);
        memory.write_halfword(XPCTRL, xpctrl);
        memory.write_halfword(XPSTTS, xpctrl);
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

    fn build_frame(&self, memory: &Memory, eye: Eye) {
        // colors to render
        let color0 = 0; // always black
        let color1 = 255.min(self.get_brightness(memory, BRTA));
        let color2 = 255.min(self.get_brightness(memory, BRTB));
        let color3 = 255.min(color1 + color2 + self.get_brightness(memory, BRTC));
        let colors = [color0 as u8, color1 as u8, color2 as u8, color3 as u8];

        let buf_address = self.get_buffer_address(eye, self.display_buffer);
        let eye_buffer = &mut self.buffers[eye as usize]
            .lock()
            .expect("Buffer lock was poisoned!");

        for (col, col_offset) in (0..(384 * 64)).step_by(64).enumerate() {
            for (row_offset, top_row) in (0..224).step_by(4).enumerate().step_by(2) {
                let address = buf_address + col_offset + row_offset;
                let pixels = memory.read_halfword(address) as u16;
                for (row, pixel) in (0..16).step_by(2).map(|i| (pixels >> i) & 0b11).enumerate() {
                    let index = col + (top_row + row) * 384;
                    eye_buffer[index] = colors[pixel as usize];
                }
            }
        }
    }

    fn get_brightness(&self, memory: &Memory, address: usize) -> i16 {
        // experimentally chosen conversion factor from led-duration-in-50-ns-increments to 8-bit color
        memory.read_halfword(address) * 19 / 8
    }

    fn get_buffer_address(&self, eye: Eye, buffer: Buffer) -> usize {
        match (eye, buffer) {
            (Left, Buffer0) => 0x00000000,
            (Right, Buffer0) => 0x00010000,
            (Left, Buffer1) => 0x00008000,
            (Right, Buffer1) => 0x00018000,
        }
    }

    // Perform the drawing procedure, writing to whichever framebuffer is inactive
    fn draw(&mut self, memory: &mut Memory) -> Result<()> {
        let buffer = self.display_buffer.toggle();

        let left_buf_address = self.get_buffer_address(Left, buffer);
        self.xp_module.draw_eye(memory, Left, left_buf_address)?;

        let right_buf_address = self.get_buffer_address(Right, buffer);
        self.xp_module.draw_eye(memory, Right, right_buf_address)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::memory::Memory;
    use crate::emulator::video::{Video, DISP, DPCTRL, DPRST, FRAMESTART, INTCLR, INTENB, INTPND};
    use crate::emulator::video::{DPSTTS, FCLK, L0BSY, L1BSY, R0BSY, R1BSY, SCANRDY};
    use crate::emulator::video::{F0BSY, F1BSY, XPCTRL, XPEN, XPSTTS};

    fn ms_to_cycles(ms: u64) -> u64 {
        ms * 20000
    }

    fn write_dpctrl(video: &mut Video, memory: &mut Memory, value: i16) {
        memory.write_halfword(DPCTRL, value);
        video.process_event(memory, DPCTRL);
    }
    fn write_intenb(video: &mut Video, memory: &mut Memory, value: i16) {
        memory.write_halfword(INTENB, value);
        video.process_event(memory, INTENB);
    }
    fn write_intclr(video: &mut Video, memory: &mut Memory, value: i16) {
        memory.write_halfword(INTCLR, value);
        video.process_event(memory, INTCLR);
    }

    #[test]
    fn can_emulate_two_frames_of_dpstts_while_drawing_is_off() {
        let mut video = Video::new();
        let mut memory = Memory::new();

        video.init(&mut memory);
        write_dpctrl(&mut video, &mut memory, DISP);

        // start 2 frames in, because that's the first time we see a rising FCLK
        video.run(&mut memory, ms_to_cycles(40)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(&mut memory, ms_to_cycles(43)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK | L0BSY);

        video.run(&mut memory, ms_to_cycles(48)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(&mut memory, ms_to_cycles(50)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(&mut memory, ms_to_cycles(53)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | R0BSY);

        video.run(&mut memory, ms_to_cycles(58)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(&mut memory, ms_to_cycles(60)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(&mut memory, ms_to_cycles(63)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK | L0BSY);

        video.run(&mut memory, ms_to_cycles(68)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(&mut memory, ms_to_cycles(70)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(&mut memory, ms_to_cycles(73)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | R0BSY);

        video.run(&mut memory, ms_to_cycles(78)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(&mut memory, ms_to_cycles(80)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);
    }

    #[test]
    fn can_emulate_two_frames_of_dpstts_while_drawing_is_on() {
        let mut video = Video::new();
        let mut memory = Memory::new();

        video.init(&mut memory);
        write_dpctrl(&mut video, &mut memory, DISP);
        memory.write_halfword(XPCTRL, XPEN);

        // start 2 frames in, because that's the first time we see a rising FCLK
        video.run(&mut memory, ms_to_cycles(40)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(&mut memory, ms_to_cycles(43)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK | L0BSY);

        video.run(&mut memory, ms_to_cycles(48)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(&mut memory, ms_to_cycles(50)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(&mut memory, ms_to_cycles(53)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | R0BSY);

        video.run(&mut memory, ms_to_cycles(58)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(&mut memory, ms_to_cycles(60)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(&mut memory, ms_to_cycles(63)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK | L1BSY);

        video.run(&mut memory, ms_to_cycles(68)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(&mut memory, ms_to_cycles(70)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(&mut memory, ms_to_cycles(73)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | R1BSY);

        video.run(&mut memory, ms_to_cycles(78)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(&mut memory, ms_to_cycles(80)).unwrap();
        assert_eq!(memory.read_halfword(DPSTTS), DISP | SCANRDY | FCLK);
    }

    #[test]
    fn can_emulate_two_frames_of_xpstts_while_drawing_is_on() {
        let mut video = Video::new();
        let mut memory = Memory::new();

        video.init(&mut memory);
        write_dpctrl(&mut video, &mut memory, DISP);
        // turn on drawing
        memory.write_halfword(XPCTRL, XPEN);

        // start 2 frames in, because that's the first time we see a rising FCLK
        video.run(&mut memory, ms_to_cycles(40)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN | F1BSY);

        video.run(&mut memory, ms_to_cycles(45)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN);

        video.run(&mut memory, ms_to_cycles(50)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN);

        video.run(&mut memory, ms_to_cycles(55)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN);

        video.run(&mut memory, ms_to_cycles(60)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN | F0BSY);

        video.run(&mut memory, ms_to_cycles(65)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN);

        video.run(&mut memory, ms_to_cycles(70)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN);

        video.run(&mut memory, ms_to_cycles(75)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN);

        video.run(&mut memory, ms_to_cycles(80)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN | F1BSY);
    }

    #[test]
    fn can_turn_off_xpstts_midframe() {
        let mut video = Video::new();
        let mut memory = Memory::new();

        video.init(&mut memory);
        write_dpctrl(&mut video, &mut memory, DISP);

        // turn on drawing 2 frames in, because that's the first time we see a rising FCLK
        video.run(&mut memory, ms_to_cycles(39)).unwrap();
        memory.write_halfword(XPCTRL, XPEN);
        video.run(&mut memory, ms_to_cycles(40)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN | F0BSY);

        // turn off drawing
        memory.write_halfword(XPCTRL, F0BSY);
        video.run(&mut memory, ms_to_cycles(42)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), F0BSY);

        video.run(&mut memory, ms_to_cycles(45)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), 0);

        video.run(&mut memory, ms_to_cycles(60)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), 0);
    }

    #[test]
    fn can_turn_on_xpstts_midframe() {
        let mut video = Video::new();
        let mut memory = Memory::new();

        video.init(&mut memory);
        write_dpctrl(&mut video, &mut memory, DISP);

        // start >2 frames in, because that's the first time we see a rising FCLK
        video.run(&mut memory, ms_to_cycles(41)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), 0);

        // turn on drawing
        memory.write_halfword(XPCTRL, XPEN);
        video.run(&mut memory, ms_to_cycles(42)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN);

        video.run(&mut memory, ms_to_cycles(45)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN);

        video.run(&mut memory, ms_to_cycles(60)).unwrap();
        assert_eq!(memory.read_halfword(XPSTTS), XPEN | F0BSY);
    }

    #[test]
    fn can_trigger_framestart_interrupt() {
        let mut video = Video::new();
        let mut memory = Memory::new();

        video.init(&mut memory);
        write_dpctrl(&mut video, &mut memory, DISP);

        // While INTENB is unset, set INTPND but don't trigger interrupts
        video.run(&mut memory, ms_to_cycles(37)).unwrap();
        assert_eq!(memory.read_halfword(INTPND), FRAMESTART);
        assert!(video.active_interrupt().is_none());

        // Interrupt can be cleared by writing to DPRST
        write_dpctrl(&mut video, &mut memory, DISP | DPRST);
        video.run(&mut memory, ms_to_cycles(38)).unwrap();
        assert_eq!(memory.read_halfword(INTPND), 0);

        // Interrupt is triggered on FCLK going high
        write_dpctrl(&mut video, &mut memory, DISP);
        write_intenb(&mut video, &mut memory, FRAMESTART);
        video.run(&mut memory, ms_to_cycles(40)).unwrap();
        assert_eq!(memory.read_halfword(INTPND), FRAMESTART);
        assert!(video.active_interrupt().is_some());

        // Interrupt can be cleared by writing to INTCLR
        write_intclr(&mut video, &mut memory, FRAMESTART);
        video.run(&mut memory, ms_to_cycles(41)).unwrap();
        assert_eq!(memory.read_halfword(INTPND), 0);
        assert!(video.active_interrupt().is_none());
    }
}
