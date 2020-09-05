use super::storage::Storage;
use anyhow::Result;

// ECR: exception cause register
const ECR: usize = 4;
// PSW: program status word
const PSW: usize = 5;
// EIPC: exception/interrupt PC
const EIPC: usize = 0;
// EIPSW: exception interrupt PSW
const EIPSW: usize = 1;

// PSW flags and masks
const INTERRUPT_LEVEL: i32 = 0x0000f0000;
const NMI_PENDING_FLAG: i32 = 0x00008000;
const EX_PENDING_FLAG: i32 = 0x00004000;
const ADDRESS_TRAP_ENABLE_FLAG: i32 = 0x00002000;
const INTERRUPT_DISABLE_FLAG: i32 = 0x00001000;
const CARRY_FLAG: i32 = 0x00000008;
const OVERFLOW_FLAG: i32 = 0x00000004;
const SIGN_FLAG: i32 = 0x00000002;
const ZERO_FLAG: i32 = 0x00000001;
const INTERRUPTS_DISABLED_MASK: i32 = INTERRUPT_DISABLE_FLAG | EX_PENDING_FLAG | NMI_PENDING_FLAG;

pub struct CPU {
    cycle: u64,
}
impl CPU {
    pub fn new() -> CPU {
        CPU { cycle: 0 }
    }
    pub fn run(&mut self, storage: &mut Storage, target_cycle: u64) -> Result<CPUProcessingResult> {
        let mut process = CPUProcess::new(self.cycle, storage);
        process.run(target_cycle)?;
        self.cycle = process.cycle;
        Ok(CPUProcessingResult {
            cycle: process.cycle,
            event: process.event,
        })
    }
    pub fn request_interrupt(&mut self, storage: &mut Storage, interrupt: &Interrupt) {
        let mut psw = storage.sys_registers[PSW];

        // if interrupts have been disabled, do nothing
        if (psw & INTERRUPTS_DISABLED_MASK) != 0 {
            return;
        }
        // if the current interrupt is more important, do nothing
        let current_level = ((psw & INTERRUPT_LEVEL) >> 16) as u8;
        if current_level > interrupt.level {
            return;
        }

        let mut ecr = storage.sys_registers[ECR];
        let pc = storage.pc;

        // Save the state from before interrupt handling
        storage.sys_registers[EIPSW] = psw;
        storage.sys_registers[EIPC] = pc as i32;

        // Update the state to process the interrupt
        ecr |= interrupt.code as u32 as i32; // zero-extending
        storage.sys_registers[ECR] = ecr;

        psw |= EX_PENDING_FLAG;
        psw &= !INTERRUPT_LEVEL;
        psw |= (interrupt.level as i32 + 1) << 16;
        psw |= INTERRUPT_DISABLE_FLAG;
        psw &= !ADDRESS_TRAP_ENABLE_FLAG;
        storage.sys_registers[PSW] = psw;

        storage.pc = interrupt.handler;
    }
    pub fn reset(&mut self) {
        self.cycle = 0;
    }
}

pub struct CPUProcessingResult {
    pub cycle: u64,
    pub event: Option<Event>,
}
pub enum Event {
    HardwareWrite { address: usize },
    DisplayControlWrite { address: usize },
    ReturnFromInterrupt,
}

#[derive(Clone, Copy)]
pub struct Interrupt {
    pub code: u16,
    pub level: u8,
    pub handler: usize,
}

