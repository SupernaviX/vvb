use crate::emulator::audio::AudioPlayer;
use anyhow::Result;

pub struct NoopAudio;
impl NoopAudio {
    pub fn new(_player: AudioPlayer) -> Result<Self> {
        Ok(NoopAudio)
    }
    pub fn start(&mut self) -> Result<()> {
        Ok(())
    }
    pub fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}
