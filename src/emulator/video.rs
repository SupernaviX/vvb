use crate::emulator::cpu::Exception;
use crate::emulator::memory::Memory;
use crate::emulator::video::drawing::DrawingProcess;
use anyhow::Result;
use log::error;
use serde_derive::{Deserialize, Serialize};
use std::cell::{Ref, RefCell};
use std::convert::TryFrom;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

mod buffer;
pub mod drawing;

pub const VB_WIDTH: usize = 384;
pub const VB_HEIGHT: usize = 224;
pub const FRAME_SIZE: usize = VB_WIDTH * VB_HEIGHT;

const INTPND: usize = 0x0005f800;
const INTENB: usize = 0x0005f802;
const INTCLR: usize = 0x0005f804;

// flags for the interrupt registers
const TIMEERR: u16 = 0x8000;
const XPEND: u16 = 0x4000;
const SBHIT: u16 = 0x2000;
const XP_INTERRUPTS: u16 = TIMEERR | XPEND | SBHIT;
const FRAMESTART: u16 = 0x0010;
const GAMESTART: u16 = 0x0008;
const RFBEND: u16 = 0x0004;
const LFBEND: u16 = 0x0002;
const SCANERR: u16 = 0x0001;
const DP_INTERRUPTS: u16 = FRAMESTART | GAMESTART | RFBEND | LFBEND | SCANERR;

const DPSTTS: usize = 0x0005f820;
const DPCTRL: usize = 0x0005f822;
const VER: usize = 0x0005f844;

// flags for DPSTTS/DPCTRL
const LOCK: u16 = 0x0400;
const FCLK: u16 = 0x0080;
const SCANRDY: u16 = 0x0040;
const R1BSY: u16 = 0x0020;
const L1BSY: u16 = 0x0010;
const R0BSY: u16 = 0x0008;
const L0BSY: u16 = 0x0004;
const DISP: u16 = 0x0002;
const DPRST: u16 = 0x0001;
const DPBSY: u16 = R1BSY | L1BSY | R0BSY | L0BSY;
const DP_READONLY_MASK: u16 = FCLK | SCANRDY | R1BSY | L1BSY | R0BSY | L0BSY;

// brightness control registers
const BRTA: usize = 0x0005f824;
const BRTB: usize = 0x0005f826;
const BRTC: usize = 0x0005f828;

const FRMCYC: usize = 0x0005f82e;

const CTA: usize = 0x0005f830;

const XPSTTS: usize = 0x0005f840;
const XPCTRL: usize = 0x0005f842;

// flags for XPSTTS/XPCTRL
const SBOUT: u16 = 0x8000;
const SBCOUNT_MASK: u16 = 0x1f00;
const F1BSY: u16 = 0x0008;
const F0BSY: u16 = 0x0004;
const XPEN: u16 = 0x0002;
const XPRST: u16 = 0x0001;
const XP_READONLY_MASK: u16 = !(XPEN | XPRST);

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Eye {
    Left,
    Right,
}
use crate::emulator::video::buffer::{SharedBuffer, SharedBufferConsumer};
use Eye::{Left, Right};

impl TryFrom<i32> for Eye {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Left),
            1 => Ok(Right),
            _ => Err(anyhow::anyhow!("Invalid eye {}", value)),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct VideoState {
    cycle: u64,
    displaying: bool,
    drawing: bool,
    game_frame_counter: u8,
    dpctrl_flags: u16,
    xpctrl_flags: u16,
    pending_interrupts: u16,
    enabled_interrupts: u16,
    display_buffer: Buffer,
}

impl Default for VideoState {
    fn default() -> Self {
        Self {
            cycle: 0,
            displaying: false,
            drawing: false,
            game_frame_counter: 0,
            dpctrl_flags: SCANRDY,
            xpctrl_flags: 0,
            pending_interrupts: 0,
            enabled_interrupts: 0,
            display_buffer: Buffer0,
        }
    }
}

#[derive(Default)]
struct FrameBuffers {
    left: SharedBuffer,
    right: SharedBuffer,
}
impl FrameBuffers {
    pub fn consumers(&self) -> FrameBufferConsumers {
        FrameBufferConsumers {
            left: self.left.consumer(),
            right: self.right.consumer(),
        }
    }
}
impl Index<Eye> for FrameBuffers {
    type Output = SharedBuffer;