pub struct CPUProcess<'a> {
    pub cycle: u64,
    pub event: Option<Event>,
    storage: &'a mut Storage,
}
impl<'a> CPUProcess<'a> {
    pub fn new(cycle: u64, storage: &mut Storage) -> CPUProcess {
        CPUProcess {
            cycle,
            event: None,
            storage,
        }
    }
    pub fn run(&mut self, target_cycle: u64) -> Result<()> {
        while self.cycle < target_cycle && self.event.is_none() {
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
                0b111000 => self.in_b(instr),
                0b111001 => self.in_h(instr),
                0b111011 => self.in_w(instr),

                0b110100 => self.st_b(instr),
                0b110101 => self.st_h(instr),
                0b110111 => self.st_w(instr),
                // OUT.x match ST.x
                0b111100 => self.st_b(instr),
                0b111101 => self.st_h(instr),
                0b111111 => self.st_w(instr),

                0b010001 => self.add_i(instr),
                0b000001 => self.add_r(instr),
                0b101001 => self.addi(instr),
                0b010011 => self.cmp_i(instr),
                0b000011 => self.cmp_r(instr),
                0b000010 => self.sub(instr),
                0b001000 => self.mul(instr),
                0b001010 => self.mulu(instr),
                0b001001 => self.div(instr)?,
                0b001011 => self.divu(instr)?,

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

                0b011110 => self.sei(),
                0b010110 => self.cli(),
                0b011001 => self.reti(),

                _ => {
                    // TODO this should trap
                    return Err(anyhow::anyhow!(
                        "Unrecognized opcode {:06b} at address 0x{:08x}",
                        opcode,
                        self.storage.pc - 2
                    ));
                }
            };
        }
        Ok(())
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
    fn in_b(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage.registers[reg2] = (self.storage.read_byte(address) as i32) & 0x000000ff;
        self.cycle += 5;
    }
    fn in_h(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage.registers[reg2] = (self.storage.read_halfword(address) as i32) & 0x0000ffff;
        self.cycle += 5;
    }
    fn in_w(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.storage.registers[reg2] = self.storage.read_word(address);
        self.cycle += 5;
    }

