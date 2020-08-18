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

const PSW: usize = 5;

pub struct CPUProcess<'a> {
    cycle: u32,
    storage: &'a mut Storage,
}
impl<'a> CPUProcess<'a> {
    pub fn run(&mut self, until_cycle: u32) -> Result<u32> {
        while self.cycle < until_cycle {
            let instr = self.read_pc();
            let opcode = (instr >> 10) & 0x003F;
            if (instr as u16) & 0xe000 == 0x8000 {
                self.bcond(instr);
                continue;
            }
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

                0b010001 => self.add_i(instr),
                0b000001 => self.add_r(instr),
                0b101001 => self.addi(instr),
                0b010011 => self.cmp_i(instr),
                0b000011 => self.cmp_r(instr),
                0b000010 => self.sub(instr),

                0b001101 => self.and(instr),
                0b101101 => self.andi(instr),
                0b001111 => self.not(instr),
                0b001100 => self.or(instr),
                0b101100 => self.ori(instr),
                0b001110 => self.xor(instr),
                0b101110 => self.xori(instr),

                0b010111 => self.sar_i(instr),
                0b000111 => self.sar_r(instr),
                0b010100 => self.shl_i(instr),
                0b000100 => self.shl_r(instr),
                0b010101 => self.shr_i(instr),
                0b000101 => self.shr_r(instr),

                0b101011 => self.jal(instr),
                0b000110 => self.jmp(instr),
                0b101010 => self.jr(instr),

                0b011100 => self.ldsr(instr),
                0b011101 => self.stsr(instr),

                _ => {
                    return Err(anyhow::anyhow!(
                        "Unrecognized opcode {:06b} at address 0x{:08x}",
                        opcode,
                        self.storage.pc - 2
                    ))
                }
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
        self.storage.registers[reg2] = imm;
        self.cycle += 1;
    }
    fn mov_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_ii_opcode(instr);
        self.storage.registers[reg2] = self.storage.registers[(reg1 & 0x1f) as usize];
        self.cycle += 1;
    }
    fn movhi(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        self.storage.registers[reg2] = self.storage.registers[reg1] + (imm << 16);
        self.cycle += 1;
    }
    fn movea(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        self.storage.registers[reg2] = self.storage.registers[reg1] + imm;
        self.cycle += 1;
    }

    fn ld_b(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage.registers[reg2] = self.storage.read_byte(address) as i32;
        self.cycle += 5;
    }
    fn ld_h(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage.registers[reg2] = self.storage.read_halfword(address) as i32;
        self.cycle += 5;
    }
    fn ld_w(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage.registers[reg2] = self.storage.read_word(address);
        self.cycle += 5;
    }

    fn st_b(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage
            .write_byte(address, self.storage.registers[reg2] as i8);
        self.cycle += 4;
    }
    fn st_h(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage
            .write_halfword(address, self.storage.registers[reg2] as i16);
        self.cycle += 4;
    }
    fn st_w(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage
            .write_word(address, self.storage.registers[reg2]);
        self.cycle += 4;
    }

