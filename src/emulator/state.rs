use anyhow::Result;
use ciborium::{de, ser};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;

use super::audio::AudioState;
use super::cpu::CpuState;
use super::hardware::HardwareState;
use super::memory::Region;
use super::video::VideoState;

const VERSION: u8 = 1;

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
pub struct GlobalState {
    pub cycle: u64,
    pub tick_calls: u64,
}

#[derive(Serialize, Deserialize)]
pub enum SaveStateData {
    Global(GlobalState),
    Memory(Region, #[serde(with = "serde_bytes")] Vec<u8>),
    Cpu(Box<CpuState>),
    Video(VideoState),
    Hardware(HardwareState),
    Audio(Box<AudioState>),
}

pub fn save_state(filename: &str, data: &[SaveStateData]) -> Result<()> {
    let mut file = File::create(filename)?;
    ser::into_writer(&VERSION, &mut file)?;
    ser::into_writer(data, &mut file)?;
    Ok(())
}

pub fn load_state(filename: &str) -> Result<Vec<SaveStateData>> {
    let file = File::open(filename)?;
    let version: u8 = de::from_reader(&file)?;
    if version != VERSION {
        return Err(anyhow::anyhow!(
            "Could not read state with version {}",
            version
        ));
    }
    let result = de::from_reader(&file)?;
    Ok(result)
}
