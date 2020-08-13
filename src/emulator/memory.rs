use std::convert::TryInto;

enum Address {
    Mapped(usize),
    Unmapped,
}

pub struct Memory {
    contents: Vec<u8>,
}
impl Memory {
    pub fn new() -> Memory {
        Memory {
            contents: vec![0; 0x07FFFFFF],
        }
    }

    pub fn write_byte(&mut self, address: usize, value: u8) {
        if let Address::Mapped(resolved) = Memory::resolve_address(address) {
            self.contents[resolved] = value;
        }
    }

    pub fn read_byte(&self, address: usize) -> u8 {
        match Memory::resolve_address(address) {
            Address::Mapped(resolved) => self.contents[resolved],
            Address::Unmapped => 0,
        }
    }

    pub fn read_halfword(&self, address: usize) -> i16 {
        let address = match Memory::resolve_address(address) {
            Address::Mapped(resolved) => resolved,
            Address::Unmapped => return 0,
        };
        let bytes: &[u8; 2] = self.contents[address..address + 2].try_into().unwrap();
        i16::from_le_bytes(*bytes)
    }

    pub fn read_word(&self, address: usize) -> i32 {
        let address = match Memory::resolve_address(address) {
            Address::Mapped(resolved) => resolved,
            Address::Unmapped => return 0,
        };
        let bytes: &[u8; 4] = self.contents[address..address + 4].try_into().unwrap();
        i32::from_le_bytes(*bytes)
    }

    fn resolve_address(address: usize) -> Address {
        let address = address & 0x07FFFFFF;
        match address {
            0x00000000..=0x00FFFFFF => Memory::resolve_vip_address(address),
            0x01000000..=0x01FFFFFF => Address::Unmapped, // TODO: VSU
            0x02000000..=0x02FFFFFF => Address::Unmapped, // TODO: hardware
            0x03000000..=0x03FFFFFF => Address::Unmapped,
            0x04000000..=0x04FFFFFF => Address::Unmapped, // Game Pak Expansion, never used
            0x05000000..=0x05FFFFFF => Memory::resolve_wram_address(address),
            0x06000000..=0x06FFFFFF => Address::Unmapped, // TODO: Game Pak RAM
            0x07000000..=0x07FFFFFF => Address::Unmapped, // TODO: Game Pak ROM
            _ => unreachable!("Math broke"),
        }
    }

    fn resolve_vip_address(address: usize) -> Address {
        let address = address & 0x0007FFFF;
        match address {
            0x00000000..=0x00077FFF => Address::Mapped(address),
            // The following ranges mirror data from the character tables
            0x00078000..=0x00079FFF => Address::Mapped(address - 0x72000),
            0x0007A000..=0x0007BFFF => Address::Mapped(address - 0x6C000),
            0x0007C000..=0x0007DFFF => Address::Mapped(address - 0x66000),
            0x0007E000..=0x0007FFFF => Address::Mapped(address - 0x60000),
            _ => unreachable!("SCP-033 containment breach"),
        }
    }

    fn resolve_wram_address(address: usize) -> Address {
        Address::Mapped(address & 0x0500FFFF)
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
    fn can_write_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0x00000000, 0x42);
    }

    #[test]
    fn can_read_byte() {
        let mut memory = Memory::new();
        memory.write_byte(0x00000000, 0x42);
        assert_eq!(memory.read_byte(0x00000000), 0x42);
    }

    #[test]
    fn can_read_halfword() {
        let mut memory = Memory::new();
        memory.write_byte(0x00000000, 0x34);
        memory.write_byte(0x00000001, 0x12);
        assert_eq!(memory.read_halfword(0x00000000), 0x1234);
    }

    #[test]
    fn can_read_word() {
        let mut memory = Memory::new();
        memory.write_byte(0x00000000, 0x78);
        memory.write_byte(0x00000001, 0x56);
        memory.write_byte(0x00000002, 0x34);
        memory.write_byte(0x00000003, 0x12);
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
}