    fn add_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let (value, ov) = old_value.overflowing_add(self.storage.registers[reg1]);
        self.storage.registers[reg2] = value;
        self.update_status_flags(value == 0, value < 0, ov, (value < 0) != (old_value < 0));
        self.cycle += 1;
    }
    fn add_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let (value, ov) = old_value.overflowing_add(imm);
        self.storage.registers[reg2] = value;
        self.update_status_flags(value == 0, value < 0, ov, (value < 0) != (old_value < 0));
        self.cycle += 1;
    }
    fn addi(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let (value, ov) = self.storage.registers[reg1].overflowing_add(imm);
        self.storage.registers[reg2] = value;
        self.update_status_flags(value == 0, value < 0, ov, (value < 0) != (old_value < 0));
        self.cycle += 1;
    }
    fn cmp_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let (value, ov) = old_value.overflowing_sub(self.storage.registers[reg1]);
        self.update_status_flags(value == 0, value < 0, ov, (value < 0) != (old_value < 0));
        self.cycle += 1;
    }
    fn cmp_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let (value, ov) = old_value.overflowing_sub(imm);
        self.update_status_flags(value == 0, value < 0, ov, (value < 0) != (old_value < 0));
        self.cycle += 1;
    }
    fn sub(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let (value, ov) = old_value.overflowing_sub(self.storage.registers[reg1]);
        self.storage.registers[reg2] = value;
        self.update_status_flags(value == 0, value < 0, ov, (value < 0) != (old_value < 0));
        self.cycle += 1;
    }

    fn and(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.storage.registers[reg2] & self.storage.registers[reg1];
        self.storage.registers[reg2] = value;
        self.update_logic_status_flags(value == 0, value < 0);
        self.cycle += 1;
    }
    fn andi(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.storage.registers[reg1] & (imm & 0x00ff);
        self.storage.registers[reg2] = value;
        self.update_logic_status_flags(value == 0, value < 0);
        self.cycle += 1;
    }
    fn not(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = !self.storage.registers[reg1];
        self.storage.registers[reg2] = value;
        self.update_logic_status_flags(value == 0, value < 0);
        self.cycle += 1;
    }
    fn or(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.storage.registers[reg2] | self.storage.registers[reg1];
        self.storage.registers[reg2] = value;
        self.update_logic_status_flags(value == 0, value < 0);
        self.cycle += 1;
    }
    fn ori(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.storage.registers[reg1] | (imm & 0x00ff);
        self.storage.registers[reg2] = value;
        self.update_logic_status_flags(value == 0, false);
        self.cycle += 1;
    }
    fn xor(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.storage.registers[reg2] ^ self.storage.registers[reg1];
        self.storage.registers[reg2] = value;
        self.update_logic_status_flags(value == 0, value < 0);
        self.cycle += 1;
    }
    fn xori(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.storage.registers[reg1] ^ (imm & 0x00ff);
        self.storage.registers[reg2] = value;
        self.update_logic_status_flags(value == 0, value < 0);
        self.cycle += 1;
    }

    fn sar_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = imm & 0x1f;
        let value = old_value >> shift;
        self.storage.registers[reg2] = value;
        let cv = shift != 0 && (old_value & (1 << (shift - 1)) != 0);
        self.update_status_flags(value == 0, value < 0, false, cv);
        self.cycle += 1;
    }
    fn sar_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = self.storage.registers[reg1] & 0x1f;
        let value = old_value >> shift;
        self.storage.registers[reg2] = value;
        let cv = shift != 0 && (old_value & (1 << (shift - 1)) != 0);
        self.update_status_flags(value == 0, value < 0, false, cv);
        self.cycle += 1;
    }
    fn shl_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = imm & 0x1f;
        let value = old_value << shift;
        self.storage.registers[reg2] = value;
        let cv = shift != 0 && (old_value & (i32::MIN >> (shift - 1)) != 0);
        self.update_status_flags(value == 0, value < 0, false, cv);
        self.cycle += 1;
    }
    fn shl_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = self.storage.registers[reg1] & 0x1f;
        let value = old_value << shift;
        self.storage.registers[reg2] = value;
        let cv = shift != 0 && (old_value & (i32::MIN >> (shift - 1)) != 0);
        self.update_status_flags(value == 0, value < 0, false, cv);
        self.cycle += 1;
    }
    fn shr_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = imm & 0x1f;
        let value = ((old_value as u32) >> shift as u32) as i32;
        self.storage.registers[reg2] = value;
        let cv = shift != 0 && (old_value & (1 << (shift - 1)) != 0);
        self.update_status_flags(value == 0, value < 0, false, cv);
        self.cycle += 1;
    }
    fn shr_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = self.storage.registers[reg1] & 0x1f;
        let value = ((old_value as u32) >> shift as u32) as i32;
        self.storage.registers[reg2] = value;
        let cv = shift != 0 && (old_value & (1 << (shift - 1)) != 0);
        self.update_status_flags(value == 0, value < 0, false, cv);
        self.cycle += 1;
    }

    fn bcond(&mut self, instr: i16) {
        let (negate, cond, disp) = self.parse_format_iii_opcode(instr);
        let psw = self.storage.sys_registers[PSW];
        let cy = (psw & 0x8) != 0;
        let ov = (psw & 0x4) != 0;
        let s = (psw & 0x2) != 0;
        let z = (psw & 0x1) != 0;
        let mut result = match cond {
            0 => ov,
            1 => cy,
            2 => z,
            3 => cy || z,
            4 => s,
            5 => true,
            6 => ov || s,
            7 => ((ov != s) || z),
            _ => unreachable!("impossible"),
        };
        if negate {
            result = !result;
        }
        if result {
            // jump is relative to start of instruction
            self.storage.pc = (self.storage.pc as i32 + disp - 2) as usize;
            self.cycle += 3;
        } else {
            self.cycle += 1;
        }
    }

    fn jal(&mut self, instr: i16) {
        let disp = self.parse_format_iv_opcode(instr);
        self.storage.registers[31] = self.storage.pc as i32;
        self.storage.pc = (self.storage.pc as i32 + disp - 4) as usize;
        self.cycle += 3;
    }
    fn jmp(&mut self, instr: i16) {
        let (_, reg1) = self.parse_format_i_opcode(instr);
        self.storage.pc = self.storage.registers[reg1] as usize;
        self.cycle += 3;
    }
    fn jr(&mut self, instr: i16) {
        let disp = self.parse_format_iv_opcode(instr);
        self.storage.pc = (self.storage.pc as i32 + disp - 4) as usize;
        self.cycle += 3;
    }

    fn ldsr(&mut self, instr: i16) {
        let (reg2, reg_id) = self.parse_format_ii_opcode(instr);
        let reg_id = (reg_id & 0x1f) as usize;
        let mut value = self.storage.registers[reg2];
        if reg_id == 31 && value < 0 {
            value = -value;
        }
        match reg_id {
            4 | 6..=23 | 26..=28 | 30 => (),
            id => self.storage.sys_registers[id] = value,
        }
        self.cycle += 8;
    }
    fn stsr(&mut self, instr: i16) {
        let (reg2, reg_id) = self.parse_format_ii_opcode(instr);
        let reg_id = (reg_id & 0x1f) as usize;
        self.storage.registers[reg2] = self.storage.sys_registers[reg_id];
        self.cycle += 8;
    }

    fn parse_format_i_opcode(&self, instr: i16) -> (usize, usize) {
        let reg2 = (instr & 0x03E0) as usize >> 5;
        let reg1 = (instr & 0x001F) as usize;
        (reg2, reg1)
    }
    fn parse_format_ii_opcode(&self, instr: i16) -> (usize, i32) {
        let reg2 = (instr & 0x03E0) as usize >> 5;
        let imm = (instr & 0x001F).wrapping_shl(11).wrapping_shr(11) as i32;
        (reg2, imm)
    }
    fn parse_format_iii_opcode(&self, instr: i16) -> (bool, u8, i32) {
        let negate = (instr & 0x1000) != 0;
        let cond = ((instr >> 9) & 0x07) as u8;
        let disp = (instr & 0x01ff).wrapping_shl(7).wrapping_shr(7) as i32;
        (negate, cond, disp)
    }
    fn parse_format_iv_opcode(&mut self, instr: i16) -> i32 {
        let mut disp: i32 = (instr as i32).wrapping_shl(24).wrapping_shr(8);
        disp |= self.read_pc() as i32;
        disp
    }
    fn parse_format_v_opcode(&mut self, instr: i16) -> (usize, usize, i32) {
        let reg2 = (instr & 0x03E0) as usize >> 5;
        let reg1 = (instr & 0x001F) as usize;
        let imm = self.read_pc() as i32;
        (reg2, reg1, imm)
    }
    fn parse_format_vi_opcode(&mut self, instr: i16) -> (usize, usize, i32) {
        let reg2 = (instr & 0x03E0) as usize >> 5;
        let reg1 = (instr & 0x001F) as usize;
        let disp = self.read_pc() as i32;
        (reg2, reg1, disp)
    }

    fn update_logic_status_flags(&mut self, z: bool, s: bool) {
        let mut psw = self.storage.sys_registers[PSW];
        psw ^= psw & 0x00000007;
        if z {
            psw |= 1;
        }
        if s {
            psw |= 2;
        }
        self.storage.sys_registers[PSW] = psw;
    }
    fn update_status_flags(&mut self, z: bool, s: bool, ov: bool, cy: bool) {
        let mut psw = self.storage.sys_registers[PSW];
        psw ^= psw & 0x0000000f;
        if z {
            psw |= 1;
        }
        if s {
            psw |= 2;
        }
        if ov {
            psw |= 4;
        }
        if cy {
            psw |= 8;
        }
        self.storage.sys_registers[PSW] = psw;
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use crate::emulator::cpu::{CPU, PSW};
    use crate::emulator::storage::Storage;

    fn _op_1(opcode: u8, r2: u8, r1: u8) -> Vec<u8> {
        vec![(r2 << 5) | r1, (opcode << 2) | (r2 >> 3)]
    }
    fn _op_2(opcode: u8, r2: u8, imm: i8) -> Vec<u8> {
        vec![(r2 << 5) | ((imm as u8) & 0x1f), (opcode << 2) | (r2 >> 3)]
    }
    fn _op_3(opcode: u8, cond: u8, disp: i16) -> Vec<u8> {
        vec![
            (disp & 0xff) as u8,
            (opcode << 5) | (cond << 1) | if disp < 0 { 1 } else { 0 },
        ]
    }
    fn _op_4(opcode: u8, disp: i32) -> Vec<u8> {
        vec![
            ((disp & 0x00ff0000) >> 16) as u8,
            (opcode << 2) | ((disp & 0x03000000) > 24) as u8,
            (disp & 0x000000ff) as u8,
            ((disp & 0x0000ff00) >> 8) as u8,
        ]
    }
    fn _op_5(opcode: u8, r2: u8, r1: u8, imm: i16) -> Vec<u8> {
        vec![
            (r2 << 5) | r1,
            (opcode << 2) | (r2 >> 3),
            imm as u8,
            (imm >> 8) as u8,
        ]
    }
    fn _op_6(opcode: u8, r2: u8, r1: u8, disp: i16) -> Vec<u8> {
        _op_5(opcode, r2, r1, disp)
    }

    fn mov_r(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b000000, r2, r1) }
    fn movhi(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101111, r2, r1, imm) }
    fn movea(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101000, r2, r1, imm) }
    fn ld_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110000, r2, r1, disp) }
    fn st_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110100, r2, r1, disp) }
    fn add_i(r2: u8, imm: i8) -> Vec<u8> { _op_2(0b010001, r2, imm) }
    fn addi(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101001, r2, r1, imm) }
    fn cmp_r(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b000011, r2, r1) }
    fn sub(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b000010, r2, r1) }
    fn and(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001101, r2, r1) }
    fn or(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001100, r2, r1) }
    fn xor(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001110, r2, r1) }
    fn not(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001111, r2, r1) }
    fn sar_i(r2: u8, imm: i8) -> Vec<u8> { _op_2(0b010111, r2, imm) }
    fn shl_i(r2: u8, imm: i8) -> Vec<u8> { _op_2(0b010100, r2, imm) }
    fn shr_i(r2: u8, imm: i8) -> Vec<u8> { _op_2(0b010101, r2, imm) }
    fn bcond(cond: u8, disp: i16) -> Vec<u8> { _op_3(0b100, cond, disp) }
    fn jal(disp: i32) -> Vec<u8> { _op_4(0b101011, disp) }
    fn jmp(r1: u8) -> Vec<u8> { _op_1(0b000110, 0, r1) }
    fn jr(disp: i32) -> Vec<u8> { _op_4(0b101010, disp) }
    fn ldsr(r2: u8, reg_id: u8) -> Vec<u8> { _op_2(0b011100, r2, reg_id as i8) }
    fn stsr(r2: u8, reg_id: u8) -> Vec<u8> { _op_2(0b011101, r2, reg_id as i8) }

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

    #[test]
    fn does_addition() {
        let mut storage = rom(&[
            movea(29, 0, 4),
            addi(31, 29, 5)
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[31], 9);
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0b0000);
    }

    #[test]
    fn does_subtraction() {
        let mut storage = rom(&[
            movea(31, 0, 4),
            movea(30, 0, 5),
            sub(31,30),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 3).unwrap();
        assert_eq!(storage.registers[31], -1);
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0b1010);
    }

    #[test]
    fn does_cmp() {
        let mut storage = rom(&[
            movea(31, 0, 4),
            movea(30, 0, 5),
            cmp_r(31,30),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 3).unwrap();
        assert_eq!(storage.registers[31], 4);
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0b1010);
    }

    #[test]
    fn handles_overflow() {
        let mut storage = rom(&[
            // most straightforward way I can find to set a register to i32::MAX
            movhi(29, 0, 0x0001),
            add_i(29, -1),
            movhi(29, 29, 0x7fff),

            // i32::MAX + 1 == i32.min
            addi(31, 29, 1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 4).unwrap();
        assert_eq!(storage.registers[29], i32::MAX);
        assert_eq!(storage.registers[31], i32::MIN);
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0b1110);
    }

    #[test]
    fn handles_bcond_true() {
        let mut storage = rom(&[
            movea(31, 0, 4),
            movea(30, 0, 5),
            cmp_r(31, 30),
            bcond(3, 6),
            movea(1, 0, 1),
            movea(2, 0, 1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 7).unwrap();
        assert_eq!(storage.registers[1], 0);
        assert_eq!(storage.registers[2], 1);
    }

    #[test]
    fn handles_bcond_false() {
        let mut storage = rom(&[
            movea(31, 0, 4),
            movea(30, 0, 5),
            cmp_r(31, 30),
            bcond(11, 6),
            movea(1, 0, 1),
            movea(2, 0, 1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 5).unwrap();
        assert_eq!(storage.registers[1], 1);
        assert_eq!(storage.registers[2], 0);
    }

    #[test]
    fn can_jump_relative() {
        let mut storage = rom(&[
            jr(0x123456),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 3).unwrap();
        assert_eq!(storage.pc, 0x07123456);
    }

    #[test]
    fn can_jump_and_link() {
        let mut storage = rom(&[
            jal(0x123456),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 3).unwrap();
        assert_eq!(storage.registers[31], 0x07000004);
        assert_eq!(storage.pc, 0x07123456);
    }

    #[test]
    fn can_shl_with_carry() {
        let mut storage = rom(&[
            movhi(31, 0, i16::MIN),
            shl_i(31, 1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[31], 0);
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0b1001);
    }

    #[test]
    fn can_shr_with_zero_filling() {
        let mut storage = rom(&[
            movhi(31, 0, i16::MIN),
            shr_i(31, 1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[31], 0x40000000);
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0b0000);
    }

    #[test]
    fn can_shr_with_sign_extension() {
        let mut storage = rom(&[
            movhi(31, 0, i16::MIN),
            sar_i(31, 1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[31], i32::MIN >> 1);
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0b0010);
    }

    #[test]
    fn can_run_logic() {
        let mut storage = rom(&[
            movea(31, 0, 0x0f0f),

            mov_r(30, 31),
            not(30, 30),

            mov_r(29, 31),
            or(29, 30),

            mov_r(28, 31),
            and(28, 29),

            mov_r(27, 29),
            xor(27, 30),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 9).unwrap();
        assert_eq!(storage.registers[30], 0xfffff0f0u32 as i32);
        assert_eq!(storage.registers[29], -1);
        assert_eq!(storage.registers[28], 0x0f0f);
        assert_eq!(storage.registers[27], 0x0f0f);
    }

    #[test]
    fn can_ldsr_and_stsr() {
        let mut storage = rom(&[
            movea(31, 0, 0x0040),
            ldsr(31, 5),
            stsr(30, 5),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 17).unwrap();
        assert_eq!(storage.sys_registers[5], 0x00000040);
        assert_eq!(storage.registers[30], 0x00000040);
    }
}
