use anyhow::Result;
use std::convert::TryInto;

enum Address {
    Mapped(usize),
    Unmapped,
}

pub struct Storage {
    pub pc: usize,
    pub registers: [i32; 32],
    pub sys_registers: [i32; 32],
    memory: Vec<u8>,
    rom_mask: usize,
}
impl Storage {
    pub fn new() -> Storage {
        let mut storage = Storage {
            pc: 0xFFFFFFF0,
            registers: [0; 32],
            sys_registers: [0; 32],
            memory: vec![0; 0x07FFFFFF],
            rom_mask: 0,
        };
        storage.init();
        storage
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
        self.pc = 0xfffffff0;
        for reg in self.registers.iter_mut() {
            *reg = 0;
        }
        for sys_reg in 0..self.sys_registers.len() {
            self.sys_registers[sys_reg] = match sys_reg {
                4 => 0x0000fff0,  // ETC
                5 => 0x00008000,  // PSW
                6 => 0x00005346,  // PIR
                7 => 0x000000E0,  // TKCW
                30 => 0x00000004, // it is a mystery
                _ => 0,
            };
        }
        // hold lr + start
        self.write_byte(0x02000010, 0x02);
        self.write_byte(0x02000014, 0x01);
    }

    pub fn write_byte(&mut self, address: usize, value: i8) {
        if let Address::Mapped(resolved) = self.resolve_address(address) {
            self.memory[resolved] = value as u8;
        }
    }

    pub fn write_halfword(&mut self, address: usize, value: i16) {
        if let Address::Mapped(resolved) = self.resolve_address(address) {
            self.memory[resolved..resolved + 2].copy_from_slice(&value.to_le_bytes());
        }
    }

    pub fn write_word(&mut self, address: usize, value: i32) {
        if let Address::Mapped(resolved) = self.resolve_address(address) {
            self.memory[resolved..resolved + 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    pub fn read_byte(&self, address: usize) -> i8 {
        match self.resolve_address(address) {
            Address::Mapped(resolved) => self.memory[resolved] as i8,
            Address::Unmapped => 0,
        }
    }

    pub fn read_halfword(&self, address: usize) -> i16 {
        let address = match self.resolve_address(address) {
            Address::Mapped(resolved) => resolved,
            Address::Unmapped => return 0,
        };
        let bytes: &[u8; 2] = self.memory[address..address + 2].try_into().unwrap();
        i16::from_le_bytes(*bytes)
    }

    pub fn read_word(&self, address: usize) -> i32 {
        let address = match self.resolve_address(address) {
            Address::Mapped(resolved) => resolved,
            Address::Unmapped => return 0,
        };
        let bytes: &[u8; 4] = self.memory[address..address + 4].try_into().unwrap();
        i32::from_le_bytes(*bytes)
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

    fn resolve_hardware_address(&self, address: usize) -> Address {
        Address::Mapped(address & 0x0200003f)
    }

    fn resolve_wram_address(&self, address: usize) -> Address {
        Address::Mapped(address & 0x0500FFFF)
    }

    fn resolve_game_pak_rom_address(&self, address: usize) -> Address {
        Address::Mapped(address & self.rom_mask)
    }
}

#[cfg(test)]
#[allow(overflowing_literals)] // 0xFF is -1, this is not a logical error
mod tests {
    use crate::emulator::storage::Storage;

    #[test]
    fn can_create() {
        Storage::new();
    }

    #[test]
    fn can_read_and_write_byte() {
        let mut storage = Storage::new();
        storage.write_byte(0x00000000, 0x42);
        assert_eq!(storage.read_byte(0x00000000), 0x42);
    }

    #[test]
    fn can_read_and_write_halfword() {
        let mut storage = Storage::new();
        storage.write_halfword(0x00000000, 0x1234);
        assert_eq!(storage.read_byte(0x00000000), 0x34);
        assert_eq!(storage.read_byte(0x00000001), 0x12);
        assert_eq!(storage.read_halfword(0x00000000), 0x1234);
    }

    #[test]
    fn can_read_and_write_word() {
        let mut storage = Storage::new();
        storage.write_word(0x00000000, 0x12345678);
        assert_eq!(storage.read_byte(0x00000000), 0x78);
        assert_eq!(storage.read_byte(0x00000001), 0x56);
        assert_eq!(storage.read_byte(0x00000002), 0x34);
        assert_eq!(storage.read_byte(0x00000003), 0x12);
        assert_eq!(storage.read_word(0x00000000), 0x12345678);
    }

    #[test]
    fn high_addresses_are_mirrored() {
        let mut storage = Storage::new();
        storage.write_byte(0x10000000, 0xFF);
        assert_eq!(storage.read_byte(0x00000000), 0xFF);
    }

    #[test]
    fn frame_buffers_are_mirrored() {
        let mut storage = Storage::new();
        storage.write_byte(0x00000000, 0xFF);
        assert_eq!(storage.read_byte(0x00080000), 0xFF);
        assert_eq!(storage.read_byte(0x00100000), 0xFF);
    }

    #[test]
    fn character_tables_are_mirrored() {
        let mut storage = Storage::new();
        storage.write_byte(0x00006000, 0x01);
        storage.write_byte(0x0000E000, 0x02);
        storage.write_byte(0x00016000, 0x03);
        storage.write_byte(0x0001E000, 0x04);
        assert_eq!(storage.read_byte(0x00078000), 0x01);
        assert_eq!(storage.read_byte(0x0007A000), 0x02);
        assert_eq!(storage.read_byte(0x0007C000), 0x03);
        assert_eq!(storage.read_byte(0x0007E000), 0x04);
    }

    #[test]
    fn wram_is_mirrored() {
        let mut storage = Storage::new();
        storage.write_byte(0x05123456, 0x63);
        assert_eq!(storage.read_byte(0x05F23456), 0x63);
    }

    #[test]
    fn can_load_game_pak_rom() {
        let mut storage = Storage::new();
        storage
            .load_game_pak_rom(&[0x78, 0x56, 0x34, 0x12])
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "ROM size must be a power of two")]
    fn asserts_rom_is_power_of_two() {
        let mut storage = Storage::new();
        storage.load_game_pak_rom(&[0x78, 0x56, 0x34]).unwrap();
    }

    #[test]
    #[should_panic(expected = "ROM size must be <= 16Mb")]
    fn asserts_rom_is_small_enough() {
        let mut storage = Storage::new();
        let too_much_rom = vec![0u8; 0x01000000];
        storage.load_game_pak_rom(too_much_rom.as_slice()).unwrap();
    }

    #[test]
    fn can_read_game_pak_rom() {
        let mut storage = Storage::new();
        storage
            .load_game_pak_rom(&[0x78, 0x56, 0x34, 0x12])
            .unwrap();
        assert_eq!(storage.read_word(0x07000000), 0x12345678);
    }

    #[test]
    fn can_read_game_pak_rom_mirrored_by_size() {
        let mut storage = Storage::new();
        storage
            .load_game_pak_rom(&[0x78, 0x56, 0x34, 0x12, 0x89, 0x57, 0x34, 0x06])
            .unwrap();
        assert_eq!(storage.read_word(0x07000000), 0x12345678);
        assert_eq!(storage.read_word(0x07000004), 0x06345789);
        assert_eq!(storage.read_word(0x07000008), 0x12345678);
    }
}