    fn index(&self, index: Eye) -> &Self::Output {
        match index {
            Left => &self.left,
            Right => &self.right,
        }
    }
}

pub struct FrameBufferConsumers {
    left: SharedBufferConsumer,
    right: SharedBufferConsumer,
}
impl Index<Eye> for FrameBufferConsumers {
    type Output = SharedBufferConsumer;

    fn index(&self, index: Eye) -> &Self::Output {
        match index {
            Left => &self.left,
            Right => &self.right,
        }
    }
}

impl IndexMut<Eye> for FrameBufferConsumers {
    fn index_mut(&mut self, index: Eye) -> &mut Self::Output {
        match index {
            Left => &mut self.left,
            Right => &mut self.right,
        }
    }
}

pub struct Video {
    cycle: u64,
    displaying: bool,
    drawing: bool,
    game_frame_counter: u8,
    dpctrl_flags: u16,
    xpctrl_flags: u16,
    pending_interrupts: u16,
    enabled_interrupts: u16,
    display_buffer: Buffer,
    memory: Rc<RefCell<Memory>>,
    xp_module: DrawingProcess,
    frame_buffers: Option<FrameBuffers>,
}
impl Video {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Video {
        let state = VideoState::default();
        Video {
            cycle: state.cycle,
            displaying: state.displaying,
            drawing: state.drawing,
            game_frame_counter: state.game_frame_counter,
            dpctrl_flags: state.dpctrl_flags,
            xpctrl_flags: state.xpctrl_flags,
            pending_interrupts: state.pending_interrupts,
            enabled_interrupts: state.enabled_interrupts,
            display_buffer: state.display_buffer,
            memory,
            xp_module: DrawingProcess::new(),
            frame_buffers: None,
        }
    }

    pub fn init(&mut self) {
        self.load_state(&VideoState::default());
        let mut memory = self.memory.borrow_mut();
        memory.write_halfword(DPCTRL, self.dpctrl_flags);
        memory.write_halfword(DPSTTS, self.dpctrl_flags);
        memory.write_halfword(INTPND, self.pending_interrupts);
        memory.write_halfword(INTENB, self.enabled_interrupts);
        memory.write_halfword(VER, 0x0002);
    }

    pub fn save_state(&self) -> VideoState {
        VideoState {
            cycle: self.cycle,
            displaying: self.displaying,
            drawing: self.drawing,
            game_frame_counter: self.game_frame_counter,
            dpctrl_flags: self.dpctrl_flags,
            xpctrl_flags: self.xpctrl_flags,
            pending_interrupts: self.pending_interrupts,
            enabled_interrupts: self.enabled_interrupts,
            display_buffer: self.display_buffer,
        }
    }

    pub fn load_state(&mut self, state: &VideoState) {
        self.cycle = state.cycle;
        self.displaying = state.displaying;
        self.drawing = state.drawing;
        self.game_frame_counter = state.game_frame_counter;
        self.dpctrl_flags = state.dpctrl_flags;
        self.xpctrl_flags = state.xpctrl_flags;
        self.pending_interrupts = state.pending_interrupts;
        self.enabled_interrupts = state.enabled_interrupts;
        self.display_buffer = state.display_buffer;
    }

    pub fn next_event(&self) -> u64 {
        let next_cta_event = if self.drawing && (self.dpctrl_flags & DPBSY) != 0 {
            // When we're "displaying", CTA goes through 96 values over the course of 5ms.
            // We should update the value every ~1040 cycles to achieve that
            let last_display_start = ((self.cycle / 200000) * 200000) + 600000;
            (((self.cycle - last_display_start) / 1040) + 1) * 1040 + last_display_start
        } else {
            u64::MAX
        };
        let next_sbcount_event = if self.drawing && (self.xpctrl_flags & SBOUT) != 0 {
            // When we're "drawing", SBCOUNT goes through 24 values over the course of 5ms.
            // find the current row batch based on how much time has passed
            const CYCLES_PER_ROW_BATCH: u64 = 100000 / 24;
            let last_draw_start = (self.cycle / 200000) * 200000;
            (((self.cycle - last_draw_start) / CYCLES_PER_ROW_BATCH) + 1) * CYCLES_PER_ROW_BATCH
                + last_draw_start
        } else {
            u64::MAX
        };
        // Every other event happens at 1ms intervals
        let next_normal_event = ((self.cycle / 20000) + 1) * 20000;
        next_cta_event
            .min(next_sbcount_event)
            .min(next_normal_event)
    }

