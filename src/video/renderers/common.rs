use crate::emulator::video::{Eye, FrameChannel};
use anyhow::Result;
use std::sync::mpsc::TryRecvError;

pub trait RenderLogic {
    fn init(&mut self) -> Result<()>;
    fn resize(&mut self, screen_size: (i32, i32)) -> Result<()>;
    fn update(&mut self, eye: Eye, buffer: &[u8]) -> Result<()>;
    fn draw(&self) -> Result<()>;
}

pub struct Renderer<TLogic: RenderLogic> {
    frame_channel: FrameChannel,
    disconnected: bool,
    pub logic: TLogic,
}
impl<TLogic: RenderLogic> Renderer<TLogic> {
    pub fn new(frame_channel: FrameChannel, logic: TLogic) -> Self {
        Self {
            frame_channel,
            disconnected: false,
            logic,
        }
    }

    pub fn on_surface_created(&mut self) -> Result<()> {
        if self.disconnected {
            return Ok(());
        }
        self.logic.init()
    }

    pub fn on_surface_changed(&mut self, width: i32, height: i32) -> Result<()> {
        if self.disconnected {
            return Ok(());
        }
        self.logic.resize((width, height))
    }

    pub fn on_draw_frame(&mut self) -> Result<()> {
        if self.disconnected {
            return Ok(());
        }
        match self.frame_channel.try_recv() {
            Ok(frame) => {
                let eye = frame.eye;
                let mut buffer = frame.buffer.lock().expect("Buffer lock was poisoned!");
                self.logic.update(eye, buffer.read())?;
                drop(buffer);
                self.logic.draw()
            }
            Err(TryRecvError::Empty) => self.logic.draw(),
            Err(TryRecvError::Disconnected) => {
                self.disconnected = true;
                Ok(())
            }
        }
    }
}
