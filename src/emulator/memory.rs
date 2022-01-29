use crate::emulator::cpu::Event;
use anyhow::Result;
use log::info;
use serde_derive::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Region {
    Vram = 0,
    Audio = 1,
    Hardware = 2,
    Dram = 5,
    Sram = 6,
    Rom = 7,
}
const VRAM_SIZE: usize = 0x00080000;
const AUDIO_SIZE: usize = 0x00000800;
const HARDWARE_SIZE: usize = 0x00000040;
const DRAM_SIZE: usize = 0x00010000;

struct MemoryRegion {
    kind: Region,
    value: Vec<u8>,
    mask: usize,
}
impl MemoryRegion {
    pub fn new(kind: Region, size: usize) -> Self {
        MemoryRegion {
            kind,
            value: vec![0; size],
            mask: size - 1,
        }
    }
    pub fn from(kind: Region, data: &[u8]) -> Self {
        MemoryRegion {
            kind,
            value: Vec::from(data),
            mask: data.len() - 1,
        }
    }
    pub fn clear(&mut self) {
        self.value[0x00000000..self.mask].fill(0);
    }
    pub fn write_byte(&mut self, address: usize, value: u8) -> Option<Event> {
        let (address, event) = self.resolve_address(address);
        self.value[address] = value;
        event
    }
    pub fn write_halfword(&mut self, address: usize, value: u16) -> Option<Event> {
        let (address, event) = self.resolve_address(address);
        self.value[address..address + 2].copy_from_slice(&value.to_le_bytes());
        event
    }
    pub fn write_word(&mut self, address: usize, value: u32) -> Option<Event> {
        let (address, event) = self.resolve_address(address);
        self.value[address..address + 4].copy_from_slice(&value.to_le_bytes());
        event
    }
    pub fn read_byte(&self, address: usize) -> u8 {
        let (address, _) = self.resolve_address(address);
        self.value[address]
    }
    pub fn read_halfword(&self, address: usize) -> u16 {
        let (address, _) = self.resolve_address(address);
        let bytes: &[u8; 2] = &self.value[address..address + 2].try_into().unwrap();
        u16::from_le_bytes(*bytes)
    }
    pub fn read_word(&self, address: usize) -> u32 {
        let (address, _) = self.resolve_address(address);
        let bytes: &[u8; 4] = &self.value[address..address + 4].try_into().unwrap();
        u32::from_le_bytes(*bytes)
    }
    fn resolve_address(&self, address: usize) -> (usize, Option<Event>) {
        let relative = address & self.mask;
        match self.kind {
            Region::Vram => {
                match relative {
                    0x0005f800..=0x0005f870 => {
                        (relative, Some(Event::DisplayControlWrite { address }))
                    }
                    // The following ranges mirror data from the character tables
                    0x00078000..=0x00079fff => (relative - 0x72000, None),
                    0x0007a000..=0x0007bfff => (relative - 0x6C000, None),
                    0x0007c000..=0x0007dfff => (relative - 0x66000, None),
                    0x0007e000..=0x0007ffff => (relative - 0x60000, None),
                    _ => (relative, None),
                }
            }
            Region::Audio => (relative, Some(Event::AudioWrite { address })),
            Region::Hardware => (relative, Some(Event::HardwareWrite { address })),
            _ => (relative, None),
        }
    }
}