    pub fn active_interrupt(&self) -> Option<Exception> {
        if (self.enabled_interrupts & self.pending_interrupts) != 0 {
            return Some(Exception::interrupt(0xfe40, 4));
        }
        None
    }

    pub fn process_event(&mut self, address: usize) -> bool {
        let mut memory = self.memory.borrow_mut();
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
            dpctrl &= !DPRST;
            memory.write_halfword(DPCTRL, dpctrl);
            memory.write_halfword(DPSTTS, dpctrl);
            memory.write_halfword(INTPND, self.pending_interrupts);
            memory.write_halfword(INTENB, self.enabled_interrupts);
        }

        if address == XPCTRL {
            let mut xpctrl = memory.read_halfword(XPCTRL);

            if (xpctrl & XPRST) != 0 {
                self.pending_interrupts &= !XP_INTERRUPTS;
                self.enabled_interrupts &= !XP_INTERRUPTS;
            }

            // Don't let the program overwrite the readonly flags
            xpctrl &= !XP_READONLY_MASK;
            xpctrl |= self.xpctrl_flags;
            xpctrl &= !XPRST;
            memory.write_halfword(XPCTRL, xpctrl);
            memory.write_halfword(XPSTTS, xpctrl);
            memory.write_halfword(INTPND, self.pending_interrupts);
            memory.write_halfword(INTENB, self.enabled_interrupts);
        }

        if address == INTENB {
            self.enabled_interrupts = memory.read_halfword(INTENB);
            if (self.enabled_interrupts & !(DP_INTERRUPTS | XP_INTERRUPTS)) != 0 {
                error!("Unsupported interrupt! 0x{:04x}", self.enabled_interrupts);
                panic!();
            }
        }

        if address == INTCLR {
            self.pending_interrupts &= !memory.read_halfword(INTCLR);
            memory.write_halfword(INTPND, self.pending_interrupts);
        }

