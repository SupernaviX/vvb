use crate::emulator::storage::Storage;
use anyhow::Result;
use std::sync::{mpsc, Arc, Mutex};

pub const VB_WIDTH: usize = 384;
pub const VB_HEIGHT: usize = 224;
pub const FRAME_SIZE: usize = VB_WIDTH * VB_HEIGHT;

const DPSTTS: usize = 0x0005f820;
const DPCTRL: usize = 0x0005f822;
const XPSTTS: usize = 0x0005f840;
const XPCTRL: usize = 0x0005f842;

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
            match curr_ms % 20 {
                0 => self.set_flags(storage, DPCTRL, 0x0080),
                3 | 13 => {
                    let curr_buffer = (curr_ms % 40) / 10;
                    self.set_flags(storage, DPCTRL, 0x0004 << (curr_buffer as i16));
                    self.set_flags(storage, XPCTRL, 0x0004 << (1 - curr_buffer as i16 / 2));
                }
                8 | 18 => {
                    let curr_buffer = (curr_ms % 40) / 10;
                    self.clear_flags(storage, DPCTRL, 0x0004 << (curr_buffer as i16));
                    self.clear_flags(storage, XPCTRL, 0x0004 << (1 - curr_buffer as i16 / 2));
                }
                10 => self.clear_flags(storage, DPCTRL, 0x0080),
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
            channel.send(Frame {
                eye,
                buffer: Arc::clone(buffer),
            })?;
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
