use super::memory::Memory;
use anyhow::Result;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

// ECR: exception cause register
const ECR: usize = 4;
const EICC: u32 = 0x0000ffff;
// PSW: program status word
const PSW: usize = 5;
// EIPC: exception/interrupt PC
const EIPC: usize = 0;
// EIPSW: exception interrupt PSW
const EIPSW: usize = 1;

// PSW flags and masks
const INTERRUPT_LEVEL: u32 = 0x000f0000;
const NMI_PENDING_FLAG: u32 = 0x00008000;
const EX_PENDING_FLAG: u32 = 0x00004000;
const ADDRESS_TRAP_ENABLE_FLAG: u32 = 0x00002000;
const INTERRUPT_DISABLE_FLAG: u32 = 0x00001000;
const CARRY_FLAG: u32 = 0x00000008;
const OVERFLOW_FLAG: u32 = 0x00000004;
const SIGN_FLAG: u32 = 0x00000002;
const ZERO_FLAG: u32 = 0x00000001;
const INTERRUPTS_DISABLED_MASK: u32 = INTERRUPT_DISABLE_FLAG | EX_PENDING_FLAG | NMI_PENDING_FLAG;

fn nth_bit_set(value: u32, n: u32) -> bool {
    (value & (1 << n)) != 0
}

#[inline]
fn sign_bit(value: u32) -> bool {
    value & 0x80000000 != 0
}

fn bit_range_mask(start: u32, length: u32) -> u32 {
    ((i32::MIN >> (length - 1)) as u32) >> (32 - length - start)
}

pub struct CPU<THandler: EventHandler> {
    cycle: u64,
    memory: Rc<RefCell<Memory>>,
    handler: THandler,
    bitstring_cycle: u64,
    halted: bool,
    pub pc: usize,
    pub registers: [u32; 32],
    pub sys_registers: [u32; 32],
}
impl<THandler: EventHandler> CPU<THandler> {
    pub fn new(memory: Rc<RefCell<Memory>>, handler: THandler) -> Self {
        let mut cpu = CPU {
            cycle: 0,
            memory,
            handler,
            bitstring_cycle: 0,
            halted: false,
            pc: 0xfffffff0,
            registers: [0; 32],
            sys_registers: [0; 32],
        };
        cpu.init();
        cpu
    }
    pub fn init(&mut self) {
        self.cycle = 0;
        self.halted = false;
        self.bitstring_cycle = 0;
        self.pc = 0xfffffff0;
        for reg in self.registers.iter_mut() {
            *reg = 0;
        }
        for sys_reg in 0..self.sys_registers.len() {
            self.sys_registers[sys_reg] = match sys_reg {
                4 => 0x0000fff0,  // ECR
                5 => 0x00008000,  // PSW
                6 => 0x00008100,  // PIR
                7 => 0x000000E0,  // TKCW
                30 => 0x00000004, // it is a mystery
                _ => 0,
            };
        }
    }

    pub fn run(&mut self, target_cycle: u64) -> Result<CPUProcessingResult> {
        let mut pc = self.pc;
        let mut cycle = self.cycle;
        let mut bitstring_cycle = self.bitstring_cycle;
        let mut halted = self.halted;
        let mut event = None;
        while !halted {
            let mut process = CPUProcess {
                pc,
                cycle,
                bitstring_cycle,
                halted,
                event: None,
                memory: self.memory.borrow_mut(),
                registers: &mut self.registers,
                sys_registers: &mut self.sys_registers,
            };
            process.run(target_cycle)?;
            pc = process.pc;
            cycle = process.cycle;
            bitstring_cycle = process.bitstring_cycle;
            halted = process.halted;
            event = process.event;
            drop(process);

            if let Some(event) = event {
                if self.handler.handle(event, cycle)? {
                    continue;
                }
            }
            break;
        }

        // Make sure that we simulate time passing, even if the CPU is halted.
        // This is safe because as long as the CPU is halted, we know that the next interrupt
        // won't happen until at least target_cycle.
        cycle = cycle.max(target_cycle);

        self.pc = pc;
        self.cycle = cycle;
        self.bitstring_cycle = bitstring_cycle;
        self.halted = halted;
        Ok(CPUProcessingResult { cycle, event })
    }
    pub fn request_interrupt(&mut self, interrupt: &Interrupt) {
        let mut psw = self.sys_registers[PSW];

        // if interrupts have been disabled, do nothing
        if (psw & INTERRUPTS_DISABLED_MASK) != 0 {
            return;
        }
        // if the current interrupt is more important, do nothing
        let current_level = ((psw & INTERRUPT_LEVEL) >> 16) as u8;
        if current_level > interrupt.level {
            return;
        }

        let mut ecr = self.sys_registers[ECR];
        let pc = self.pc;

        // Save the state from before interrupt handling
        self.sys_registers[EIPSW] = psw;
        self.sys_registers[EIPC] = pc as u32;

        // Update the state to process the interrupt
        ecr &= !EICC;
        ecr |= interrupt.code as u32;
        self.sys_registers[ECR] = ecr;

        psw |= EX_PENDING_FLAG;
        psw &= !INTERRUPT_LEVEL;
        psw |= (interrupt.level as u32 + 1) << 16;
        psw |= INTERRUPT_DISABLE_FLAG;
        psw &= !ADDRESS_TRAP_ENABLE_FLAG;
        self.sys_registers[PSW] = psw;

        self.pc = interrupt.handler;
        self.halted = false;
    }
}

pub struct CPUProcessingResult {
    pub cycle: u64,
    pub event: Option<Event>,
}

#[derive(Clone, Copy)]
pub enum Event {
    DisplayControlWrite { address: usize },
    AudioWrite { address: usize },
    HardwareWrite { address: usize },
    ReturnFromInterrupt,
}

pub trait EventHandler {
    fn handle(&mut self, event: Event, cycle: u64) -> Result<bool>;
}

#[derive(Clone, Copy, Debug)]
pub struct Interrupt {
    pub code: u16,
    pub level: u8,
    pub handler: usize,
}