        // If we have no active interrupts, the event is handled
        (self.enabled_interrupts & self.pending_interrupts) == 0
    }

    pub fn run(&mut self, target_cycle: u64) -> Result<()> {
        let mut dpctrl = self.memory.borrow().read_halfword(DPCTRL);
        let mut xpctrl = self.memory.borrow().read_halfword(XPCTRL);

        let mut curr_ms = self.cycle / 20000;
        let next_ms = target_cycle / 20000;
        while curr_ms < next_ms {
            curr_ms += 1;
            self.cycle += curr_ms * 20000;

            match curr_ms % 20 {
                0 => {
                    // If we're starting a display frame, check what's enabled
                    self.displaying = (dpctrl & DISP) != 0 && (dpctrl & DPRST) == 0;

                    // If we're starting a game frame,
                    if self.game_frame_counter == 0 {
                        // start drawing (if enabled)
                        self.drawing = (xpctrl & XPEN) != 0;
                        // and let the CPU know
                        self.pending_interrupts |= GAMESTART;
                        let frmcyc = self.memory.borrow().read_halfword(FRMCYC) & 0x0f;
                        self.game_frame_counter = frmcyc as u8;
                    } else {
                        self.drawing = false;
                        self.game_frame_counter -= 1;
                    }

                    // Frame clock up
                    self.dpctrl_flags |= FCLK;
                    self.pending_interrupts |= FRAMESTART;

                    if self.drawing {
                        // Start drawing on whichever buffer was displayed before
                        self.xp_module.start(self.memory.borrow());
                        self.xpctrl_flags |= match self.display_buffer {
                            Buffer0 => F0BSY,
                            Buffer1 => F1BSY,
                        };
                        // Pretend we're drawing line by line
                        self.xpctrl_flags |= SBOUT;

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
                }
                5 => {
                    if self.drawing {
                        // Actually draw on the background buffer
                        self.draw();
                    }
                    if self.displaying {
                        // Actually display the left eye
                        self.build_and_send_frame(Left);
                    }

                    if self.drawing {
                        // "Stop drawing" on background buffer
                        self.xpctrl_flags &= !(F0BSY | F1BSY | SBOUT);
                        self.pending_interrupts |= XPEND;
                    }
                }
                8 => {
                    // "Stop displaying" left eye
                    if self.displaying {
                        self.dpctrl_flags &= !(L0BSY | L1BSY);
                        self.pending_interrupts |= LFBEND;
                    }
                }
                10 => {
                    // Frame clock down
                    self.dpctrl_flags &= !FCLK;
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
                        self.build_and_send_frame(Right);
                    }
                }
                18 => {
                    // "Stop displaying" right eye,
                    if self.displaying {
                        self.dpctrl_flags &= !(R0BSY | R1BSY);
                        self.pending_interrupts |= RFBEND;
                    }
                }
                _ => (),
            };
        }
        self.cycle = target_cycle;

        let mut memory = self.memory.borrow_mut();
        memory.write_halfword(INTPND, self.pending_interrupts);

        // calculate CTA
        if (dpctrl & LOCK) == 0 {
            let mut col_l = 0;
            let mut col_r = 0;
            if self.drawing && (self.dpctrl_flags & DPBSY) != 0 {
                // Find the current column based on how much time has passed since drawing started
                let column = (((self.cycle % 200000) - 60000) / 1040) as u16;
                let eye = if (self.cycle % 400000) < 200000 {
                    Eye::Left
                } else {
                    Eye::Right
                };
                if let Eye::Left = eye {
                    col_l += column;
                } else {
                    col_r += column;
                }
            }
            let cta = ((0x52 + 95 - col_r) << 8) + (0x52 + 95 - col_l);
            memory.write_halfword(CTA, cta);
        }

        // calculate SBCOUNT
        self.xpctrl_flags &= !SBCOUNT_MASK;
        if (self.xpctrl_flags & SBOUT) != 0 {
            // find the current row batch based on how much time has passed
            const CYCLES_PER_ROW_BATCH: u64 = 100000 / 24;
            let row_batch = ((self.cycle % 100000) / CYCLES_PER_ROW_BATCH) as u16;
            self.xpctrl_flags |= row_batch << 8;
        }

        dpctrl &= !DP_READONLY_MASK;
        dpctrl |= self.dpctrl_flags;
        memory.write_halfword(DPCTRL, dpctrl);
        memory.write_halfword(DPSTTS, dpctrl & !DPRST);

        xpctrl &= !XP_READONLY_MASK;
        xpctrl |= self.xpctrl_flags;
        memory.write_halfword(XPCTRL, xpctrl);
        memory.write_halfword(XPSTTS, xpctrl & !XPRST);
        Ok(())
    }

    pub fn claim_frame_buffer_consumers(&mut self) -> FrameBufferConsumers {
        let frame_buffers = FrameBuffers::default();
        let consumers = frame_buffers.consumers();
        self.frame_buffers = Some(frame_buffers);
        consumers
    }

    pub fn load_and_send_frame(&self, eye: Eye, image: &[u8]) {
        let buffer = match &self.frame_buffers {
            Some(fb) => &fb[eye],
            None => return,
        };
        buffer.write(|data| {
            // Input data is RGBA, only copy the R
            for (place, data) in data.iter_mut().zip(image.iter().step_by(4)) {
                *place = *data;
            }
        })
    }

    pub fn build_and_send_frame(&mut self, eye: Eye) {
        let buffer = match &self.frame_buffers {
            Some(fb) => &fb[eye],
            None => return,
        };
        buffer.write(|data| self.write_frame(eye, data));
    }

    fn write_frame(&self, eye: Eye, buffer: &mut [u8]) {
        let buf_address = self.get_buffer_address(eye, self.display_buffer);
        let memory = self.memory.borrow();
        for (col, col_offset) in (0..(384 * 64)).step_by(64).enumerate() {
            // colors to render
            let colors = self.get_brightnesses(&memory, eye, col);

            for (row_offset, top_row) in (0..224).step_by(4).enumerate().step_by(2) {
                let address = buf_address + col_offset + row_offset;
                let pixels = memory.read_halfword(address);
                for (row, pixel) in (0..16).step_by(2).map(|i| (pixels >> i) & 0b11).enumerate() {
                    let index = col + (top_row + row) * 384;
                    buffer[index] = colors[pixel as usize];
                }
            }
        }
    }

    fn get_brightnesses(&self, memory: &Ref<Memory>, eye: Eye, col: usize) -> [u8; 4] {
        let cta_index = 0x52 + 95 - (col / 4);
        let cta = match eye {
            Eye::Left => 0x0003dc00 + (cta_index * 2),
            Eye::Right => 0x0003de00 + (cta_index * 2),
        };
        let ct = memory.read_halfword(cta);
        let repeat = ct >> 8;
        let len = ct & 0xff;
        let color0 = 0; // always black
        let color1 = 255.min(self.get_brightness(memory, BRTA, repeat, len));
        let color2 = 255.min(self.get_brightness(memory, BRTB, repeat, len));
        let color3 = 255.min(color1 + color2 + self.get_brightness(memory, BRTC, repeat, len));
        [color0 as u8, color1 as u8, color2 as u8, color3 as u8]
    }

    fn get_brightness(&self, memory: &Ref<Memory>, address: usize, repeat: u16, len: u16) -> u16 {
        let brt = memory.read_halfword(address);

        // experimentally chosen conversion factor from led-duration-in-50-ns-increments to 8-bit color
        (brt * 19 / 8) + (brt * repeat * len / 40)
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
    fn draw(&mut self) {
        let buffer = self.display_buffer.toggle();
        let mut memory = self.memory.borrow_mut();

        let left_buf_address = self.get_buffer_address(Left, buffer);
        self.xp_module.draw_eye(&mut memory, Left, left_buf_address);

        let right_buf_address = self.get_buffer_address(Right, buffer);
        self.xp_module
            .draw_eye(&mut memory, Right, right_buf_address);
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::memory::Memory;
    use crate::emulator::video::{
        Video, DISP, DPCTRL, DPRST, FRAMESTART, FRMCYC, GAMESTART, INTCLR, INTENB, INTPND,
        SBCOUNT_MASK, SBOUT, XPEND, XPRST,
    };
    use crate::emulator::video::{DPSTTS, FCLK, L0BSY, L1BSY, R0BSY, R1BSY, SCANRDY};
    use crate::emulator::video::{F0BSY, F1BSY, XPCTRL, XPEN, XPSTTS};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn ms_to_cycles(ms: u64) -> u64 {
        ms * 20000
    }

    fn write_dpctrl(video: &mut Video, memory: &RefCell<Memory>, value: u16) {
        memory.borrow_mut().write_halfword(DPCTRL, value);
        video.process_event(DPCTRL);
    }
    fn write_xpctrl(video: &mut Video, memory: &RefCell<Memory>, value: u16) {
        memory.borrow_mut().write_halfword(XPCTRL, value);
        video.process_event(XPCTRL);
    }
    fn write_intenb(video: &mut Video, memory: &RefCell<Memory>, value: u16) {
        memory.borrow_mut().write_halfword(INTENB, value);
        video.process_event(INTENB);
    }
    fn write_intclr(video: &mut Video, memory: &RefCell<Memory>, value: u16) {
        memory.borrow_mut().write_halfword(INTCLR, value);
        video.process_event(INTCLR);
    }

    fn get_video() -> (Video, Rc<RefCell<Memory>>) {
        let memory = Rc::new(RefCell::new(Memory::new()));
        let video = Video::new(Rc::clone(&memory));
        (video, memory)
    }

    #[test]
    fn can_emulate_two_frames_of_dpstts_while_drawing_is_off() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, DISP);

        // start 2 frames in, because that's the first time we see a rising FCLK
        video.run(ms_to_cycles(40)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(ms_to_cycles(43)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(DPSTTS),
            DISP | SCANRDY | FCLK | L0BSY
        );

        video.run(ms_to_cycles(48)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(ms_to_cycles(50)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(ms_to_cycles(53)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(DPSTTS),
            DISP | SCANRDY | R0BSY
        );

        video.run(ms_to_cycles(58)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(ms_to_cycles(60)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(ms_to_cycles(63)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(DPSTTS),
            DISP | SCANRDY | FCLK | L0BSY
        );

        video.run(ms_to_cycles(68)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(ms_to_cycles(70)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(ms_to_cycles(73)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(DPSTTS),
            DISP | SCANRDY | R0BSY
        );

        video.run(ms_to_cycles(78)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(ms_to_cycles(80)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);
    }

    #[test]
    fn can_emulate_two_frames_of_dpstts_while_drawing_is_on() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, DISP);
        write_xpctrl(&mut video, &memory, XPEN);

        // start 2 frames in, because that's the first time we see a rising FCLK
        video.run(ms_to_cycles(40)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(ms_to_cycles(43)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(DPSTTS),
            DISP | SCANRDY | FCLK | L0BSY
        );

        video.run(ms_to_cycles(48)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(ms_to_cycles(50)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(ms_to_cycles(53)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(DPSTTS),
            DISP | SCANRDY | R0BSY
        );

        video.run(ms_to_cycles(58)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(ms_to_cycles(60)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(ms_to_cycles(63)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(DPSTTS),
            DISP | SCANRDY | FCLK | L1BSY
        );

        video.run(ms_to_cycles(68)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);

        video.run(ms_to_cycles(70)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(ms_to_cycles(73)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(DPSTTS),
            DISP | SCANRDY | R1BSY
        );

        video.run(ms_to_cycles(78)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY);

        video.run(ms_to_cycles(80)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);
    }

    #[test]
    fn can_render_when_disp_and_dprst_are_both_set() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, DISP | DPRST);
        write_xpctrl(&mut video, &memory, XPEN);

        // start 2 frames in, because that's the first time we see a rising FCLK
        video.run(ms_to_cycles(40)).unwrap();
        assert_eq!(memory.borrow().read_halfword(DPSTTS), DISP | SCANRDY | FCLK);
    }

    #[test]
    fn can_emulate_two_frames_of_xpstts_while_drawing_is_on() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, DISP);
        write_xpctrl(&mut video, &memory, XPEN);

        // start 2 frames in, because that's the first time we see a rising FCLK
        video.run(ms_to_cycles(40)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN | F1BSY | SBOUT);

        video.run(ms_to_cycles(45)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN);

        video.run(ms_to_cycles(50)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN);

        video.run(ms_to_cycles(55)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN);

        video.run(ms_to_cycles(60)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN | F0BSY | SBOUT);

        video.run(ms_to_cycles(65)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN);

        video.run(ms_to_cycles(70)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN);

        video.run(ms_to_cycles(75)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN);

        video.run(ms_to_cycles(80)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN | F1BSY | SBOUT);
    }

    #[test]
    fn can_turn_off_xpstts_midframe() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, DISP);

        // turn on drawing 2 frames in, because that's the first time we see a rising FCLK
        video.run(ms_to_cycles(39)).unwrap();
        write_xpctrl(&mut video, &memory, XPEN);
        video.run(ms_to_cycles(40)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN | F0BSY | SBOUT);

        // turn off drawing
        write_xpctrl(&mut video, &memory, 0);
        video.run(ms_to_cycles(42)).unwrap();
        assert_eq!(
            memory.borrow().read_halfword(XPSTTS) & !SBCOUNT_MASK,
            F0BSY | SBOUT
        );

        video.run(ms_to_cycles(45)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), 0);

        video.run(ms_to_cycles(60)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), 0);
    }

    #[test]
    fn can_turn_on_xpstts_midframe() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, DISP);

        // start >2 frames in, because that's the first time we see a rising FCLK
        video.run(ms_to_cycles(41)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), 0);

        // turn on drawing
        write_xpctrl(&mut video, &memory, XPEN);
        video.run(ms_to_cycles(42)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN);

        video.run(ms_to_cycles(45)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN);

        video.run(ms_to_cycles(60)).unwrap();
        assert_eq!(memory.borrow().read_halfword(XPSTTS), XPEN | F0BSY | SBOUT);
    }

    #[test]
    fn can_trigger_framestart_interrupt() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, 0);

        // While INTENB is unset, set INTPND but don't trigger interrupts
        video.run(ms_to_cycles(37)).unwrap();
        assert_ne!(memory.borrow().read_halfword(INTPND) & FRAMESTART, 0);
        assert!(video.active_interrupt().is_none());

        // Interrupt can be cleared by writing to DPRST
        write_dpctrl(&mut video, &memory, DISP | DPRST);
        video.run(ms_to_cycles(38)).unwrap();
        assert_eq!(memory.borrow().read_halfword(INTPND) & FRAMESTART, 0);

        // Interrupt is triggered on FCLK going high
        write_dpctrl(&mut video, &memory, DISP);
        write_intenb(&mut video, &memory, FRAMESTART);
        video.run(ms_to_cycles(40)).unwrap();
        assert_ne!(memory.borrow().read_halfword(INTPND) & FRAMESTART, 0);
        assert!(video.active_interrupt().is_some());

        // Interrupt can be cleared by writing to INTCLR
        write_intclr(&mut video, &memory, FRAMESTART);
        video.run(ms_to_cycles(41)).unwrap();
        assert_eq!(memory.borrow().read_halfword(INTPND) & FRAMESTART, 0);
        assert!(video.active_interrupt().is_none());
    }

    #[test]
    fn can_trigger_gamestart_interrupt_on_game_frames() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, 0);
        write_xpctrl(&mut video, &memory, 0);
        // set FRMCYC to 1 so that there are 1+1==2 display frames per game frame
        // note that this only takes effect after the first game frame
        memory.borrow_mut().write_halfword(FRMCYC, 1);

        // While INTENB is unset, set INTPND but don't trigger interrupts
        video.run(ms_to_cycles(37)).unwrap();
        assert_ne!(memory.borrow().read_halfword(INTPND) & GAMESTART, 0);
        assert!(video.active_interrupt().is_none());

        // Interrupt can be cleared by writing to DPRST
        write_dpctrl(&mut video, &memory, DISP | DPRST);
        video.run(ms_to_cycles(38)).unwrap();
        assert_eq!(memory.borrow().read_halfword(INTPND) & GAMESTART, 0);

        // Interrupt is NOT triggered on FCLK going high, because only one display frame has passed
        write_dpctrl(&mut video, &memory, DISP);
        write_intenb(&mut video, &memory, GAMESTART);
        video.run(ms_to_cycles(40)).unwrap();
        assert_eq!(memory.borrow().read_halfword(INTPND) & GAMESTART, 0);
        assert!(video.active_interrupt().is_none());

        // One display frame later, the interrupt is triggered for real
        video.run(ms_to_cycles(60)).unwrap();
        assert_ne!(memory.borrow().read_halfword(INTPND) & GAMESTART, 0);
        assert!(video.active_interrupt().is_some());
    }

    #[test]
    fn can_trigger_xpend_interrupt() {
        let (mut video, memory) = get_video();

        video.init();
        write_dpctrl(&mut video, &memory, DISP);
        write_xpctrl(&mut video, &memory, XPEN);

        // While INTENB is unset, set INTPND but don't trigger interrupts
        video.run(ms_to_cycles(37)).unwrap();
        assert_ne!(memory.borrow().read_halfword(INTPND) & XPEND, 0);
        assert!(video.active_interrupt().is_none());

        // Interrupt can be cleared by writing to XPRST
        write_xpctrl(&mut video, &memory, XPRST);
        video.run(ms_to_cycles(38)).unwrap();
        assert_eq!(memory.borrow().read_halfword(INTPND) & XPEND, 0);

        // Interrupt is triggered when "drawing" completes
        write_xpctrl(&mut video, &memory, XPEN);
        write_intenb(&mut video, &memory, XPEND);
        video.run(ms_to_cycles(45)).unwrap();
        assert_ne!(memory.borrow().read_halfword(INTPND) & XPEND, 0);
        assert!(video.active_interrupt().is_some());

        // Interrupt can be cleared by writing to INTCLR
        write_intclr(&mut video, &memory, XPEND);
        video.run(ms_to_cycles(46)).unwrap();
        assert_eq!(memory.borrow().read_halfword(INTPND) & XPEND, 0);
        assert!(video.active_interrupt().is_none());
    }
}
