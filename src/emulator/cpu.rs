use super::storage::Storage;
use anyhow::Result;

pub struct CPU {
    cycle: u32,
}
impl CPU {
    pub fn new() -> CPU {
        CPU { cycle: 0 }
    }
    pub fn run(&mut self, storage: &mut Storage, until_cycle: u32) -> Result<()> {
        let cycles = CPUProcess {
            cycle: self.cycle,
            storage,
        }
        .run(until_cycle)?;
        self.cycle = cycles;
        Ok(())
    }
    pub fn reset(&mut self) {
        self.cycle = 0;
    }
}

pub struct CPUProcess<'a> {
    cycle: u32,
    storage: &'a mut Storage,
}
impl<'a> CPUProcess<'a> {
    pub fn run(&mut self, until_cycle: u32) -> Result<u32> {
        while self.cycle < until_cycle {
            let instr = self.read_pc();
            let opcode = (instr >> 10) & 0x003F;
            match opcode {
                0b010000 => self.mov_i(instr),
                0b000000 => self.mov_r(instr),
                0b101111 => self.movhi(instr),
                0b101000 => self.movea(instr),
                0b110000 => self.ld_b(instr),
                0b110001 => self.ld_h(instr),
                0b110011 => self.ld_w(instr),
                0b110100 => self.st_b(instr),
                0b110101 => self.st_h(instr),
                0b110111 => self.st_w(instr),
                0b000110 => self.jmp(instr),
                _ => return Err(anyhow::anyhow!("Unrecognized opcode {:06b}", opcode)),
            };
        }
        Ok(self.cycle)
    }

    fn read_pc(&mut self) -> i16 {
        let result = self.storage.read_halfword(self.storage.pc);
        self.storage.pc += 2;
        result
    }

