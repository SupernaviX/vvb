use super::storage::Storage;
use anyhow::Result;

pub struct CPU<'a> {
    cycle: u32,
    storage: &'a mut Storage,
}
impl<'a> CPU<'a> {
    pub fn run(storage: &mut Storage, cycles: u32) -> Result<()> {
        let mut cpu = CPU { cycle: 0, storage };
        while cpu.cycle < cycles {
            let instr = cpu.read_pc();
            let opcode = (instr >> 10) & 0x003F;
            match opcode {
                0b101111 => cpu.movhi(instr),
                0b101000 => cpu.movea(instr),
                0b000110 => cpu.jmp(instr),
                _ => return Err(anyhow::anyhow!("Unrecognized opcode {:06b}", opcode)),
            };
        }
        Ok(())
    }

    fn read_pc(&mut self) -> i16 {
        let result = self.storage.read_halfword(self.storage.pc);
        self.storage.pc += 2;
        result
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
    fn parse_format_v_opcode(&mut self, instr: i16) -> (usize, usize, i16) {
        let reg2 = (instr & 0x03E0) >> 5;
        let reg1 = instr & 0x001F;
        let imm = self.read_pc();
        (reg2 as usize, reg1 as usize, imm)
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

    fn movhi(r2: u8, r1: u8, imm: i16) -> Vec<u8> {
        _op_5(0b101111, r2, r1, imm)
    }

    fn movea(r2: u8, r1: u8, imm: i16) -> Vec<u8> {
        _op_5(0b101000, r2, r1, imm)
    }

    fn jmp(r1: u8) -> Vec<u8> {
        _op_1(0b000110, 0, r1)
    }

    fn rom(instructions: &[Vec<u8>]) -> Storage {
        let mut storage = Storage::new();
        storage.load_game_pak_rom(&[0; 256]).unwrap();
        storage.pc = 0x07000000;
        let mut address = storage.pc;
        for instr in instructions {
            for byte in instr {
                storage.write_byte(address, *byte);
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
        CPU::run(&mut storage, 0).unwrap();
        assert_eq!(storage.pc, 0x07000000);
    }

    #[test]
    fn jumps_to_address() {
        let mut storage = rom(&[
            movhi(31, 0, 0x0700),
            movea(31, 31, 0x0420),
            jmp(31),
        ]);
        CPU::run(&mut storage, 5).unwrap();
        assert_eq!(storage.pc, 0x07000420);
    }
}
