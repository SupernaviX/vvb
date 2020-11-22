use super::memory::Memory;
use crate::emulator::cpu::Interrupt;
use std::cell::RefCell;
use std::rc::Rc;

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
    memory: Rc<RefCell<Memory>>,
    next_tick: u64,
    reload_value: u16,
    zero_flag: bool,
    interrupt: Option<Interrupt>,
}
impl Hardware {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Hardware {
        Hardware {
            cycle: 0,
            memory,
            next_tick: u64::MAX,
            reload_value: 0,
            zero_flag: false,
            interrupt: None,
        }
    }

    pub fn init(&mut self) {
        self.cycle = 0;
        self.next_tick = u64::MAX;
        self.reload_value = 0;
        self.zero_flag = false;
        self.interrupt = None;
        let mut memory = self.memory.borrow_mut();
        memory.write_byte(TCR, 0);
        memory.write_halfword(TLR, 0xff);
        memory.write_halfword(THR, 0xff);
    }

    pub fn process_inputs(&self, input_state: u16) {
        // Always set flag 0x02 on the lower register, "real" controllers do
        let sdlr = input_state & 0xff | 0x02;
        let sdhr = (input_state >> 8) & 0xff;
        let mut memory = self.memory.borrow_mut();
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
    pub fn process_event(&mut self, address: usize) {
        if address == THR || address == TLR {
            // A game set the timer, so start counting down from the new value
            self.reload_value = self.read_timer();
            self.compute_next_tick();
        }
        if address == TCR {
            // A game wrote to the timer control register, do what we gotta do
            self.update_timer_settings();
        }
    }

    fn update_timer_settings(&mut self) {
        let tcr = self.memory.borrow().read_byte(TCR);
        if (tcr & T_ENABLED) == 0 {
            // Stop counting down
            self.next_tick = u64::MAX;
            return;
        } else {
            self.compute_next_tick();
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
        self.correct_tcr();
    }

    pub fn run(&mut self, target_cycle: u64) {
        while self.cycle < target_cycle {
            self.cycle = std::cmp::min(target_cycle, self.next_tick);
            if self.cycle == self.next_tick {
                let new_timer_value = self.read_timer() - 1;
                if new_timer_value == 0 {
                    // Timer's going off!

                    // Set the flag that says the timer went off
                    self.zero_flag = true;

                    // Maybe fire the interrupt
                    if (self.memory.borrow().read_byte(TCR) & T_INTERRUPT) != 0 {
                        self.interrupt = Some(Interrupt {
                            code: 0xfe10,
                            level: 2,
                            handler: 0xfffffe10,
                        });
                    }

                    // Reset the timer
                    self.write_timer(self.reload_value);
                    self.compute_next_tick();
                } else {
                    // Keep on ticking
                    self.write_timer(new_timer_value);
                    self.compute_next_tick();
                }
            }
        }
        self.correct_tcr();
    }

    fn read_timer(&self) -> u16 {
        let memory = self.memory.borrow();
        return (memory.read_halfword(THR) as u16) << 8 | memory.read_halfword(TLR) as u16;
    }

    fn write_timer(&self, value: u16) {
        let mut memory = self.memory.borrow_mut();
        memory.write_halfword(THR, value >> 8);
        memory.write_halfword(TLR, value & 0xff);
    }

    fn compute_next_tick(&mut self) {
        let memory = self.memory.borrow();
        let tcr = memory.read_byte(TCR);
        let enabled = (tcr & T_ENABLED) != 0;
        if enabled && self.reload_value != 0 {
            let interval = if (tcr & T_INTERVAL) == 1 { 400 } else { 2000 };
            self.next_tick = self.cycle + interval;
        } else {
            self.next_tick = u64::MAX;
        }
    }

    fn correct_tcr(&self) {
        let mut memory = self.memory.borrow_mut();
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
    use std::cell::RefCell;
    use std::rc::Rc;

    fn set_tcr(hardware: &mut Hardware, memory: &Rc<RefCell<Memory>>, value: u8) {
        memory.borrow_mut().write_byte(TCR, value);
        hardware.process_event(TCR);
    }

    fn set_timer(hardware: &mut Hardware, memory: &Rc<RefCell<Memory>>, value: u16) {
        memory.borrow_mut().write_halfword(THR, value >> 8);
        hardware.process_event(THR);
        memory.borrow_mut().write_halfword(TLR, value & 0xff);
        hardware.process_event(TLR);
    }

    fn get_hardware() -> (Hardware, Rc<RefCell<Memory>>) {
        let memory = Rc::new(RefCell::new(Memory::new()));
        let hardware = Hardware::new(Rc::clone(&memory));
        (hardware, memory)
    }

    #[test]
    fn does_nothing_interesting_when_timer_is_off() {
        let (mut hardware, memory) = get_hardware();

        set_tcr(&mut hardware, &memory, 0);
        assert_eq!(hardware.next_event(), u64::MAX);
        hardware.run(1000000);
        assert_eq!(hardware.next_event(), u64::MAX);
    }

    #[test]
    fn does_nothing_when_timer_is_on_but_time_is_unset() {
        let (mut hardware, memory) = get_hardware();

        set_tcr(&mut hardware, &memory, T_ENABLED);
        assert_eq!(hardware.next_event(), u64::MAX);
        hardware.run(1000000);
        assert_eq!(hardware.next_event(), u64::MAX);
    }

    #[test]
    fn counts_down_when_timer_is_configured_properly() {
        let (mut hardware, memory) = get_hardware();

        set_timer(&mut hardware, &memory, 3);
        assert_eq!(memory.borrow().read_byte(TCR), 0);
        assert_eq!(hardware.read_timer(), 3);
        assert_eq!(hardware.next_event(), u64::MAX);

        set_tcr(&mut hardware, &memory, T_ENABLED);
        assert_eq!(memory.borrow().read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(), 3);
        assert_eq!(hardware.next_event(), 2000);

        hardware.run(1000);
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(memory.borrow().read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(), 3);
        assert_eq!(hardware.next_event(), 2000);

        hardware.run(hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(memory.borrow().read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(), 2);
        assert_eq!(hardware.next_event(), 4000);

        hardware.run(hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(memory.borrow().read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(), 1);
        assert_eq!(hardware.next_event(), 6000);

        hardware.run(hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(
            memory.borrow().read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO
        );
        assert_eq!(hardware.read_timer(), 3);
        assert_eq!(hardware.next_event(), 8000);
    }

    #[test]
    fn handles_interrupt_cycle() {
        let (mut hardware, memory) = get_hardware();

        set_timer(&mut hardware, &memory, 3);
        set_tcr(&mut hardware, &memory, T_ENABLED | T_INTERRUPT);
        assert!(hardware.active_interrupt().is_none());

        // interrupt fires when clock goes off
        hardware.run(6000);
        assert_eq!(
            memory.borrow().read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO | T_INTERRUPT
        );
        assert!(hardware.active_interrupt().is_some());

        // interrupt keeps firing while unacknowledged
        hardware.run(7000);
        assert_eq!(
            memory.borrow().read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO | T_INTERRUPT
        );
        assert!(hardware.active_interrupt().is_some());

        // acknowledge interrupt
        set_tcr(&mut hardware, &memory, T_ENABLED);
        hardware.run(8000);
        assert_eq!(
            memory.borrow().read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO
        );
        assert!(hardware.active_interrupt().is_none());

        // clear zero
        set_tcr(&mut hardware, &memory, T_ENABLED | T_CLEAR_ZERO);
        hardware.run(9000);
        assert_eq!(memory.borrow().read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert!(hardware.active_interrupt().is_none());
    }
}
