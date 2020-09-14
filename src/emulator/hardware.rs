use super::memory::Memory;
use crate::emulator::cpu::Interrupt;

const SDLR: usize = 0x02000010;
const SDHR: usize = 0x02000014;
const TLR: usize = 0x02000018;
const THR: usize = 0x0200001c;
const TCR: usize = 0x02000020;

// TCR bits
const T_INTERVAL: u8 = 0x10;
const T_INTERRUPT: u8 = 0x08;
const T_CLEAR_ZERO: u8 = 0x04;
const T_IS_ZERO: u8 = 0x02;
const T_ENABLED: u8 = 0x01;

pub struct Hardware {
    cycle: u64,
    next_tick: u64,
    reload_value: u16,
    zero_flag: bool,
    interrupt: Option<Interrupt>,
}
impl Hardware {
    pub fn new() -> Hardware {
        Hardware {
            cycle: 0,
            next_tick: u64::MAX,
            reload_value: 0,
            zero_flag: false,
            interrupt: None,
        }
    }

    pub fn init(&mut self, memory: &mut Memory) {
        self.cycle = 0;
        self.next_tick = u64::MAX;
        self.reload_value = 0;
        self.zero_flag = false;
        self.interrupt = None;
        memory.write_byte(TCR, 0);
        memory.write_halfword(TLR, 0xff);
        memory.write_halfword(THR, 0xff);
    }

    pub fn process_inputs(&self, memory: &mut Memory, input_state: u16) {
        // Always set flag 0x02 on the lower register, "real" controllers do
        let sdlr = input_state & 0xff | 0x02;
        let sdhr = (input_state >> 8) & 0xff;
        memory.write_halfword(SDLR, sdlr);
        memory.write_halfword(SDHR, sdhr);
    }

    // When is the next time that this module will do something that affects other modules?
    pub fn next_event(&self) -> u64 {
        self.next_tick
    }

    // Get any unacknowledged interrupt from this module
    pub fn active_interrupt(&self) -> Option<Interrupt> {
        self.interrupt.clone()
    }

    // The CPU has written to somewhere in the hardware address space
    // Update whatever internal state we need to in response
    pub fn process_event(&mut self, memory: &mut Memory, address: usize) {
        if address == THR || address == TLR {
            // A game set the timer, so start counting down from the new value
            self.reload_value = self.read_timer(memory);
            self.compute_next_tick(memory);
        }
        if address == TCR {
            // A game wrote to the timer control register, do what we gotta do
            self.update_timer_settings(memory);
        }
    }

    fn update_timer_settings(&mut self, memory: &mut Memory) {
        let tcr = memory.read_byte(TCR);
        if (tcr & T_ENABLED) == 0 {
            // Stop counting down
            self.next_tick = u64::MAX;
            return;
        } else {
            self.compute_next_tick(memory);
        }
        if (tcr & T_INTERRUPT) == 0 {
            // Interrupt was disabled and/or acknowledged
            self.interrupt = None;
        }
        if (tcr & T_CLEAR_ZERO) != 0 {
            // Interrupt was acknowledged and zero flag was cleared
            self.interrupt = None;
            self.zero_flag = false;
        }
        self.correct_tcr(memory);
    }

    pub fn run(&mut self, memory: &mut Memory, target_cycle: u64) {
        while self.cycle < target_cycle {
            self.cycle = std::cmp::min(target_cycle, self.next_tick);
            if self.cycle == self.next_tick {
                let new_timer_value = self.read_timer(memory) - 1;
                if new_timer_value == 0 {
                    // Timer's going off!

                    // Set the flag that says the timer went off
                    self.zero_flag = true;

                    // Maybe fire the interrupt
                    if (memory.read_byte(TCR) & T_INTERRUPT) != 0 {
                        self.interrupt = Some(Interrupt {
                            code: 0xfe10,
                            level: 2,
                            handler: 0xfffffe10,
                        });
                    }

                    // Reset the timer
                    self.write_timer(memory, self.reload_value);
                    self.compute_next_tick(memory);
                } else {
                    // Keep on ticking
                    self.write_timer(memory, new_timer_value);
                    self.compute_next_tick(memory);
                }
            }
        }
        self.correct_tcr(memory);
    }

    fn read_timer(&self, memory: &Memory) -> u16 {
        return (memory.read_halfword(THR) as u16) << 8 | memory.read_halfword(TLR) as u16;
    }

    fn write_timer(&self, memory: &mut Memory, value: u16) {
        memory.write_halfword(THR, value >> 8);
        memory.write_halfword(TLR, value & 0xff);
    }