    fn mov_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        self.storage.registers[reg2] = imm as i32;
        self.cycle += 1;
    }
    fn mov_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_ii_opcode(instr);
        self.storage.registers[reg2] = self.storage.registers[reg1 as usize];
        self.cycle += 1;
    }
    fn movhi(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        self.storage.registers[reg2] = self.storage.registers[reg1] + ((imm as i32) << 16);
        self.cycle += 1;
    }
    fn movea(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        self.storage.registers[reg2] = self.storage.registers[reg1] + (imm as i32);
        self.cycle += 1;
    }

    fn ld_b(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let addr = self.storage.registers[reg1] + disp as i32;
        self.storage.registers[reg2] = self.storage.read_byte(addr as usize) as i32;
        self.cycle += 5;
    }
    fn ld_h(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let addr = self.storage.registers[reg1] + disp as i32;
        self.storage.registers[reg2] = self.storage.read_halfword(addr as usize) as i32;
        self.cycle += 5;
    }
    fn ld_w(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let addr = self.storage.registers[reg1] + disp as i32;
        self.storage.registers[reg2] = self.storage.read_word(addr as usize) as i32;
        self.cycle += 5;
    }

    fn st_b(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let addr = self.storage.registers[reg1] + disp as i32;
        self.storage
            .write_byte(addr as usize, self.storage.registers[reg2] as i8);
        self.cycle += 4;
    }
    fn st_h(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let addr = self.storage.registers[reg1] + disp as i32;
        self.storage
            .write_halfword(addr as usize, self.storage.registers[reg2] as i16);
        self.cycle += 4;
    }
    fn st_w(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let addr = self.storage.registers[reg1] + disp as i32;
        self.storage
            .write_word(addr as usize, self.storage.registers[reg2]);
        self.cycle += 4;
    }

    fn jmp(&mut self, instr: i16) {
        let (_, reg1) = self.parse_format_i_opcode(instr);
        self.storage.pc = self.storage.registers[reg1] as usize;
        self.cycle += 3;
    }

    fn parse_format_i_opcode(&self, instr: i16) -> (usize, usize) {
        let reg2 = (instr & 0x03E0) >> 5;
        let reg1 = instr & 0x001F;
        (reg2 as usize, reg1 as usize)
    }
    fn parse_format_ii_opcode(&self, instr: i16) -> (usize, i16) {
        let reg2 = (instr & 0x03E0) >> 5;
        let imm = instr & 0x001F;
        (reg2 as usize, imm)
    }
    fn parse_format_v_opcode(&mut self, instr: i16) -> (usize, usize, i16) {
        let reg2 = (instr & 0x03E0) >> 5;
        let reg1 = instr & 0x001F;
        let imm = self.read_pc();
        (reg2 as usize, reg1 as usize, imm)
    }
    fn parse_format_vi_opcode(&mut self, instr: i16) -> (usize, usize, i16) {
        let reg2 = (instr & 0x03E0) >> 5;
        let reg1 = instr & 0x001F;
        let disp = self.read_pc();
        (reg2 as usize, reg1 as usize, disp)
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use crate::emulator::cpu::CPU;
    use crate::emulator::storage::Storage;

    fn _op_1(opcode: u8, r2: u8, r1: u8) -> Vec<u8> {
        vec![(r2 << 5) + r1, (opcode << 2) + (r2 >> 3)]
    }

    fn _op_5(opcode: u8, r2: u8, r1: u8, imm: i16) -> Vec<u8> {
        vec![
            (r2 << 5) + r1,
            (opcode << 2) + (r2 >> 3),
            imm as u8,
            (imm >> 8) as u8,
        ]
    }
    fn _op_6(opcode: u8, r2: u8, r1: u8, disp: i16) -> Vec<u8> {
        _op_5(opcode, r2, r1, disp)
    }

    fn movhi(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101111, r2, r1, imm) }
    fn movea(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101000, r2, r1, imm) }
    fn ld_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110000, r2, r1, disp) }
    fn st_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110100, r2, r1, disp) }
    fn jmp(r1: u8) -> Vec<u8> { _op_1(0b000110, 0, r1) }

    fn rom(instructions: &[Vec<u8>]) -> Storage {
        let mut storage = Storage::new();
        storage.load_game_pak_rom(&[0; 256]).unwrap();
        storage.pc = 0x07000000;
        let mut address = storage.pc;
        for instr in instructions {
            for byte in instr {
                storage.write_byte(address, *byte as i8);
                address += 1;
            }
        }
        storage
    }

    #[test]
    fn does_nothing_on_zero_cycles() {
        let mut storage = rom(&[
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 0).unwrap();
        assert_eq!(storage.pc, 0x07000000);
    }

    #[test]
    fn runs_one_cycle_at_a_time() {
        let mut storage = rom(&[
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 1).unwrap();
        assert_eq!(storage.registers[31], 0x07000000);
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[31], 0x07000420);
    }

    #[test]
    fn does_nothing_when_ahead_of_current_cycle() {
        let mut storage = rom(&[
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 1).unwrap();
        assert_eq!(storage.registers[31], 0x07000000);
        cpu.run(&mut storage, 1).unwrap();
        assert_eq!(storage.registers[31], 0x07000000);
    }

    #[test]
    fn jumps_to_address() {
        let mut storage = rom(&[
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 5).unwrap();
        assert_eq!(storage.pc, 0x07000420);
    }

    #[test]
    fn reads_from_memory() {
        let mut storage = rom(&[
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            ld_b(31, 30, 16),
        ]);
        storage.write_byte(0x07000052, 69);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 7).unwrap();
        assert_eq!(storage.registers[31], 69);
    }

    #[test]
    fn sign_extends_for_reads() {
        let mut storage = rom(&[
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            ld_b(31, 30, -16),
        ]);
        storage.write_byte(0x07000032, -2);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 7).unwrap();
        assert_eq!(storage.registers[31], -2);
    }

    #[test]
    fn writes_to_memory() {
        let mut storage = rom(&[
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            movea(31, 0, 0x0069),
            st_b(31, 30, 16),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 7).unwrap();
        assert_eq!(storage.read_byte(0x07000052), 0x69);
    }

    #[test]
    fn sign_extends_for_writes() {
        let mut storage = rom(&[
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            movea(31, 0, -2),
            st_b(31, 30, -16),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 7).unwrap();
        assert_eq!(storage.read_byte(0x07000032), -2);
    }

    #[test]
    fn truncates_during_stores() {
        let mut storage = rom(&[
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            movea(31, 0, 257),
            st_b(31, 30, 16),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 7).unwrap();
        assert_eq!(storage.read_byte(0x07000052), 1);
    }
}