pub struct Memory {
    regions: [Option<MemoryRegion>; 8],
}
impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
impl Memory {
    pub fn new() -> Self {
        Self {
            regions: [
                Some(MemoryRegion::new(Region::Vram, VRAM_SIZE)),
                Some(MemoryRegion::new(Region::Audio, AUDIO_SIZE)),
                Some(MemoryRegion::new(Region::Hardware, HARDWARE_SIZE)),
                None,
                None, // Game Pak Expansion (never used)
                Some(MemoryRegion::new(Region::Dram, DRAM_SIZE)),
                None, // Sram (loaded later)
                None, // Rom (loaded later)
            ],
        }
    }
    pub fn vram_only() -> Self {
        Self {
            regions: [
                Some(MemoryRegion::new(Region::Vram, VRAM_SIZE)),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        }
    }

    pub fn load_game_pak(&mut self, rom: &[u8], sram: &[u8]) -> Result<()> {
        let rom_size = rom.len();
        if rom_size.count_ones() != 1 {
            return Err(anyhow::anyhow!("ROM size must be a power of two"));
        }
        if rom_size > 0x01000000 {
            return Err(anyhow::anyhow!("ROM size must be <= 16Mb"));
        }

        if sram.is_empty() {
            *self.mut_region(Region::Sram) = None;
        } else {
            *self.mut_region(Region::Sram) = Some(MemoryRegion::from(Region::Sram, sram));
        }
        info!("Loaded SRAM");
        *self.mut_region(Region::Rom) = Some(MemoryRegion::from(Region::Rom, rom));
        info!("Loaded ROM");
        self.init();
        info!("Initialized all memory");
        Ok(())
    }

    pub fn unload_game_pak(&mut self) {
        *self.mut_region(Region::Sram) = None;
        *self.mut_region(Region::Rom) = None;
    }

    fn init(&mut self) {
        if let Some(region) = self.mut_region(Region::Vram) {
            region.clear();
        }
        if let Some(region) = self.mut_region(Region::Audio) {
            region.clear();
        }
        if let Some(region) = self.mut_region(Region::Hardware) {
            region.clear();
        }
        if let Some(region) = self.mut_region(Region::Dram) {
            region.clear();
        }
    }

    pub fn write_byte(&mut self, address: usize, value: u8) -> Option<Event> {
        match self.mut_region_of(address) {
            Some(region) => region.write_byte(address, value),
            None => None,
        }
    }
    pub fn write_halfword(&mut self, address: usize, value: u16) -> Option<Event> {
        match self.mut_region_of(address) {
            Some(region) => region.write_halfword(address, value),
            None => None,
        }
    }
    pub fn write_word(&mut self, address: usize, value: u32) -> Option<Event> {
        match self.mut_region_of(address) {
            Some(region) => region.write_word(address, value),
            None => None,
        }
    }
    pub fn read_byte(&self, address: usize) -> u8 {
        match self.get_region_of(address) {
            Some(region) => region.read_byte(address),
            None => 0,
        }
    }
    pub fn read_halfword(&self, address: usize) -> u16 {
        match self.get_region_of(address) {
            Some(region) => region.read_halfword(address),
            None => 0,
        }
    }
    pub fn read_word(&self, address: usize) -> u32 {
        match self.get_region_of(address) {
            Some(region) => region.read_word(address),
            None => 0,
        }
    }

    pub fn read_region(&self, region: Region) -> Option<&[u8]> {
        self.get_region(region)
            .as_ref()
            .map(|region| region.value.as_slice())
    }

    pub fn write_region(&mut self, region: Region) -> Option<&mut [u8]> {
        self.mut_region(region)
            .as_mut()
            .map(|region| region.value.as_mut_slice())
    }

    fn get_region(&self, region: Region) -> &Option<MemoryRegion> {
        &self.regions[region as usize]
    }
    fn mut_region(&mut self, region: Region) -> &mut Option<MemoryRegion> {
        &mut self.regions[region as usize]
    }
    fn get_region_of(&self, address: usize) -> Option<&MemoryRegion> {
        let index = (address >> 24) & 0x07;
        self.regions[index].as_ref()
    }
    fn mut_region_of(&mut self, address: usize) -> Option<&mut MemoryRegion> {
        let index = (address >> 24) & 0x07;
        self.regions[index].as_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::memory::Memory;

    #[test]
    fn can_create() {
        Memory::new();
    }

    #[test]
    fn can_read_and_write_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0x00000000, 0x42);
        assert_eq!(memory.read_byte(0x00000000), 0x42);
    }

    #[test]
    fn can_read_and_write_halfword() {
        let mut memory = Memory::new();
        memory.write_halfword(0x00000000, 0x1234);
        assert_eq!(memory.read_byte(0x00000000), 0x34);
        assert_eq!(memory.read_byte(0x00000001), 0x12);
        assert_eq!(memory.read_halfword(0x00000000), 0x1234);
    }