struct CPUProcess<'a> {
    pc: usize,
    cycle: u64,
    bitstring_cycle: u64,
    halted: bool,
    event: Option<Event>,
    memory: RefMut<'a, Memory>,
    registers: &'a mut [u32; 32],
    sys_registers: &'a mut [u32; 32],
}
impl<'a> CPUProcess<'a> {
    pub fn run(&mut self, target_cycle: u64) -> Result<()> {
        while self.cycle < target_cycle && self.event.is_none() && !self.halted {
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

                0b010010 => self.setf(instr),

                0b011010 => self.halt(),

                0b011100 => self.ldsr(instr),
                0b011101 => self.stsr(instr),

                0b011110 => self.sei(),
                0b010110 => self.cli(),
                0b011001 => self.reti(),

                0b011111 => self.bitstring_operation(instr)?,

                0b111110 => {
                    // format 7 opcodes are format 1 with a subopcode suffix
                    let subopcode = (self.read_pc() >> 10) & 0x3f;
                    match subopcode {
                        0b000000 => self.cmpf_s(instr),
                        0b000010 => self.cvt_ws(instr),
                        0b000011 => self.cvt_sw(instr),
                        0b000100 => self.addf_s(instr),
                        0b000101 => self.subf_s(instr),
                        0b000110 => self.mulf_s(instr),
                        0b000111 => self.divf_s(instr),
                        0b001011 => self.trnc_sw(instr),
                        0b001100 => self.mpyhw(instr),
                        0b001010 => self.rev(instr),
                        0b001000 => self.xb(instr),
                        0b001001 => self.xh(instr),
                        _ => {
                            // TODO this should trap
                            return Err(anyhow::anyhow!(
                                "Unrecognized subopcode {:06b} at address 0x{:08x}",
                                subopcode,
                                self.pc - 4
                            ));
                        }
                    }
                }

                _ => {
                    // TODO this should trap
                    return Err(anyhow::anyhow!(
                        "Unrecognized opcode {:06b} at address 0x{:08x}",
                        opcode,
                        self.pc - 2
                    ));
                }
            };
        }
        Ok(())
    }

    fn read_pc(&mut self) -> u16 {
        let result = self.memory.read_halfword(self.pc);
        self.pc += 2;
        result
    }

    fn mov_i(&mut self, instr: u16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        self.set_register(reg2, imm);
        self.cycle += 1;
    }
    fn mov_r(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        self.set_register(reg2, self.registers[reg1]);
        self.cycle += 1;
    }
    fn movhi(&mut self, instr: u16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        self.set_register(reg2, self.registers[reg1] + (imm << 16));
        self.cycle += 1;
    }
    fn movea(&mut self, instr: u16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        self.set_register(reg2, self.registers[reg1] + imm);
        self.cycle += 1;
    }

    fn ld_b(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize;
        self.set_register(reg2, self.memory.read_byte(address) as i8 as u32);
        self.cycle += 5;
    }
    fn ld_h(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize & 0xfffffffe;
        self.set_register(reg2, self.memory.read_halfword(address) as i16 as u32);
        self.cycle += 5;
    }
    fn ld_w(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize & 0xfffffffc;
        self.set_register(reg2, self.memory.read_word(address));
        self.cycle += 5;
    }
    fn in_b(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize;
        self.set_register(reg2, (self.memory.read_byte(address) as u32) & 0x000000ff);
        self.cycle += 5;
    }
    fn in_h(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize & 0xfffffffe;
        self.set_register(
            reg2,
            (self.memory.read_halfword(address) as u32) & 0x0000ffff,
        );
        self.cycle += 5;
    }
    fn in_w(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize & 0xfffffffc;
        self.set_register(reg2, self.memory.read_word(address));
        self.cycle += 5;
    }

    fn st_b(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize;
        self.event = self.memory.write_byte(address, self.registers[reg2] as u8);
        self.cycle += 4;
    }
    fn st_h(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize & 0xfffffffe;
        self.event = self
            .memory
            .write_halfword(address, self.registers[reg2] as u16);
        self.cycle += 4;
    }
    fn st_w(&mut self, instr: u16) {
        let (reg2, reg1, disp) = self.parse_format_vi_opcode(instr);
        let address = (self.registers[reg1] as i32 + disp) as usize & 0xfffffffc;
        self.event = self.memory.write_word(address, self.registers[reg2]);
        self.cycle += 4;
    }

    fn add_r(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self._add(self.registers[reg2], self.registers[reg1]);
        self.set_register(reg2, value);
        self.cycle += 1;
    }
    fn add_i(&mut self, instr: u16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let value = self._add(self.registers[reg2], imm);
        self.set_register(reg2, value);
        self.cycle += 1;
    }
    fn addi(&mut self, instr: u16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self._add(self.registers[reg1], imm);
        self.set_register(reg2, value);
        self.cycle += 1;
    }
    fn _add(&mut self, val1: u32, val2: u32) -> u32 {
        let (result, cy) = val1.overflowing_add(val2);
        let s1 = sign_bit(val1);
        let s2 = sign_bit(val2);
        let s = sign_bit(result);
        let ov = if s { !s1 && !s2 } else { s1 && s2 };
        self.update_psw_flags_cy(result == 0, s, ov, cy);
        result
    }

    fn cmp_r(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        self._subtract(self.registers[reg2], self.registers[reg1]);
        self.cycle += 1;
    }
    fn cmp_i(&mut self, instr: u16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        self._subtract(self.registers[reg2], imm);
        self.cycle += 1;
    }
    fn sub(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self._subtract(self.registers[reg2], self.registers[reg1]);
        self.set_register(reg2, value);
        self.cycle += 1;
    }
    fn _subtract(&mut self, val1: u32, val2: u32) -> u32 {
        let (result, cy) = val1.overflowing_sub(val2);
        let s1 = sign_bit(val1);
        let s2 = sign_bit(val2);
        let s = sign_bit(result);
        let ov = if s1 { !s2 && !s } else { s2 && s };
        self.update_psw_flags_cy(result == 0, s, ov, cy);
        result
    }
    fn mul(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let product = self.registers[reg2] as i32 as i64 * self.registers[reg1] as i32 as i64;

        let hiword = (product >> 32) as u32;
        let loword = product as u32;
        self.set_register(30, hiword);
        self.set_register(reg2, loword);

        let ov = product != loword as i32 as i64;
        self.update_psw_flags(loword == 0, sign_bit(loword), ov);
        self.cycle += 13;
    }
    fn mulu(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let product = self.registers[reg2] as u64 * self.registers[reg1] as u64;

        let hiword = (product >> 32) as u32;
        let loword = product as u32;

        self.set_register(30, hiword);
        self.set_register(reg2, loword);
        let ov = product != loword as i32 as u64;
        self.update_psw_flags(loword == 0, sign_bit(loword), ov);
        self.cycle += 13;
    }
    fn div(&mut self, instr: u16) -> Result<()> {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let dividend = self.registers[reg2] as i32;
        let divisor = self.registers[reg1] as i32;
        if divisor == 0 {
            // TODO this should trap
            return Err(anyhow::anyhow!("Divide by zero at 0x{:08x}", self.pc - 2));
        } else if dividend == i32::MIN && divisor == -1 {
            self.set_register(30, 0);
            self.set_register(reg2, 0x80000000);
            self.update_psw_flags(false, true, true);
        } else {
            let quotient = dividend / divisor;
            let remainder = dividend % divisor;
            self.set_register(30, remainder as u32);
            self.set_register(reg2, quotient as u32);
            self.update_psw_flags(quotient == 0, quotient < 0, false);
        }
        self.cycle += 38;
        Ok(())
    }
    fn divu(&mut self, instr: u16) -> Result<()> {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let dividend = self.registers[reg2];
        let divisor = self.registers[reg1];
        if divisor == 0 {
            // TODO this should trap
            return Err(anyhow::anyhow!("Divide by zero at 0x{:08x}", self.pc - 2));
        } else {
            let quotient = dividend / divisor;
            let remainder = dividend % divisor;
            self.set_register(30, remainder);
            self.set_register(reg2, quotient);
            self.update_psw_flags(quotient == 0, sign_bit(quotient), false);
        }
        self.cycle += 36;
        Ok(())
    }

    fn and(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.registers[reg2] & self.registers[reg1];
        self.set_register(reg2, value);
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 1;
    }
    fn andi(&mut self, instr: u16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.registers[reg1] & (imm & 0xffff);
        self.set_register(reg2, value);
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 1;
    }
    fn not(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = !self.registers[reg1];
        self.set_register(reg2, value);
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 1;
    }
    fn or(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.registers[reg2] | self.registers[reg1];
        self.set_register(reg2, value);
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 1;
    }
    fn ori(&mut self, instr: u16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.registers[reg1] | (imm & 0xffff);
        self.set_register(reg2, value);
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 1;
    }
    fn xor(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.registers[reg2] ^ self.registers[reg1];
        self.set_register(reg2, value);
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 1;
    }
    fn xori(&mut self, instr: u16) {
        let (reg2, reg1, imm) = self.parse_format_v_opcode(instr);
        let value = self.registers[reg1] ^ (imm & 0xffff);
        self.set_register(reg2, value);
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 1;
    }

    fn sar_i(&mut self, instr: u16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.registers[reg2];
        let shift = imm & 0x1f;
        let value = ((old_value as i32) >> shift as i32) as u32;
        self.set_register(reg2, value);
        let cy = shift != 0 && nth_bit_set(old_value, shift - 1);
        self.update_psw_flags_cy(value == 0, sign_bit(value), false, cy);
        self.cycle += 1;
    }
    fn sar_r(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.registers[reg2];
        let shift = self.registers[reg1] & 0x1f;
        let value = ((old_value as i32) >> shift as i32) as u32;
        self.set_register(reg2, value);
        let cy = shift != 0 && nth_bit_set(old_value, shift - 1);
        self.update_psw_flags_cy(value == 0, sign_bit(value), false, cy);
        self.cycle += 1;
    }
    fn shl_i(&mut self, instr: u16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.registers[reg2];
        let shift = imm & 0x1f;
        let value = old_value << shift;
        self.set_register(reg2, value);
        let cy = shift != 0 && nth_bit_set(old_value, 32 - shift);
        self.update_psw_flags_cy(value == 0, sign_bit(value), false, cy);
        self.cycle += 1;
    }
    fn shl_r(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.registers[reg2];
        let shift = self.registers[reg1] & 0x1f;
        let value = old_value << shift;
        self.set_register(reg2, value);
        let cy = shift != 0 && nth_bit_set(old_value, 32 - shift);
        self.update_psw_flags_cy(value == 0, sign_bit(value), false, cy);
        self.cycle += 1;
    }
    fn shr_i(&mut self, instr: u16) {
        let (reg2, imm) = self.parse_format_ii_opcode(instr);
        let old_value = self.registers[reg2];
        let shift = imm & 0x1f;
        let value = old_value >> shift;
        self.set_register(reg2, value);
        let cy = shift != 0 && nth_bit_set(old_value, shift - 1);
        self.update_psw_flags_cy(value == 0, sign_bit(value), false, cy);
        self.cycle += 1;
    }
    fn shr_r(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let old_value = self.registers[reg2];
        let shift = self.registers[reg1] & 0x1f;
        let value = old_value >> shift;
        self.set_register(reg2, value);
        let cy = shift != 0 && nth_bit_set(old_value, shift - 1);
        self.update_psw_flags_cy(value == 0, sign_bit(value), false, cy);
        self.cycle += 1;
    }

    fn bcond(&mut self, instr: u16) {
        let (cond, disp) = self.parse_format_iii_opcode(instr);
        if self._condition(cond) {
            // jump is relative to start of instruction
            self.pc = (self.pc as i32 + disp - 2) as usize & 0xfffffffe;
            self.cycle += 3;
        } else {
            self.cycle += 1;
        }
    }

    fn setf(&mut self, instr: u16) {
        let (reg2, cond) = self.parse_format_ii_opcode(instr);
        let result = if self._condition(cond) { 1 } else { 0 };
        self.registers[reg2] = result;
        self.cycle += 1;
    }

    fn _condition(&self, cond: u32) -> bool {
        let negate = (cond & 0b1000) != 0;
        let psw = self.sys_registers[PSW];
        let cy = (psw & CARRY_FLAG) != 0;
        let ov = (psw & OVERFLOW_FLAG) != 0;
        let s = (psw & SIGN_FLAG) != 0;
        let z = (psw & ZERO_FLAG) != 0;
        let result = match cond & 0b0111 {
            0 => ov,
            1 => cy,
            2 => z,
            3 => cy || z,
            4 => s,
            5 => true,
            6 => ov != s,
            7 => ((ov != s) || z),
            _ => unreachable!("impossible"),
        };
        result != negate
    }

    fn jal(&mut self, instr: u16) {
        let disp = self.parse_format_iv_opcode(instr);
        self.set_register(31, self.pc as u32);
        self.pc = (self.pc as i32 + disp - 4) as usize & 0xfffffffe;
        self.cycle += 3;
    }
    fn jmp(&mut self, instr: u16) {
        let (_, reg1) = self.parse_format_i_opcode(instr);
        self.pc = self.registers[reg1] as usize & 0xfffffffe;
        self.cycle += 3;
    }
    fn jr(&mut self, instr: u16) {
        let disp = self.parse_format_iv_opcode(instr);
        self.pc = (self.pc as i32 + disp - 4) as usize & 0xfffffffe;
        self.cycle += 3;
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn ldsr(&mut self, instr: u16) {
        let (reg2, reg_id) = self.parse_format_ii_opcode(instr);
        let reg_id = (reg_id & 0x1f) as usize;
        let mut value = self.registers[reg2];
        if reg_id == 31 && sign_bit(value) {
            value = -(value as i32) as u32;
        }
        match reg_id {
            4 | 6..=23 | 26..=28 | 30 => (),
            id => self.sys_registers[id] = value,
        }
        self.cycle += 8;
    }
    fn stsr(&mut self, instr: u16) {
        let (reg2, reg_id) = self.parse_format_ii_opcode(instr);
        let reg_id = (reg_id & 0x1f) as usize;
        self.set_register(reg2, self.sys_registers[reg_id]);
        self.cycle += 8;
    }

    fn sei(&mut self) {
        let psw = self.sys_registers[PSW];
        self.sys_registers[PSW] = psw | INTERRUPT_DISABLE_FLAG;
        self.cycle += 12;
    }

    fn cli(&mut self) {
        let psw = self.sys_registers[PSW];
        self.sys_registers[PSW] = psw & !INTERRUPT_DISABLE_FLAG;
        self.cycle += 12;
    }

    fn reti(&mut self) {
        self.sys_registers[PSW] = self.sys_registers[EIPSW];
        self.pc = self.sys_registers[EIPC] as usize;
        self.event = Some(Event::ReturnFromInterrupt);
        self.cycle += 10;
    }

    fn bitstring_operation(&mut self, instr: u16) -> Result<()> {
        let (_, opcode) = self.parse_format_ii_opcode(instr);
        let is_search = (opcode & 0b000010) != 0;
        if is_search {
            return Err(anyhow::anyhow!(
                "Bitwise search 0b{:06b} not supported",
                opcode
            ));
        }
        if self.bitstring_cycle == 0 {
            // clear lower bits of word addresses
            self.registers[30] &= 0xfffffffc;
            self.registers[29] &= 0xfffffffc;
            // clear higher bits of bit offsets
            self.registers[27] &= 0x0000001f;
            self.registers[26] &= 0x0000001f;
        }
        let mut src_address = self.registers[30] as usize;
        let mut dst_address = self.registers[29] as usize;
        let mut length = self.registers[28];
        let mut src_offset = self.registers[27];
        let mut dst_offset = self.registers[26];

        let mut bit_goal = length.min(32 - dst_offset);
        while bit_goal > 0 {
            let src_word = self.memory.read_word(src_address);
            let mut dst_word = self.memory.read_word(dst_address);
            let bits_to_read = bit_goal.min(32 - src_offset);
            let src_bits = src_word & bit_range_mask(src_offset, bits_to_read);
            // make sure src_bits line up with dst_bits
            let src_bits = if src_offset > dst_offset {
                src_bits >> (src_offset - dst_offset)
            } else {
                src_bits << (dst_offset - src_offset)
            };
            let mask = bit_range_mask(dst_offset, bits_to_read);
            match opcode {
                // ANDBSU
                0b01001 => dst_word = (dst_word & !mask) | (dst_word & src_bits),
                // ANDNBSU
                0b01101 => dst_word = (dst_word & !mask) | (dst_word & !src_bits & mask),
                // MOVBSU
                0b01011 => dst_word = (dst_word & !mask) | src_bits,
                // NOTBSU
                0b01111 => dst_word = (dst_word & !mask) | (!src_bits & mask),
                // ORBSU
                0b01000 => dst_word |= src_bits,
                // ORNBSU
                0b01100 => dst_word |= !src_bits & mask,
                // XORBSU
                0b01010 => dst_word ^= src_bits,
                // XORNBSU
                0b01110 => dst_word ^= !src_bits & mask,
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unrecognized bitstring opcode 0b{:05b}",
                        opcode
                    ))
                }
            };
            self.memory.write_word(dst_address, dst_word);
            dst_offset += bits_to_read;
            if dst_offset > 31 {
                dst_offset -= 32;
                dst_address += 4;
            }
            src_offset += bits_to_read;
            if src_offset > 31 {
                src_offset -= 32;
                src_address += 4;
            }

            length -= bits_to_read;
            bit_goal -= bits_to_read;
        }
        self.registers[30] = src_address as u32;
        self.registers[29] = dst_address as u32;
        self.registers[28] = length;
        self.registers[27] = src_offset;
        self.registers[26] = dst_offset;

        // use the worst-case cycle count for simplicity
        if self.bitstring_cycle == 0 {
            self.cycle += 49;
        } else {
            self.cycle += 12;
        }
        if length == 0 {
            // bitstring operation complete!
            self.bitstring_cycle = 0;
        } else {
            // try again next cycle
            self.bitstring_cycle += 1;
            self.pc -= 2;
        }
        Ok(())
    }

    fn cmpf_s(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = f32::from_bits(self.registers[reg2]) - f32::from_bits(self.registers[reg1]);
        self.update_psw_flags_cy(value == 0.0, value < 0.0, false, value < 0.0);
        self.cycle += 10;
    }
    fn cvt_ws(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.registers[reg1] as f32;
        self.registers[reg2] = value.to_bits();
        self.update_psw_flags_cy(value == 0.0, value < 0.0, false, value < 0.0);
        self.cycle += 16;
    }
    fn cvt_sw(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = f32::from_bits(self.registers[reg1]).round() as u32;
        self.registers[reg2] = value;
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 14;
    }
    fn addf_s(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = f32::from_bits(self.registers[reg2]) + f32::from_bits(self.registers[reg1]);
        self.registers[reg2] = value.to_bits();
        self.update_psw_flags_cy(value == 0.0, value < 0.0, false, value < 0.0);
        self.cycle += 28;
    }
    fn subf_s(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = f32::from_bits(self.registers[reg2]) - f32::from_bits(self.registers[reg1]);
        self.registers[reg2] = value.to_bits();
        self.update_psw_flags_cy(value == 0.0, value < 0.0, false, value < 0.0);
        self.cycle += 28;
    }
    fn mulf_s(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = f32::from_bits(self.registers[reg2]) * f32::from_bits(self.registers[reg1]);
        self.registers[reg2] = value.to_bits();
        self.update_psw_flags_cy(value == 0.0, value < 0.0, false, value < 0.0);
        self.cycle += 30;
    }
    fn divf_s(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = f32::from_bits(self.registers[reg2]) / f32::from_bits(self.registers[reg1]);
        self.registers[reg2] = value.to_bits();
        self.update_psw_flags_cy(value == 0.0, value < 0.0, false, value < 0.0);
        self.cycle += 44;
    }
    fn trnc_sw(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = f32::from_bits(self.registers[reg1]).trunc() as u32;
        self.registers[reg2] = value;
        self.update_psw_flags(value == 0, sign_bit(value), false);
        self.cycle += 14;
    }

    fn mpyhw(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let lhs = (self.registers[reg2] as i32)
            .wrapping_shl(15)
            .wrapping_shr(15);
        let rhs = (self.registers[reg1] as i32)
            .wrapping_shl(15)
            .wrapping_shr(15);
        self.set_register(reg2, (lhs * rhs) as u32);
        self.cycle += 9;
    }
    fn rev(&mut self, instr: u16) {
        let (reg2, reg1) = self.parse_format_i_opcode(instr);
        let value = self.registers[reg1].reverse_bits();
        self.set_register(reg2, value);
        self.cycle += 22;
    }
    fn xb(&mut self, instr: u16) {
        let (reg2, _) = self.parse_format_i_opcode(instr);
        let old_value = self.registers[reg2];
        let value = (old_value & 0xffff0000)
            | ((old_value << 8) & 0x0000ff00)
            | ((old_value >> 8) & 0x000000ff);
        self.set_register(reg2, value);
        self.cycle += 6;
    }
    fn xh(&mut self, instr: u16) {
        let (reg2, _) = self.parse_format_i_opcode(instr);
        let old_value = self.registers[reg2];
        let value = (old_value << 16) | ((old_value >> 16) & 0x0000ffff);
        self.set_register(reg2, value);
        self.cycle += 1;
    }

    fn set_register(&mut self, reg: usize, value: u32) {
        if reg != 0 {
            self.registers[reg] = value;
        }
    }

    fn parse_format_i_opcode(&self, instr: u16) -> (usize, usize) {
        let reg2 = (instr & 0x03e0) as usize >> 5;
        let reg1 = (instr & 0x001f) as usize;
        (reg2, reg1)
    }
    fn parse_format_ii_opcode(&self, instr: u16) -> (usize, u32) {
        let reg2 = (instr & 0x03e0) as usize >> 5;
        let imm = (instr as i16 & 0x001f).wrapping_shl(11).wrapping_shr(11) as u32;
        (reg2, imm)
    }
    fn parse_format_iii_opcode(&self, instr: u16) -> (u32, i32) {
        let cond = ((instr >> 9) & 0x0f) as u32;
        let disp = (instr as i16 & 0x01ff).wrapping_shl(7).wrapping_shr(7) as i32;
        (cond, disp)
    }
    fn parse_format_iv_opcode(&mut self, instr: u16) -> i32 {
        let mut disp: i32 = (instr as i32).wrapping_shl(22).wrapping_shr(6);
        disp |= self.read_pc() as i32;
        disp
    }
    fn parse_format_v_opcode(&mut self, instr: u16) -> (usize, usize, u32) {
        let reg2 = (instr & 0x03e0) as usize >> 5;
        let reg1 = (instr & 0x001f) as usize;
        let imm = self.read_pc() as i16 as u32;
        (reg2, reg1, imm)
    }
    fn parse_format_vi_opcode(&mut self, instr: u16) -> (usize, usize, i32) {
        let reg2 = (instr & 0x03e0) as usize >> 5;
        let reg1 = (instr & 0x001f) as usize;
        let disp = self.read_pc() as i16 as i32;
        (reg2, reg1, disp)
    }

    fn update_psw_flags(&mut self, z: bool, s: bool, ov: bool) {
        let mut psw = self.sys_registers[PSW];
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
        self.sys_registers[PSW] = psw;
    }
    fn update_psw_flags_cy(&mut self, z: bool, s: bool, ov: bool, cy: bool) {
        let mut psw = self.sys_registers[PSW];
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
        self.sys_registers[PSW] = psw;
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use crate::emulator::cpu::{CPU, PSW, CARRY_FLAG, SIGN_FLAG, OVERFLOW_FLAG, ZERO_FLAG, Interrupt, EX_PENDING_FLAG, INTERRUPT_DISABLE_FLAG, EIPC, EIPSW, NMI_PENDING_FLAG, EventHandler, Event};
    use crate::emulator::memory::Memory;
    use anyhow::Result;
    use std::cell::{RefCell};
    use std::rc::Rc;

    fn _op_1(opcode: u8, r2: u8, r1: u8) -> Vec<u8> {
        vec![(r2 << 5) | r1, (opcode << 2) | (r2 >> 3)]
    }
    fn _op_2(opcode: u8, r2: u8, imm: u8) -> Vec<u8> {
        vec![(r2 << 5) | (imm & 0x1f), (opcode << 2) | (r2 >> 3)]
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
    fn _op_5(opcode: u8, r2: u8, r1: u8, imm: u16) -> Vec<u8> {
        vec![
            (r2 << 5) | r1,
            (opcode << 2) | (r2 >> 3),
            imm as u8,
            (imm >> 8) as u8,
        ]
    }
    fn _op_6(opcode: u8, r2: u8, r1: u8, disp: i16) -> Vec<u8> {
        _op_5(opcode, r2, r1, disp as u16)
    }
    fn _op_7(opcode: u8, r2: u8, r1: u8, subopcode: u8) -> Vec<u8> {
        vec![
            (r2 << 5) | r1,
            (opcode << 2) | (r2 >> 3),
            0,
            subopcode << 2,
        ]
    }

    fn mov_r(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b000000, r2, r1) }
    fn movhi(r2: u8, r1: u8, imm: u16) -> Vec<u8> { _op_5(0b101111, r2, r1, imm) }
    fn movea(r2: u8, r1: u8, imm: u16) -> Vec<u8> { _op_5(0b101000, r2, r1, imm) }
    fn in_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b111000, r2, r1, disp) }
    fn ld_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110000, r2, r1, disp) }
    fn ld_h(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110001, r2, r1, disp) }
    fn ld_w(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110011, r2, r1, disp) }
    fn st_b(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110100, r2, r1, disp) }
    fn st_h(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110101, r2, r1, disp) }
    fn st_w(r2: u8, r1: u8, disp: i16) -> Vec<u8> { _op_6(0b110111, r2, r1, disp) }
    fn add_i(r2: u8, imm: u8) -> Vec<u8> { _op_2(0b010001, r2, imm) }
    fn addi(r2: u8, r1: u8, imm: u16) -> Vec<u8> { _op_5(0b101001, r2, r1, imm) }
    fn cmp_r(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b000011, r2, r1) }
    fn sub(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b000010, r2, r1) }
    fn mul(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001000, r2, r1) }
    fn mulu(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001010, r2, r1) }
    fn div(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001001, r2, r1) }
    fn divu(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001011, r2, r1) }
    fn and(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001101, r2, r1) }
    fn andi(r2: u8, r1: u8, imm: u16) -> Vec<u8> { _op_5(0b101101, r2, r1, imm) }
    fn or(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001100, r2, r1) }
    fn ori(r2: u8, r1: u8, imm: u16) -> Vec<u8> { _op_5(0b101100, r2, r1, imm) }
    fn xor(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001110, r2, r1) }
    fn xori(r2: u8, r1: u8, imm: u16) -> Vec<u8> { _op_5(0b101110, r2, r1, imm) }
    fn not(r2: u8, r1: u8) -> Vec<u8> { _op_1(0b001111, r2, r1) }
    fn sar_i(r2: u8, imm: u8) -> Vec<u8> { _op_2(0b010111, r2, imm) }
    fn shl_i(r2: u8, imm: u8) -> Vec<u8> { _op_2(0b010100, r2, imm) }
    fn shr_i(r2: u8, imm: u8) -> Vec<u8> { _op_2(0b010101, r2, imm) }
    fn bcond(cond: u8, disp: i16) -> Vec<u8> { _op_3(0b100, cond, disp) }
    fn setf(r2: u8, cond: u8) -> Vec<u8> { _op_2(0b010010, r2, cond) }
    fn jal(disp: i32) -> Vec<u8> { _op_4(0b101011, disp) }
    fn jmp(r1: u8) -> Vec<u8> { _op_1(0b000110, 0, r1) }
    fn jr(disp: i32) -> Vec<u8> { _op_4(0b101010, disp) }
    fn halt() -> Vec<u8> { _op_2(0b011010, 0, 0) }
    fn ldsr(r2: u8, reg_id: u8) -> Vec<u8> { _op_2(0b011100, r2, reg_id) }
    fn stsr(r2: u8, reg_id: u8) -> Vec<u8> { _op_2(0b011101, r2, reg_id) }
    fn orbsu() -> Vec<u8> { _op_2(0b011111, 0, 0b01000) }
    fn cvt_ws(r2: u8, r1: u8) -> Vec<u8> { _op_7(0b111110, r2, r1, 0b000010) }
    fn cvt_sw(r2: u8, r1: u8) -> Vec<u8> { _op_7(0b111110, r2, r1, 0b000011) }
    fn addf_s(r2: u8, r1: u8) -> Vec<u8> { _op_7(0b111110, r2, r1, 0b000100) }
    fn divf_s(r2: u8, r1: u8) -> Vec<u8> { _op_7(0b111110, r2, r1, 0b000111) }
    fn mpyhw(r2: u8, r1: u8) -> Vec<u8> { _op_7(0b111110, r2, r1, 0b001100) }
    fn rev(r2: u8, r1: u8) -> Vec<u8> { _op_7(0b111110, r2, r1, 0b001010) }
    fn xb(r2: u8) -> Vec<u8> { _op_7(0b111110, r2, 0, 0b001000) }
    fn xh(r2: u8) -> Vec<u8> { _op_7(0b111110, r2, 0, 0b001001) }
    fn reti() -> Vec<u8> { _op_2(0b011001, 0, 0) }

    struct NoopEventHandler;
    impl EventHandler for NoopEventHandler {
        fn handle(&mut self, _event: Event, _cycle: u64) -> Result<bool> {
            Ok(false)
        }
    }

    fn rom(instructions: Vec<Vec<u8>>) -> (CPU<NoopEventHandler>, Rc<RefCell<Memory>>) {
        let memory = Rc::new(RefCell::new(Memory::new()));
        let handler = NoopEventHandler;
        let mut cpu = CPU::new(Rc::clone(&memory), handler);
        cpu.pc = 0x07000000;

        {
            let mut memory = memory.as_ref().borrow_mut();
            memory.load_game_pak(&[0; 256], &[]).unwrap();
            let mut address = cpu.pc;
            for instr in instructions {
                for byte in instr {
                    memory.write_byte(address, byte);
                    address += 1;
                }
            }
        }

        (cpu, memory)
    }

    fn add_interrupt_handler(memory: &mut Memory, mut address: usize, instructions: Vec<Vec<u8>>) {
        for instr in instructions {
            for byte in instr {
                memory.write_byte(address, byte);
                address += 1;
            }
        }
    }

    #[test]
    fn does_nothing_on_zero_cycles() {
        let (mut cpu, _memory) = rom(vec![
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        cpu.run(0).unwrap();
        assert_eq!(cpu.pc, 0x07000000);
    }

    #[test]
    fn runs_one_cycle_at_a_time() {
        let (mut cpu, _memory) = rom(vec![
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        assert_eq!(cpu.pc, 0x07000000);
        cpu.run(1).unwrap();
        assert_eq!(cpu.pc, 0x07000004);
        assert_eq!(cpu.registers[31], 0x07000000);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[31], 0x07000420);
    }

    #[test]
    fn cannot_overwrite_zero() {
        let (mut cpu, _memory) = rom(vec![
            movhi(0, 0, 0x0700),
        ]);
        cpu.run(1).unwrap();
        assert_eq!(cpu.registers[0], 0);
    }

    #[test]
    fn does_nothing_when_ahead_of_current_cycle() {
        let (mut cpu, _memory) = rom(vec![
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        cpu.run(1).unwrap();
        assert_eq!(cpu.registers[31], 0x07000000);
        cpu.run(1).unwrap();
        assert_eq!(cpu.registers[31], 0x07000000);
    }

    #[test]
    fn jumps_to_address() {
        let (mut cpu, _memory) = rom(vec![
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        cpu.run(5).unwrap();
        assert_eq!(cpu.pc, 0x07000420);
    }

    #[test]
    fn reads_from_memory() {
        let (mut cpu, memory) = rom(vec![
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            ld_b(31, 30, 16),
        ]);
        memory.borrow_mut().write_byte(0x07000052, 69);
        cpu.run(7).unwrap();
        assert_eq!(cpu.registers[31], 69);
    }

    #[test]
    fn sign_extends_for_reads() {
        let (mut cpu, memory) = rom(vec![
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            ld_b(31, 30, -16),
        ]);
        memory.borrow_mut().write_byte(0x07000032, 0xfe);
        cpu.run(7).unwrap();
        assert_eq!(cpu.registers[31] as i32, -2);
    }

    #[test]
    fn zero_extends_for_loads() {
        let (mut cpu, memory) = rom(vec![
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            in_b(31, 30, -16),
        ]);
        memory.borrow_mut().write_byte(0x07000032, 0xfe);
        cpu.run(7).unwrap();
        assert_eq!(cpu.registers[31], 0x000000fe);
    }

    #[test]
    fn masks_lower_bits_of_addresses_for_multibyte_reads() {
        let (mut cpu, memory) = rom(vec![
            movhi(10, 0, 0x0500),
            ld_h(11, 10, 1),
            ld_w(12, 10, 2),
        ]);
        memory.borrow_mut().write_word(0x05000000, 0x12345678);
        cpu.run(11).unwrap();
        assert_eq!(cpu.registers[11], 0x5678);
        assert_eq!(cpu.registers[12], 0x12345678);
    }

    #[test]
    fn writes_to_memory() {
        let (mut cpu, memory) = rom(vec![
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            movea(31, 0, 0x0069),
            st_b(31, 30, 16),
        ]);
        cpu.run(7).unwrap();
        assert_eq!(memory.borrow().read_byte(0x07000052), 0x69);
    }

    #[test]
    fn sign_extends_for_writes() {
        let (mut cpu, memory) = rom(vec![
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            movea(31, 0, -2i16 as u16),
            st_b(31, 30, -16),
        ]);
        cpu.run(7).unwrap();
        assert_eq!(memory.borrow().read_byte(0x07000032) as i8, -2);
    }

    #[test]
    fn masks_lower_bits_of_addresses_for_multibyte_writes() {
        let (mut cpu, memory) = rom(vec![
            movhi(10, 0, 0x0500),
            movhi(11, 0, 0x1234),
            movea(11, 11, 0x5678),
            st_h(11, 10, 1),
            st_w(11, 10, 10),
        ]);
        memory.borrow_mut().write_word(0x05000000, 0x12345678);
        cpu.run(13).unwrap();
        assert_eq!(memory.borrow().read_halfword(0x05000000), 0x5678);
        assert_eq!(memory.borrow().read_word(0x05000008), 0x12345678);
    }

    #[test]
    fn truncates_during_stores() {
        let (mut cpu, memory) = rom(vec![
            movhi(30, 0, 0x0700),
            movea(30, 30, 0x0042),
            movea(31, 0, 257),
            st_b(31, 30, 16),
        ]);
        cpu.run(7).unwrap();
        assert_eq!(memory.borrow().read_byte(0x07000052), 1);
    }

    #[test]
    fn does_addition() {
        let (mut cpu, _memory) = rom(vec![
            movea(29, 0, 4),
            addi(31, 29, 5)
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[31], 9);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, 0);
    }

    #[test]
    fn sets_overflow_flag_on_addition_signed_wraparound() {
        let (mut cpu, _memory) = rom(vec![
            // most straightforward way I can find to set a register to i32::MAX
            movhi(29, 0, 0x0001),
            add_i(29, 0xff),
            movhi(29, 29, 0x7fff),

            // i32::MAX + 1 == i32.min
            addi(31, 29, 1),
        ]);
        cpu.run(4).unwrap();
        assert_eq!(cpu.registers[29] as i32, i32::MAX);
        assert_eq!(cpu.registers[31] as i32, i32::MIN);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, OVERFLOW_FLAG | SIGN_FLAG);
    }

    #[test]
    fn sets_carry_flag_on_addition_unsigned_wraparound() {
        let (mut cpu, _memory) = rom(vec![
            movea(29, 0, 0xffff),
            addi(31, 29, 1)
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[31], 0);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, ZERO_FLAG | CARRY_FLAG);
    }

    #[test]
    fn does_subtraction() {
        let (mut cpu, _memory) = rom(vec![
            movea(31, 0, 4),
            movea(30, 0, 5),
            sub(31,30),
        ]);
        cpu.run(3).unwrap();
        assert_eq!(cpu.registers[31] as i32, -1);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, CARRY_FLAG | SIGN_FLAG);
    }

    #[test]
    fn sets_overflow_flag_on_subtraction_signed_wraparound() {
        let (mut cpu, _memory) = rom(vec![
            // most straightforward way I can find to set a register to i32::MIN
            movhi(31, 0, 0x8000),
            movea(30, 0, 1),

            // i32::MIN - 1 == i32.MAX
            sub(31, 30),
        ]);
        cpu.run(3).unwrap();
        assert_eq!(cpu.registers[31] as i32, i32::MAX);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, OVERFLOW_FLAG);
    }

    #[test]
    fn sets_carry_flag_on_subtraction_unsigned_wraparound() {
        let (mut cpu, _memory) = rom(vec![
            movea(29, 0, 1),
            sub(31, 29),
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[31], u32::MAX);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, SIGN_FLAG | CARRY_FLAG);
    }

    #[test]
    fn does_cmp() {
        let (mut cpu, _memory) = rom(vec![
            movea(31, 0, 4),
            movea(30, 0, 5),
            cmp_r(31,30),
        ]);
        cpu.run(3).unwrap();
        assert_eq!(cpu.registers[31], 4);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, CARRY_FLAG | SIGN_FLAG);
    }

    #[test]
    fn handles_multiplication() {
        let (mut cpu, _memory) = rom(vec![
            movea(10, 0, 3),
            movea(11, 0, 6),
            movea(12, 0, -4i16 as u16),
            mulu(11, 10),
            mul(12, 11),
        ]);
        cpu.run(16).unwrap();
        assert_eq!(cpu.registers[11], 18);
        assert_eq!(cpu.registers[30], 0);

        cpu.run(29).unwrap();
        assert_eq!(cpu.registers[12] as i32, -72);
        assert_eq!(cpu.registers[30] as i32, -1);
    }

    #[test]
    fn handles_division() {
        let (mut cpu, _memory) = rom(vec![
            movea(10, 0, -72i16 as u16),
            movea(11, 0, -4i16 as u16),
            movea(12, 0, 4),
            div(10, 11),
            divu(10, 12),
        ]);
        cpu.run(39).unwrap();
        assert_eq!(cpu.registers[10], 18);
        assert_eq!(cpu.registers[30], 0);

        cpu.run(77).unwrap();
        assert_eq!(cpu.registers[10], 4);
        assert_eq!(cpu.registers[30], 2);
    }

    #[test]
    fn handles_bcond_true() {
        let (mut cpu, _memory) = rom(vec![
            movea(31, 0, 4),
            movea(30, 0, 5),
            cmp_r(31, 30),
            bcond(3, 6),
            movea(1, 0, 1),
            movea(2, 0, 1),
        ]);
        cpu.run(7).unwrap();
        assert_eq!(cpu.registers[1], 0);
        assert_eq!(cpu.registers[2], 1);
    }

    #[test]
    fn handles_bcond_false() {
        let (mut cpu, _memory) = rom(vec![
            movea(31, 0, 4),
            movea(30, 0, 5),
            cmp_r(31, 30),
            bcond(11, 6),
            movea(1, 0, 1),
            movea(2, 0, 1),
        ]);
        cpu.run(5).unwrap();
        assert_eq!(cpu.registers[1], 1);
        assert_eq!(cpu.registers[2], 0);
    }

    #[test]
    fn handles_setf_true() {
        let (mut cpu, _memory) = rom(vec![
            movea(31, 0, 4),
            movea(30, 0, 5),
            cmp_r(31, 30),
            setf(1, 6),
        ]);
        cpu.run(4).unwrap();
        assert_eq!(cpu.registers[1], 1);
    }

    #[test]
    fn handles_setf_false() {
        let (mut cpu, _memory) = rom(vec![
            movea(31, 0, 4),
            movea(30, 0, 5),
            cmp_r(31, 30),
            setf(1, 11),
        ]);
        cpu.run(4).unwrap();
        assert_eq!(cpu.registers[1], 0);
    }

    #[test]
    fn can_jump_relative() {
        let (mut cpu, _memory) = rom(vec![
            jr(0x123456),
        ]);
        cpu.run(3).unwrap();
        assert_eq!(cpu.pc, 0x07123456);
    }

    #[test]
    fn can_jump_and_link() {
        let (mut cpu, _memory) = rom(vec![
            jal(0x123456),
        ]);
        cpu.run(3).unwrap();
        assert_eq!(cpu.registers[31], 0x07000004);
        assert_eq!(cpu.pc, 0x07123456);
    }

    #[test]
    fn can_shl_with_carry() {
        let (mut cpu, _memory) = rom(vec![
            movhi(31, 0, 0x8000),
            shl_i(31, 1),
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[31], 0);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, CARRY_FLAG | ZERO_FLAG);
    }

    #[test]
    fn can_shr_with_zero_filling() {
        let (mut cpu, _memory) = rom(vec![
            movhi(31, 0, 0x8000),
            shr_i(31, 1),
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[31], 0x40000000);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, 0);
    }

    #[test]
    fn can_shr_with_sign_extension() {
        let (mut cpu, _memory) = rom(vec![
            movhi(31, 0, 0x8000),
            sar_i(31, 1),
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[31], 0xc0000000);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, SIGN_FLAG);
    }

    #[test]
    fn can_run_logic() {
        let (mut cpu, _memory) = rom(vec![
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
        cpu.run(9).unwrap();
        assert_eq!(cpu.registers[30], 0xfffff0f0);
        assert_eq!(cpu.registers[29], 0xffffffff);
        assert_eq!(cpu.registers[28], 0x0f0f);
        assert_eq!(cpu.registers[27], 0x0f0f);
    }

    #[test]
    fn andi_0xffff_should_preserve_high_bits() {
        let (mut cpu, _memory) = rom(vec![
            movea(10, 0, 0x1082),
            andi(11, 10, 0xffff),
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[11], 0x1082);
    }

    #[test]
    fn ori_0xffff_should_set_to_0xffff() {
        let (mut cpu, _memory) = rom(vec![
            movea(10, 0, 0x1082),
            ori(11, 10, 0xffff),
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[11], 0xffff);
    }

    #[test]
    fn xori_0xffff_should_flip_bits() {
        let (mut cpu, _memory) = rom(vec![
            movea(10, 0, 0x1082),
            xori(11, 10, 0xffff),
        ]);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[11], 0xef7d);
    }

    #[test]
    fn can_ldsr_and_stsr() {
        let (mut cpu, _memory) = rom(vec![
            movea(31, 0, 0x0040),
            ldsr(31, 5),
            stsr(30, 5),
        ]);
        cpu.run(17).unwrap();
        assert_eq!(cpu.sys_registers[5], 0x00000040);
        assert_eq!(cpu.registers[30], 0x00000040);
    }

    #[test]
    fn can_run_bitstring_operations() {
        let (mut cpu, memory) = rom(vec![
            movhi(10, 0, 0x0500),
            movhi(11, 0, 0x0500),
            movea(11, 11, 0x1000),
            movhi(12, 0, 0x7777),
            movea(12, 12, 0x7777),
            movhi(13, 0, 0x5555),
            movea(13, 13, 0x5555),
            st_w(12, 10, 0),
            st_w(12, 10, 4),
            st_w(13, 11, 0),
            st_w(13, 11, 4),
            mov_r(30, 10),
            mov_r(29, 11),
            movea(28, 0, 30),
            movea(27, 0, 26),
            movea(26, 0, 20),
            orbsu(),
        ]);

        let setup_cycles = 28;
        let first_cycle = setup_cycles + 49;
        let final_cycle = first_cycle + 12;
        let bitstring_op_pc = 0x0700003c;

        cpu.run(setup_cycles).unwrap();
        assert_eq!(cpu.registers[10], 0x05000000);
        assert_eq!(cpu.registers[11], 0x05001000);
        assert_eq!(cpu.registers[12], 0x77777777);
        assert_eq!(cpu.registers[13], 0x55555555);
        assert_eq!(memory.borrow().read_word(0x05000000), 0x77777777);
        assert_eq!(memory.borrow().read_word(0x05001000), 0x55555555);

        // First cycle, instruction shouldn't be done
        cpu.run(first_cycle).unwrap();
        assert_eq!(cpu.pc, bitstring_op_pc);
        assert_eq!(cpu.registers[30], 0x05000004);
        assert_eq!(cpu.registers[29], 0x05001004);
        assert_eq!(cpu.registers[28], 18);
        assert_eq!(cpu.registers[27], 6);
        assert_eq!(cpu.registers[26], 0);
        assert_eq!(memory.borrow().read_word(0x05001000), 0xddd55555);
        assert_eq!(memory.borrow().read_word(0x05001004), 0x55555555);

        // NOW we should be done
        cpu.run(final_cycle).unwrap();
        assert_eq!(cpu.pc, bitstring_op_pc + 2);
        assert_eq!(cpu.registers[30], 0x05000004);
        assert_eq!(cpu.registers[29], 0x05001004);
        assert_eq!(cpu.registers[28], 0);
        assert_eq!(cpu.registers[27], 24);
        assert_eq!(cpu.registers[26], 18);
        assert_eq!(memory.borrow().read_word(0x05001000), 0xddd55555);
        assert_eq!(memory.borrow().read_word(0x05001004), 0x5555dddd);
    }

    #[test]
    fn can_do_float_things() {
        let (mut cpu, _memory) = rom(vec![
            movea(10, 0, 3),
            cvt_ws(11, 10),
            movea(10, 0, 14),
            cvt_ws(12, 10),
            addf_s(12, 11),
            cvt_sw(13, 12),
            divf_s(12, 11),
            cvt_sw(14, 12),
        ]);
        cpu.run(134).unwrap();
        assert_eq!(f32::from_bits(cpu.registers[11]), 3.0);
        assert_eq!(f32::from_bits(cpu.registers[12]), 17.0 / 3.0);
        assert_eq!(cpu.registers[13], 17);
        assert_eq!(cpu.registers[14], 6);
    }

    #[test]
    fn can_run_mpyhw() {
        let (mut cpu, _memory) = rom(vec![
            movea(10, 0, 9),
            movea(11, 0, 6),
            mpyhw(10, 11),
        ]);
        cpu.run(11).unwrap();
        assert_eq!(cpu.registers[10], 54);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, 0);
    }

    #[test]
    fn can_mpyhw_negative_numbers() {
        let (mut cpu, _memory) = rom(vec![
            movea(10, 0, -9i16 as u16),
            movea(11, 0, 6),
            mpyhw(10, 11),
        ]);
        cpu.run(11).unwrap();
        assert_eq!(cpu.registers[10] as i32, -54);
        assert_eq!(cpu.sys_registers[PSW] & 0xf, 0);
    }

    #[test]
    fn can_run_extended_opcodes() {
        let (mut cpu, _memory) = rom(vec![
            movhi(10, 0, 0x1234),
            movea(10, 10, 0x5678),
            mov_r(11, 10),
            mov_r(12, 10),
            rev(10, 10),
            xb(11),
            xh(12),
        ]);
        cpu.run(33).unwrap();
        assert_eq!(cpu.registers[10], 0x1e6a2c48);
        assert_eq!(cpu.registers[11], 0x12347856);
        assert_eq!(cpu.registers[12], 0x56781234);
    }

    #[test]
    fn can_request_interrupt() {
        let (mut cpu, memory) = rom(vec![
            movea(31, 0, 1),
        ]);
        add_interrupt_handler(&mut memory.borrow_mut(), 0xfffffe10, vec![
            movea(31, 0, 2),
            reti(),
        ]);
        cpu.sys_registers[PSW] = 0;

        cpu.request_interrupt(&Interrupt {
            code: 0xfe10,
            level: 1,
            handler: 0xfffffe10,
        });
        assert_eq!(cpu.sys_registers[PSW], EX_PENDING_FLAG
            | 0x20000 // interrupt level 1
            | INTERRUPT_DISABLE_FLAG
        );
        assert_eq!(cpu.sys_registers[EIPC], 0x07000000);
        assert_eq!(cpu.sys_registers[EIPSW], 0);

        cpu.run(1).unwrap();
        assert_eq!(cpu.registers[31], 2);

        // Run another 10 cycles for RETI
        cpu.run(11).unwrap();
        assert_eq!(cpu.pc, 0x07000000);
        cpu.run(12).unwrap();
        assert_eq!(cpu.registers[31], 1);
    }

    #[test]
    fn can_not_request_interrupt_when_disabled() {
        let (mut cpu, memory) = rom(vec![
            movea(31, 0, 1),
            movea(31, 0, 2),
            movea(31, 0, 3),
        ]);
        add_interrupt_handler(&mut memory.borrow_mut(), 0xfffffe10, vec![
            movea(31, 0, 9001),
        ]);

        let interrupt = Interrupt {
            code: 0xfe10,
            level: 1,
            handler: 0xfffffe10
        };

        cpu.sys_registers[PSW] = INTERRUPT_DISABLE_FLAG;
        cpu.request_interrupt(&interrupt);
        cpu.run(1).unwrap();
        assert_eq!(cpu.registers[31], 1);

        cpu.sys_registers[PSW] = EX_PENDING_FLAG;
        cpu.request_interrupt(&interrupt);
        cpu.run(2).unwrap();
        assert_eq!(cpu.registers[31], 2);

        cpu.sys_registers[PSW] = NMI_PENDING_FLAG;
        cpu.request_interrupt(&interrupt);
        cpu.run(3).unwrap();
        assert_eq!(cpu.registers[31], 3);
    }

    #[test]
    fn can_not_request_interrupt_when_current_interrupt_takes_priority() {
        let (mut cpu, memory) = rom(vec![
            movea(31, 0, 1),
        ]);
        add_interrupt_handler(&mut memory.borrow_mut(), 0xfffffe10, vec![
            movea(31, 0, 2),
        ]);
        add_interrupt_handler(&mut memory.borrow_mut(), 0xfffffe40, vec![
            movea(31, 0, 3),
        ]);
        cpu.sys_registers[PSW] = 0;

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
        cpu.request_interrupt(&high_priority_interrupt);
        cpu.sys_registers[PSW] ^= EX_PENDING_FLAG | INTERRUPT_DISABLE_FLAG;
        cpu.request_interrupt(&low_priority_interrupt);

        cpu.run(1).unwrap();
        assert_eq!(cpu.registers[31], 3);
    }

    #[test]
    fn can_halt() {
        let (mut cpu, _memory) = rom(vec![
            movea(31, 0, 1),
            halt(),
            movea(31, 0, 2),
        ]);

        let res = cpu.run(1000000).unwrap();
        assert_eq!(res.cycle, 1000000);
        assert_eq!(cpu.registers[31], 1);

        let res = cpu.run(2000000).unwrap();
        assert_eq!(res.cycle, 2000000);
        assert_eq!(cpu.registers[31], 1);
    }

    #[test]
    fn can_unhalt_during_interrupt() {
        let (mut cpu, memory) = rom(vec![
            movea(31, 0, 1),
            halt(),
            movea(31, 0, 3),
        ]);
        add_interrupt_handler(&mut memory.borrow_mut(), 0xfffffe40, vec![
            movea(31, 0, 2),
            reti(),
        ]);
        cpu.sys_registers[PSW] = 0;

        let res = cpu.run(50).unwrap();
        assert_eq!(res.cycle, 50);
        assert_eq!(cpu.registers[31], 1);

        cpu.request_interrupt(&Interrupt {
            code: 0xfe40,
            level: 4,
            handler: 0xfffffe40,
        });

        let res = cpu.run(61).unwrap();
        assert_eq!(res.cycle, 61);
        assert_eq!(cpu.registers[31], 2);

        cpu.run(62).unwrap();
        assert_eq!(cpu.registers[31], 3);
    }
}
