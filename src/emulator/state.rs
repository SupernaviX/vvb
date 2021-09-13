use anyhow::Result;
use ciborium::{de, ser};
use serde_derive::{Deserialize, Serialize};
use std::fs::File;

use super::cpu::CpuState;
use super::memory::Region;

const VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
pub enum SaveStateData {
    Memory(Region, #[serde(with = "serde_bytes")] Vec<u8>),
    Cpu(Box<CpuState>),
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