    #[test]
    fn can_read_and_write_word() {
        let mut memory = Memory::new();
        memory.write_word(0x00000000, 0x12345678);
        assert_eq!(memory.read_byte(0x00000000), 0x78);
        assert_eq!(memory.read_byte(0x00000001), 0x56);
        assert_eq!(memory.read_byte(0x00000002), 0x34);
        assert_eq!(memory.read_byte(0x00000003), 0x12);
        assert_eq!(memory.read_word(0x00000000), 0x12345678);
    }

    #[test]
    fn high_addresses_are_mirrored() {
        let mut memory = Memory::new();
        memory.write_byte(0x10000000, 0xFF);
        assert_eq!(memory.read_byte(0x00000000), 0xFF);
    }

    #[test]
    fn frame_buffers_are_mirrored() {
        let mut memory = Memory::new();
        memory.write_byte(0x00000000, 0xFF);
        assert_eq!(memory.read_byte(0x00080000), 0xFF);
        assert_eq!(memory.read_byte(0x00100000), 0xFF);
    }

    #[test]
    fn character_tables_are_mirrored() {
        let mut memory = Memory::new();
        memory.write_byte(0x00006000, 0x01);
        memory.write_byte(0x0000E000, 0x02);
        memory.write_byte(0x00016000, 0x03);
        memory.write_byte(0x0001E000, 0x04);
        assert_eq!(memory.read_byte(0x00078000), 0x01);
        assert_eq!(memory.read_byte(0x0007A000), 0x02);
        assert_eq!(memory.read_byte(0x0007C000), 0x03);
        assert_eq!(memory.read_byte(0x0007E000), 0x04);
    }

    #[test]
    fn wram_is_mirrored() {
        let mut memory = Memory::new();
        memory.write_byte(0x05123456, 0x63);
        assert_eq!(memory.read_byte(0x05F23456), 0x63);
    }

    #[test]
    fn can_load_game_pak_rom() {
        let mut memory = Memory::new();
        memory
            .load_game_pak(&[0x78, 0x56, 0x34, 0x12], &[])
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "ROM size must be a power of two")]
    fn asserts_rom_is_power_of_two() {
        let mut memory = Memory::new();
        memory.load_game_pak(&[0x78, 0x56, 0x34], &[]).unwrap();
    }

    #[test]
    #[should_panic(expected = "ROM size must be <= 16Mb")]
    fn asserts_rom_is_small_enough() {
        let mut memory = Memory::new();
        let too_much_rom = vec![0u8; 0x02000000];
        memory.load_game_pak(too_much_rom.as_slice(), &[]).unwrap();
    }

    #[test]
    fn can_read_game_pak_rom() {
        let mut memory = Memory::new();
        memory
            .load_game_pak(&[0x78, 0x56, 0x34, 0x12], &[])
            .unwrap();
        assert_eq!(memory.read_word(0x07000000), 0x12345678);
    }

    #[test]
    fn can_read_game_pak_rom_mirrored_by_size() {
        let mut memory = Memory::new();
        memory
            .load_game_pak(&[0x78, 0x56, 0x34, 0x12, 0x89, 0x57, 0x34, 0x06], &[])
            .unwrap();
        assert_eq!(memory.read_word(0x07000000), 0x12345678);
        assert_eq!(memory.read_word(0x07000004), 0x06345789);
        assert_eq!(memory.read_word(0x07000008), 0x12345678);
    }

    #[test]
    fn can_read_game_pak_ram_mirrored_by_8k() {
        let mut memory = Memory::new();
        let mut sram = vec![0; 0x00002000];
        sram[0] = 0x78;
        sram[1] = 0x56;
        sram[2] = 0x34;
        sram[3] = 0x12;
        memory.load_game_pak(&[0x69], &sram).unwrap();
        assert_eq!(memory.read_word(0x06000000), 0x12345678);
        assert_eq!(memory.read_word(0x06000004), 0);
        assert_eq!(memory.read_word(0x06002000), 0x12345678);
    }
}