    fn compute_next_tick(&mut self, memory: &mut Memory) {
        let tcr = memory.read_byte(TCR);
        let enabled = (tcr & T_ENABLED) != 0;
        if enabled && self.reload_value != 0 {
            let interval = if (tcr & T_INTERVAL) == 1 { 400 } else { 2000 };
            self.next_tick = self.cycle + interval;
        } else {
            self.next_tick = u64::MAX;
        }
    }

    fn correct_tcr(&self, memory: &mut Memory) {
        // update TCR to what the CPU expects
        let mut tcr = memory.read_byte(TCR);
        if self.zero_flag {
            tcr |= T_IS_ZERO;
        } else {
            tcr &= !T_IS_ZERO;
        }
        tcr |= T_CLEAR_ZERO;
        memory.write_byte(TCR, tcr);
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::hardware::{
        Hardware, TCR, THR, TLR, T_CLEAR_ZERO, T_ENABLED, T_INTERRUPT, T_IS_ZERO,
    };
    use crate::emulator::memory::Memory;

    fn set_tcr(hardware: &mut Hardware, memory: &mut Memory, value: u8) {
        memory.write_byte(TCR, value);
        hardware.process_event(memory, TCR);
    }

    fn set_timer(hardware: &mut Hardware, memory: &mut Memory, value: u16) {
        memory.write_halfword(THR, value >> 8);
        hardware.process_event(memory, THR);
        memory.write_halfword(TLR, value & 0xff);
        hardware.process_event(memory, TLR);
    }

    #[test]
    fn does_nothing_interesting_when_timer_is_off() {
        let mut hardware = Hardware::new();
        let mut memory = Memory::new();

        set_tcr(&mut hardware, &mut memory, 0);
        assert_eq!(hardware.next_event(), u64::MAX);
        hardware.run(&mut memory, 1000000);
        assert_eq!(hardware.next_event(), u64::MAX);
    }

    #[test]
    fn does_nothing_when_timer_is_on_but_time_is_unset() {
        let mut hardware = Hardware::new();
        let mut memory = Memory::new();

        set_tcr(&mut hardware, &mut memory, T_ENABLED);
        assert_eq!(hardware.next_event(), u64::MAX);
        hardware.run(&mut memory, 1000000);
        assert_eq!(hardware.next_event(), u64::MAX);
    }

    #[test]
    fn counts_down_when_timer_is_configured_properly() {
        let mut hardware = Hardware::new();
        let mut memory = Memory::new();

        set_timer(&mut hardware, &mut memory, 3);
        assert_eq!(memory.read_byte(TCR), 0);
        assert_eq!(hardware.read_timer(&memory), 3);
        assert_eq!(hardware.next_event(), u64::MAX);

        set_tcr(&mut hardware, &mut memory, T_ENABLED);
        assert_eq!(memory.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(&memory), 3);
        assert_eq!(hardware.next_event(), 2000);

        hardware.run(&mut memory, 1000);
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(memory.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(&memory), 3);
        assert_eq!(hardware.next_event(), 2000);

        hardware.run(&mut memory, hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(memory.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(&memory), 2);
        assert_eq!(hardware.next_event(), 4000);

        hardware.run(&mut memory, hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(memory.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(&memory), 1);
        assert_eq!(hardware.next_event(), 6000);

        hardware.run(&mut memory, hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(memory.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO);
        assert_eq!(hardware.read_timer(&memory), 3);
        assert_eq!(hardware.next_event(), 8000);
    }

    #[test]
    fn handles_interrupt_cycle() {
        let mut hardware = Hardware::new();
        let mut memory = Memory::new();

        set_timer(&mut hardware, &mut memory, 3);
        set_tcr(&mut hardware, &mut memory, T_ENABLED | T_INTERRUPT);
        assert!(hardware.active_interrupt().is_none());

        // interrupt fires when clock goes off
        hardware.run(&mut memory, 6000);
        assert_eq!(
            memory.read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO | T_INTERRUPT
        );
        assert!(hardware.active_interrupt().is_some());

        // interrupt keeps firing while unacknowledged
        hardware.run(&mut memory, 7000);
        assert_eq!(
            memory.read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO | T_INTERRUPT
        );
        assert!(hardware.active_interrupt().is_some());

        // acknowledge interrupt
        set_tcr(&mut hardware, &mut memory, T_ENABLED);
        hardware.run(&mut memory, 8000);
        assert_eq!(memory.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO);
        assert!(hardware.active_interrupt().is_none());

        // clear zero
        set_tcr(&mut hardware, &mut memory, T_ENABLED | T_CLEAR_ZERO);
        hardware.run(&mut memory, 9000);
        assert_eq!(memory.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert!(hardware.active_interrupt().is_none());
    }
}
