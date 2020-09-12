use crate::emulator::cpu::Event;
use anyhow::Result;
use std::convert::TryInto;

enum Address {
    Mapped(usize, Option<Event>),
    Unmapped,
}

pub struct Memory {
    memory: Vec<u8>,
    rom_mask: usize,
}
impl Memory {
    pub fn new() -> Memory {
        let mut memory = Memory {
            memory: vec![0; 0x07FFFFFF],
            rom_mask: 0,
        };
        memory.init();
        memory
    }

    pub fn load_game_pak_rom(&mut self, rom: &[u8]) -> Result<()> {
        let rom_size = rom.len();
        if rom_size.count_ones() != 1 {
            return Err(anyhow::anyhow!("ROM size must be a power of two"));
        }
        if rom_size > 0x00FFFFFF {
            return Err(anyhow::anyhow!("ROM size must be <= 16Mb"));
        }
        self.rom_mask = 0x07000000 + rom_size - 1;
        self.memory[0x07000000..0x07000000 + rom_size].copy_from_slice(rom);
        self.init();
        Ok(())
    }

    fn init(&mut self) {
        for vram in self.memory[0x00000000..=0x00077fff].iter_mut() {
            *vram = 0;
        }
        for hardware in self.memory[0x02000000..=0x0200003f].iter_mut() {
            *hardware = 0;
        }
        for dram in self.memory[0x05000000..=0x0500ffff].iter_mut() {
            *dram = 0;
        }
    }

    pub fn write_byte(&mut self, address: usize, value: u8) -> Option<Event> {
        if let Address::Mapped(resolved, event) = self.resolve_address(address) {
            self.memory[resolved] = value;
            return event;
        }
        None
    }

    pub fn write_halfword(&mut self, address: usize, value: u16) -> Option<Event> {
        if let Address::Mapped(resolved, event) = self.resolve_address(address) {
            self.memory[resolved..resolved + 2].copy_from_slice(&value.to_le_bytes());
            return event;
        }
        None
    }

    pub fn write_word(&mut self, address: usize, value: u32) -> Option<Event> {
        if let Address::Mapped(resolved, event) = self.resolve_address(address) {
            self.memory[resolved..resolved + 4].copy_from_slice(&value.to_le_bytes());
            return event;
        }
        None
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        match self.resolve_address(address) {
            Address::Mapped(resolved, _) => self.memory[resolved],
            Address::Unmapped => 0,
        }
    }

    pub fn read_halfword(&self, address: usize) -> u16 {
        let address = match self.resolve_address(address) {
            Address::Mapped(resolved, _) => resolved,
            Address::Unmapped => return 0,
        };
        let bytes: &[u8; 2] = self.memory[address..address + 2].try_into().unwrap();
        u16::from_le_bytes(*bytes)
    }

    pub fn read_word(&self, address: usize) -> u32 {
        let address = match self.resolve_address(address) {
            Address::Mapped(resolved, _) => resolved,
            Address::Unmapped => return 0,
        };
        let bytes: &[u8; 4] = self.memory[address..address + 4].try_into().unwrap();
        u32::from_le_bytes(*bytes)
    }

    fn resolve_address(&self, address: usize) -> Address {
        let address = address & 0x07FFFFFF;
        match address {
            0x00000000..=0x00FFFFFF => self.resolve_vip_address(address),
            0x01000000..=0x01FFFFFF => Address::Unmapped, // TODO: VSU
            0x02000000..=0x02FFFFFF => self.resolve_hardware_address(address),
            0x03000000..=0x03FFFFFF => Address::Unmapped,
            0x04000000..=0x04FFFFFF => Address::Unmapped, // Game Pak Expansion, never used
            0x05000000..=0x05FFFFFF => self.resolve_wram_address(address),
            0x06000000..=0x06FFFFFF => Address::Unmapped, // TODO: Game Pak RAM
            0x07000000..=0x07FFFFFF => self.resolve_game_pak_rom_address(address),
            _ => unreachable!("Math broke"),
        }
    }

    fn resolve_vip_address(&self, address: usize) -> Address {
        let address = address & 0x0007ffff;
        match address {
            0x0005f800..=0x0005f870 => {
                Address::Mapped(address, Some(Event::DisplayControlWrite { address }))
            }
            0x00000000..=0x00077fff => Address::Mapped(address, None),
            // The following ranges mirror data from the character tables
            0x00078000..=0x00079fff => Address::Mapped(address - 0x72000, None),
            0x0007a000..=0x0007bfff => Address::Mapped(address - 0x6C000, None),
            0x0007c000..=0x0007dfff => Address::Mapped(address - 0x66000, None),
            0x0007e000..=0x0007ffff => Address::Mapped(address - 0x60000, None),
            _ => unreachable!("SCP-033 containment breach"),
        }
    }

    fn resolve_hardware_address(&self, address: usize) -> Address {
        let address = address & 0x0200003f;
        Address::Mapped(address, Some(Event::HardwareWrite { address }))
    }

    fn resolve_wram_address(&self, address: usize) -> Address {
        Address::Mapped(address & 0x0500FFFF, None)
    }

    fn resolve_game_pak_rom_address(&self, address: usize) -> Address {
        Address::Mapped(address & self.rom_mask, None)
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
        memory.load_game_pak_rom(&[0x78, 0x56, 0x34, 0x12]).unwrap();
    }

    #[test]
    #[should_panic(expected = "ROM size must be a power of two")]
    fn asserts_rom_is_power_of_two() {
        let mut memory = Memory::new();
        memory.load_game_pak_rom(&[0x78, 0x56, 0x34]).unwrap();
    }

    #[test]
    #[should_panic(expected = "ROM size must be <= 16Mb")]
    fn asserts_rom_is_small_enough() {
        let mut memory = Memory::new();
        let too_much_rom = vec![0u8; 0x01000000];
        memory.load_game_pak_rom(too_much_rom.as_slice()).unwrap();
    }

    #[test]
    fn can_read_game_pak_rom() {
        let mut memory = Memory::new();
        memory.load_game_pak_rom(&[0x78, 0x56, 0x34, 0x12]).unwrap();
        assert_eq!(memory.read_word(0x07000000), 0x12345678);
    }

    #[test]
    fn can_read_game_pak_rom_mirrored_by_size() {
        let mut memory = Memory::new();
        memory
            .load_game_pak_rom(&[0x78, 0x56, 0x34, 0x12, 0x89, 0x57, 0x34, 0x06])
            .unwrap();
        assert_eq!(memory.read_word(0x07000000), 0x12345678);
        assert_eq!(memory.read_word(0x07000004), 0x06345789);
        assert_eq!(memory.read_word(0x07000008), 0x12345678);
    }
}
