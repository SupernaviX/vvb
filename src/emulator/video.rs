use crate::emulator::storage::Storage;
use anyhow::Result;
use std::sync::{mpsc, Arc, Mutex};

pub const VB_WIDTH: usize = 384;
pub const VB_HEIGHT: usize = 224;
pub const FRAME_SIZE: usize = VB_WIDTH * VB_HEIGHT * 4;

const DPSTTS: usize = 0x0005f820;
const DPCTRL: usize = 0x0005f822;

#[derive(Copy, Clone)]
pub enum Eye {
    Left,
    Right,
}
pub type EyeBuffer = [u8; FRAME_SIZE];

pub struct Frame {
    pub eye: Eye,
    pub buffer: Arc<Mutex<EyeBuffer>>,
}

pub type FrameChannel = mpsc::Receiver<Frame>;

pub struct Video {
    frame_channel: Option<mpsc::Sender<Frame>>,
    buffers: [Arc<Mutex<EyeBuffer>>; 2],
}
impl Video {
    pub fn new() -> Video {
        Video {
            frame_channel: None,
            buffers: [
                Arc::new(Mutex::new([0; FRAME_SIZE])),
                Arc::new(Mutex::new([0; FRAME_SIZE])),
            ],
        }
    }

    pub fn init(&self, storage: &mut Storage) {
        storage.write_halfword(DPSTTS, 0x0040);
        storage.write_halfword(DPCTRL, 0x0040);
    }

    pub fn run(&mut self, storage: &mut Storage, _until_cycle: u32) -> Result<()> {
        storage.write_halfword(DPSTTS, storage.read_halfword(DPCTRL));
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
        for (place, data) in buffer.iter_mut().zip(image.iter()) {
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
}