    fn st_b(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.event = self
            .storage
            .write_byte(address, self.storage.registers[reg2] as i8);
        self.cycle += 4;
    }
    fn st_h(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.event = self
            .storage
            .write_halfword(address, self.storage.registers[reg2] as i16);
        self.cycle += 4;
    }
    fn st_w(&mut self, instr: i16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.storage.registers[reg1] + disp) as usize;
        self.event = self
            .storage
            .write_word(address, self.storage.registers[reg2]);
        self.cycle += 4;
    }

    fn add_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        self.storage.registers[reg2] =
            self._add(self.storage.registers[reg2], self.storage.registers[reg1]);
        self.cycle += 1;
    }
    fn add_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        self.storage.registers[reg2] = self._add(self.storage.registers[reg2], imm);
        self.cycle += 1;
    }
    fn addi(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        self.storage.registers[reg2] = self._add(self.storage.registers[reg1], imm);
        self.cycle += 1;
    }
    fn _add(&mut self, val1: i32, val2: i32) -> i32 {
        let (result, ov) = val1.overflowing_add(val2);
        let s1 = val1 < 0;
        let s2 = val2 < 0;
        let s = result < 0;
        let cy = if s { s1 && s2 } else { s1 || s2 };
        self.update_psw_flags_cy(result == 0, s, ov, cy);
        result
    }

    fn cmp_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        self._subtract(self.storage.registers[reg2], self.storage.registers[reg1]);
        self.cycle += 1;
    }
    fn cmp_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        self._subtract(self.storage.registers[reg2], imm);
        self.cycle += 1;
    }
    fn sub(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        self.storage.registers[reg2] =
            self._subtract(self.storage.registers[reg2], self.storage.registers[reg1]);
        self.cycle += 1;
    }
    fn _subtract(&mut self, val1: i32, val2: i32) -> i32 {
        let (result, ov) = val1.overflowing_sub(val2);
        let s1 = val1 < 0;
        let s2 = val2 < 0;
        let s = result < 0;
        let cy = if s1 { s2 && s } else { s2 || s };
        self.update_psw_flags_cy(result == 0, s, ov, cy);
        result
    }
    fn mul(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let product = self.storage.registers[reg2] as i64 * self.storage.registers[reg1] as i64;

        let hiword = (product >> 32) as i32;
        let loword = product as i32;
        self.storage.registers[30] = hiword;
        self.storage.registers[reg2] = loword;

        let ov = product != loword as i64;
        self.update_psw_flags(loword == 0, loword < 0, ov);
        self.cycle += 13;
    }
    fn mulu(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let product = self.storage.registers[reg2] as u64 * self.storage.registers[reg1] as u64;

        let hiword = (product >> 32) as i32;
        let loword = product as i32;

        self.storage.registers[30] = hiword;
        self.storage.registers[reg2] = loword;
        let ov = product != loword as u64;
        self.update_psw_flags(loword == 0, loword < 0, ov);
        self.cycle += 13;
    }
    fn div(&mut self, instr: i16) -> Result<()> {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let dividend = self.storage.registers[reg2];
        let divisor = self.storage.registers[reg1];
        if divisor == 0 {
            // TODO this should trap
            return Err(anyhow::anyhow!(
                "Divide by zero at 0x{:08x}",
                self.storage.pc - 2
            ));
        } else if dividend == i32::MIN && divisor == -1 {
            self.storage.registers[30] = 0;
            self.storage.registers[reg2] = i32::MIN;
            self.update_psw_flags(false, true, true);
        } else {
            let quotient = dividend / divisor;
            let remainder = dividend % divisor;
            self.storage.registers[30] = remainder;
            self.storage.registers[reg2] = quotient;
            self.update_psw_flags(quotient == 0, quotient < 0, false);
        }
        self.cycle += 38;
        Ok(())
    }
    fn divu(&mut self, instr: i16) -> Result<()> {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let dividend = self.storage.registers[reg2] as u32;
        let divisor = self.storage.registers[reg1] as u32;
        if divisor == 0 {
            // TODO this should trap
            return Err(anyhow::anyhow!(
                "Divide by zero at 0x{:08x}",
                self.storage.pc - 2
            ));
        } else {
            let quotient = (dividend / divisor as u32) as i32;
            let remainder = (dividend % divisor) as i32;
            self.storage.registers[30] = remainder;
            self.storage.registers[reg2] = quotient;
            self.update_psw_flags(quotient == 0, quotient < 0, false);
        }
        self.cycle += 36;
        Ok(())
    }

    fn and(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.storage.registers[reg2] & self.storage.registers[reg1];
        self.storage.registers[reg2] = value;
        self.update_psw_flags(value == 0, value < 0, false);
        self.cycle += 1;
    }
    fn andi(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.storage.registers[reg1] & (imm & 0xffff);
        self.storage.registers[reg2] = value;
        self.update_psw_flags(value == 0, value < 0, false);
        self.cycle += 1;
    }
    fn not(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = !self.storage.registers[reg1];
        self.storage.registers[reg2] = value;
        self.update_psw_flags(value == 0, value < 0, false);
        self.cycle += 1;
    }
    fn or(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.storage.registers[reg2] | self.storage.registers[reg1];
        self.storage.registers[reg2] = value;
        self.update_psw_flags(value == 0, value < 0, false);
        self.cycle += 1;
    }
    fn ori(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.storage.registers[reg1] | (imm & 0xffff);
        self.storage.registers[reg2] = value;
        self.update_psw_flags(value == 0, false, false);
        self.cycle += 1;
    }
    fn xor(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.storage.registers[reg2] ^ self.storage.registers[reg1];
        self.storage.registers[reg2] = value;
        self.update_psw_flags(value == 0, value < 0, false);
        self.cycle += 1;
    }
    fn xori(&mut self, instr: i16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.storage.registers[reg1] ^ (imm & 0xffff);
        self.storage.registers[reg2] = value;
        self.update_psw_flags(value == 0, value < 0, false);
        self.cycle += 1;
    }

    fn sar_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = imm & 0x1f;
        let value = old_value >> shift;
        self.storage.registers[reg2] = value;
        let cy = shift != 0 && (old_value & (1 << (shift - 1)) != 0);
        self.update_psw_flags_cy(value == 0, value < 0, false, cy);
        self.cycle += 1;
    }
    fn sar_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = self.storage.registers[reg1] & 0x1f;
        let value = old_value >> shift;
        self.storage.registers[reg2] = value;
        let cy = shift != 0 && (old_value & (1 << (shift - 1)) != 0);
        self.update_psw_flags_cy(value == 0, value < 0, false, cy);
        self.cycle += 1;
    }
    fn shl_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = imm & 0x1f;
        let value = old_value << shift;
        self.storage.registers[reg2] = value;
        let cy = shift != 0 && (old_value & (i32::MIN >> (shift - 1)) != 0);
        self.update_psw_flags_cy(value == 0, value < 0, false, cy);
        self.cycle += 1;
    }
    fn shl_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = self.storage.registers[reg1] & 0x1f;
        let value = old_value << shift;
        self.storage.registers[reg2] = value;
        let cy = shift != 0 && (old_value & (i32::MIN >> (shift - 1)) != 0);
        self.update_psw_flags_cy(value == 0, value < 0, false, cy);
        self.cycle += 1;
    }
    fn shr_i(&mut self, instr: i16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = imm & 0x1f;
        let value = ((old_value as u32) >> shift as u32) as i32;
        self.storage.registers[reg2] = value;
        let cy = shift != 0 && (old_value & (1 << (shift - 1)) != 0);
        self.update_psw_flags_cy(value == 0, value < 0, false, cy);
        self.cycle += 1;
    }
    fn shr_r(&mut self, instr: i16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.storage.registers[reg2];
        let shift = self.storage.registers[reg1] & 0x1f;
        let value = ((old_value as u32) >> shift as u32) as i32;
        self.storage.registers[reg2] = value;
        let cy = shift != 0 && (old_value & (1 << (shift - 1)) != 0);
        self.update_psw_flags_cy(value == 0, value < 0, false, cy);
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

    fn sei(&mut self) {
        let psw = self.storage.sys_registers[PSW];
        self.storage.sys_registers[PSW] = psw | INTERRUPT_DISABLE_FLAG;
        self.cycle += 12;
    }

    fn cli(&mut self) {
        let psw = self.storage.sys_registers[PSW];
        self.storage.sys_registers[PSW] = psw ^ INTERRUPT_DISABLE_FLAG;
        self.cycle += 12;
    }

    fn reti(&mut self) {
        self.storage.sys_registers[PSW] = self.storage.sys_registers[EIPSW];
        self.storage.pc = self.storage.sys_registers[EIPC] as usize;
        self.event = Some(Event::ReturnFromInterrupt);
        self.cycle += 10;
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

    fn update_psw_flags(&mut self, z: bool, s: bool, ov: bool) {
        let mut psw = self.storage.sys_registers[PSW];
        psw ^= psw & (ZERO_FLAG | SIGN_FLAG | OVERFLOW_FLAG);
        if z {
            psw |= ZERO_FLAG;
        }
        if s {
            psw |= SIGN_FLAG;
        }
        if ov {
            psw |= OVERFLOW_FLAG;
        }
        self.storage.sys_registers[PSW] = psw;
    }
    fn update_psw_flags_cy(&mut self, z: bool, s: bool, ov: bool, cy: bool) {
        let mut psw = self.storage.sys_registers[PSW];
        psw ^= psw & (ZERO_FLAG | SIGN_FLAG | OVERFLOW_FLAG | CARRY_FLAG);
        if z {
            psw |= ZERO_FLAG;
        }
        if s {
            psw |= SIGN_FLAG;
        }
        if ov {
            psw |= OVERFLOW_FLAG;
        }
        if cy {
            psw |= CARRY_FLAG;
        }
        self.storage.sys_registers[PSW] = psw;
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use crate::emulator::cpu::{CPU, PSW, CARRY_FLAG, SIGN_FLAG, OVERFLOW_FLAG, ZERO_FLAG, Interrupt, EX_PENDING_FLAG, INTERRUPT_DISABLE_FLAG, EIPC, EIPSW, NMI_PENDING_FLAG};
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
    fn in_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b111000, r2, r1, disp) }
    fn ld_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110000, r2, r1, disp) }
    fn st_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110100, r2, r1, disp) }
    fn add_i(r2: u8, imm: i8) -> Vec<u8> { _op_2(0b010001, r2, imm) }
    fn addi(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101001, r2, r1, imm) }
    fn cmp_r(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b000011, r2, r1) }
    fn sub(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b000010, r2, r1) }
    fn mul(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001000, r2, r1) }
    fn mulu(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001010, r2, r1) }
    fn div(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001001, r2, r1) }
    fn divu(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001011, r2, r1) }
    fn and(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001101, r2, r1) }
    fn andi(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101101, r2, r1, imm) }
    fn or(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001100, r2, r1) }
    fn ori(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101100, r2, r1, imm) }
    fn xor(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001110, r2, r1) }
    fn xori(r2: u8, r1: u8, imm: i16) -> Vec<u8> { _op_5(0b101110, r2, r1, imm) }
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
    fn reti() -> Vec<u8> { _op_2(0b011001, 0, 0) }

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

    fn add_interrupt_handler(storage: &mut Storage, mut address: usize, instructions: &[Vec<u8>]) {
        for instr in instructions {
            for byte in instr {
                storage.write_byte(address, *byte as i8);
                address += 1;
            }
        }
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
    fn zero_extends_for_loads() {
        let mut storage = rom(&[
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            in_b(31, 30, -16),
        ]);
        storage.write_byte(0x07000032, -2);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 7).unwrap();
        assert_eq!(storage.registers[31], 0x000000fe);
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
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0);
    }

    #[test]
    fn sets_overflow_flag_on_addition_signed_wraparound() {
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
        assert_eq!(storage.sys_registers[PSW] & 0xf, OVERFLOW_FLAG | SIGN_FLAG);
    }

    #[test]
    fn sets_carry_flag_on_addition_unsigned_wraparound() {
        let mut storage = rom(&[
            movea(29, 29, -1),
            addi(31, 29, 1)
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[31], 0);
        assert_eq!(storage.sys_registers[PSW] & 0xf, ZERO_FLAG | CARRY_FLAG);
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
        assert_eq!(storage.sys_registers[PSW] & 0xf, CARRY_FLAG | SIGN_FLAG);
    }

    #[test]
    fn sets_overflow_flag_on_subtraction_signed_wraparound() {
        let mut storage = rom(&[
            // most straightforward way I can find to set a register to i32::MIN
            movhi(31, 0, i16::MIN),
            movea(30, 0, 1),

            // i32::MIN - 1 == i32.MAX
            sub(31, 30),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 3).unwrap();
        assert_eq!(storage.registers[31], i32::MAX);
        assert_eq!(storage.sys_registers[PSW] & 0xf, OVERFLOW_FLAG);
    }

    #[test]
    fn sets_carry_flag_on_subtraction_unsigned_wraparound() {
        let mut storage = rom(&[
            movea(29, 29, 1),
            sub(31, 29),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[31], -1);
        assert_eq!(storage.sys_registers[PSW] & 0xf, SIGN_FLAG | CARRY_FLAG);
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
        assert_eq!(storage.sys_registers[PSW] & 0xf, CARRY_FLAG | SIGN_FLAG);
    }

    #[test]
    fn handles_multiplication() {
        let mut storage = rom(&[
            movea(10, 0, 3),
            movea(11, 0, 6),
            movea(12, 0, -4),
            mulu(11, 10),
            mul(12, 11),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 16).unwrap();
        assert_eq!(storage.registers[11], 18);
        assert_eq!(storage.registers[30], 0);

        cpu.run(&mut storage, 29).unwrap();
        assert_eq!(storage.registers[12], -72);
        assert_eq!(storage.registers[30], -1);
    }

    #[test]
    fn handles_division() {
        let mut storage = rom(&[
            movea(10, 0, -72),
            movea(11, 0, -4),
            movea(12, 0, 4),
            div(10, 11),
            divu(10, 12),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 39).unwrap();
        assert_eq!(storage.registers[10], 18);
        assert_eq!(storage.registers[30], 0);

        cpu.run(&mut storage, 77).unwrap();
        assert_eq!(storage.registers[10], 4);
        assert_eq!(storage.registers[30], 2);
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
        assert_eq!(storage.sys_registers[PSW] & 0xf, CARRY_FLAG | ZERO_FLAG);
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
        assert_eq!(storage.sys_registers[PSW] & 0xf, 0);
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
        assert_eq!(storage.sys_registers[PSW] & 0xf, SIGN_FLAG);
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
    fn andi_0xffff_should_preserve_high_bits() {
        let mut storage = rom(&[
            movea(10, 0, 0x1082),
            andi(11, 10, -1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[11], 0x1082);
    }

    #[test]
    fn ori_0xffff_should_set_to_0xffff() {
        let mut storage = rom(&[
            movea(10, 0, 0x1082),
            ori(11, 10, -1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[11], 0xffff as i32);
    }

    #[test]
    fn xori_0xffff_should_flip_bits() {
        let mut storage = rom(&[
            movea(10, 0, 0x1082),
            xori(11, 10, -1),
        ]);
        let mut cpu = CPU::new();
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[11], 0xef7d as i32);
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

    #[test]
    fn can_request_interrupt() {
        let mut storage = rom(&[
            movea(31, 0, 1),
        ]);
        add_interrupt_handler(&mut storage, 0xfffffe10, &[
            movea(31, 0, 2),
            reti(),
        ]);
        let mut cpu = CPU::new();
        storage.sys_registers[PSW] = 0;

        cpu.request_interrupt(&mut storage, &Interrupt {
            code: 0xfe10,
            level: 1,
            handler: 0xfffffe10,
        });
        assert_eq!(storage.sys_registers[PSW], EX_PENDING_FLAG
            | 0x20000 // interrupt level 1
            | INTERRUPT_DISABLE_FLAG
        );
        assert_eq!(storage.sys_registers[EIPC], 0x07000000);
        assert_eq!(storage.sys_registers[EIPSW], 0);

        cpu.run(&mut storage, 1).unwrap();
        assert_eq!(storage.registers[31], 2);

        // Run another 10 cycles for RETI
        cpu.run(&mut storage, 11).unwrap();
        assert_eq!(storage.pc, 0x07000000);
        cpu.run(&mut storage, 12).unwrap();
        assert_eq!(storage.registers[31], 1);
    }

    #[test]
    fn can_not_request_interrupt_when_disabled() {
        let mut storage = rom(&[
            movea(31, 0, 1),
            movea(31, 0, 2),
            movea(31, 0, 3),
        ]);
        add_interrupt_handler(&mut storage, 0xfffffe10, &[
            movea(31, 0, 9001),
        ]);
        let mut cpu = CPU::new();

        let interrupt = Interrupt {
            code: 0xfe10,
            level: 1,
            handler: 0xfffffe10
        };

        storage.sys_registers[PSW] = INTERRUPT_DISABLE_FLAG;
        cpu.request_interrupt(&mut storage, &interrupt);
        cpu.run(&mut storage, 1).unwrap();
        assert_eq!(storage.registers[31], 1);

        storage.sys_registers[PSW] = EX_PENDING_FLAG;
        cpu.request_interrupt(&mut storage, &interrupt);
        cpu.run(&mut storage, 2).unwrap();
        assert_eq!(storage.registers[31], 2);

        storage.sys_registers[PSW] = NMI_PENDING_FLAG;
        cpu.request_interrupt(&mut storage, &interrupt);
        cpu.run(&mut storage, 3).unwrap();
        assert_eq!(storage.registers[31], 3);
    }

    #[test]
    fn can_not_request_interrupt_when_current_interrupt_takes_priority() {
        let mut storage = rom(&[
            movea(31, 0, 1),
        ]);
        add_interrupt_handler(&mut storage, 0xfffffe10, &[
            movea(31, 0, 2),
        ]);
        add_interrupt_handler(&mut storage, 0xfffffe40, &[
            movea(31, 0, 3),
        ]);
        let mut cpu = CPU::new();
        storage.sys_registers[PSW] = 0;

        let high_priority_interrupt = Interrupt {
            code: 0xfe40,
            level: 4,
            handler: 0xfffffe40,
        };
        let low_priority_interrupt = Interrupt {
            code: 0xfe10,
            level: 1,
            handler: 0xfffffe10,
        };
        cpu.request_interrupt(&mut storage, &high_priority_interrupt);
        storage.sys_registers[PSW] ^= EX_PENDING_FLAG | INTERRUPT_DISABLE_FLAG;
        cpu.request_interrupt(&mut storage, &low_priority_interrupt);

        cpu.run(&mut storage, 1).unwrap();
        assert_eq!(storage.registers[31], 3);
    }
}
