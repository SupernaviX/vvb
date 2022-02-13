use super::memory::Memory;
use crate::emulator::cpu::Exception;
use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

const SDLR: usize = 0x02000010;
const SDHR: usize = 0x02000014;
const TLR: usize = 0x02000018;
const THR: usize = 0x0200001c;
const TCR: usize = 0x02000020;
const SCR: usize = 0x02000028;

// TCR bits
const T_INTERVAL: u8 = 0x10;
const T_INTERRUPT: u8 = 0x08;
const T_CLEAR_ZERO: u8 = 0x04;
const T_IS_ZERO: u8 = 0x02;
const T_ENABLED: u8 = 0x01;

// SCR bits
const S_HW_READ: u8 = 0x04;
const S_HW_STAT: u8 = 0x02;
const S_HW_ABORT: u8 = 0x01;

const HARDWARE_READ_CYCLES: u64 = 10240;

#[derive(Serialize, Deserialize)]
pub struct HardwareState {
    cycle: u64,
    next_tick: u64,
    next_controller_read: u64,
    reload_value: u16,
    zero_flag: bool,
    interrupt_requested: bool,
}
impl Default for HardwareState {
    fn default() -> Self {
        Self {
            cycle: 0,
            next_tick: u64::MAX,
            next_controller_read: u64::MAX,
            reload_value: 0,
            zero_flag: false,
            interrupt_requested: false,
        }
    }
}

pub struct Hardware {
    cycle: u64,
    next_tick: u64,
    next_controller_read: u64,
    reload_value: u16,
    zero_flag: bool,
    interrupt_requested: bool,
    memory: Rc<RefCell<Memory>>,
    controller_state: Option<Arc<AtomicU16>>,
}
impl Hardware {
    pub fn new(memory: Rc<RefCell<Memory>>) -> Hardware {
        let state = HardwareState::default();
        Hardware {
            cycle: state.cycle,
            next_tick: state.next_tick,
            next_controller_read: state.next_controller_read,
            reload_value: state.reload_value,
            zero_flag: state.zero_flag,
            interrupt_requested: state.interrupt_requested,
            memory,
            controller_state: None,
        }
    }

    pub fn init(&mut self) {
        self.load_state(&HardwareState::default());
        let mut memory = self.memory.borrow_mut();
        memory.write_byte(TCR, 0);
        memory.write_halfword(TLR, 0xff);
        memory.write_halfword(THR, 0xff);
    }

    pub fn save_state(&self) -> HardwareState {
        HardwareState {
            cycle: self.cycle,
            next_tick: self.next_tick,
            next_controller_read: self.next_controller_read,
            reload_value: self.reload_value,
            zero_flag: self.zero_flag,
            interrupt_requested: self.interrupt_requested,
        }
    }

    pub fn load_state(&mut self, state: &HardwareState) {
        self.cycle = state.cycle;
        self.next_tick = state.next_tick;
        self.next_controller_read = state.next_controller_read;
        self.reload_value = state.reload_value;
        self.zero_flag = state.zero_flag;
        self.interrupt_requested = state.interrupt_requested;
    }

    pub fn get_controller_state(&mut self) -> Arc<AtomicU16> {
        let controller_state = Arc::new(AtomicU16::new(0));
        self.controller_state = Some(Arc::clone(&controller_state));
        controller_state
    }

    // When is the next time that this module will do something that affects other modules?
    pub fn next_event(&self) -> u64 {
        self.next_tick.min(self.next_controller_read)
    }

    // Get any unacknowledged interrupt from this module
    pub fn active_interrupt(&self) -> Option<Exception> {
        if self.interrupt_requested {
            Some(Exception::interrupt(0xfe10, 1))
        } else {
            None
        }
    }

    // The CPU has written to somewhere in the hardware address space
    // Update whatever internal state we need to in response
    pub fn process_event(&mut self, address: usize) -> bool {
        if address == THR || address == TLR {
            // A game set the timer, so start counting down from the new value
            self.reload_value = self.read_timer();
            self.compute_next_tick();
        }
        if address == TCR {
            // A game wrote to the timer control register, do what we gotta do
            self.update_timer_settings();
        }
        if address == SCR {
            // A game is attempting to read controller input
            self.handle_controller_read();
        }
        // The CPU only needs to stop what it's doing if an interrupt is active
        !self.interrupt_requested
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
            self.interrupt_requested = false;
        }
        if (tcr & T_CLEAR_ZERO) != 0 {
            // Interrupt was acknowledged and zero flag was cleared
            self.interrupt_requested = false;
            self.zero_flag = false;
        }
        self.correct_tcr();
    }

    fn handle_controller_read(&mut self) {
        let mut memory = self.memory.borrow_mut();
        let value = memory.read_byte(SCR);
        let mut new_value = value | S_HW_READ;
        if value & S_HW_ABORT != 0 {
            // hardware read was cancelled
            self.next_controller_read = u64::MAX;
            new_value &= !S_HW_STAT;
        } else if value & S_HW_READ != 0 {
            // hardware read has begun
            self.next_controller_read = self.cycle + HARDWARE_READ_CYCLES;
            new_value |= S_HW_STAT;
        }
        memory.write_byte(SCR, new_value);
    }

    pub fn run(&mut self, target_cycle: u64) {
        while self.cycle < target_cycle {
            self.cycle = std::cmp::min(target_cycle, self.next_tick);
            if self.cycle == self.next_tick {
                let old_timer_value = self.read_timer();
                let new_timer_value = if old_timer_value == 0 {
                    self.reload_value
                } else {
                    old_timer_value - 1
                };
                if old_timer_value == 1 && new_timer_value == 0 {
                    // Timer's going off!

                    // Set the flag that says the timer went off
                    self.zero_flag = true;

                    // Maybe fire the interrupt
                    if (self.memory.borrow().read_byte(TCR) & T_INTERRUPT) != 0 {
                        self.interrupt_requested = true;
                    }
                }
                // Keep on ticking
                self.write_timer(new_timer_value);
                self.compute_next_tick();
            }
        }
        self.correct_tcr();
        if self.cycle >= self.next_controller_read {
            // hardware read completed
            self.read_controller();
            self.next_controller_read = u64::MAX;
        }
    }

    fn read_timer(&self) -> u16 {
        let memory = self.memory.borrow();
        (memory.read_halfword(THR) as u16) << 8 | memory.read_halfword(TLR) as u16
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
            let interval = if self.reload_value == 1 && self.read_timer() == 0 {
                70
            } else if (tcr & T_INTERVAL) != 0 {
                400
            } else {
                2000
            };
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

    fn read_controller(&self) {
        let input_state = match self.controller_state.as_ref() {
            Some(state) => state.load(Ordering::Relaxed),
            None => 0x0000,
        };
        let sdlr = input_state & 0xff;
        let sdhr = (input_state >> 8) & 0xff;
        let mut memory = self.memory.borrow_mut();
        memory.write_halfword(SDLR, sdlr);
        memory.write_halfword(SDHR, sdhr);

        // Mark in SCR that we've finished reading
        let scr = memory.read_byte(SCR);
        memory.write_byte(SCR, scr & !S_HW_STAT);
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::hardware::{
        Hardware, SCR, SDHR, SDLR, S_HW_ABORT, S_HW_READ, S_HW_STAT, TCR, THR, TLR, T_CLEAR_ZERO,
        T_ENABLED, T_INTERRUPT, T_IS_ZERO,
    };
    use crate::emulator::memory::Memory;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::Ordering;

    fn set_tcr(hardware: &mut Hardware, memory: &Rc<RefCell<Memory>>, value: u8) {
        memory.borrow_mut().write_byte(TCR, value);
        hardware.process_event(TCR);
    }

    fn set_scr(hardware: &mut Hardware, memory: &Rc<RefCell<Memory>>, value: u8) {
        memory.borrow_mut().write_byte(SCR, value);
        hardware.process_event(SCR);
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
        assert_eq!(hardware.read_timer(), 0);
        assert_eq!(hardware.next_event(), 8000);

        hardware.run(hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(
            memory.borrow().read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO
        );
        assert_eq!(hardware.read_timer(), 3);
        assert_eq!(hardware.next_event(), 10000);
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

    #[test]
    fn performs_hardware_read() {
        let (mut hardware, memory) = get_hardware();

        // "start" pushing a button on the controller
        let state = hardware.get_controller_state();
        state.store(0x1002, Ordering::Relaxed);

        // Kick off the hardware read
        set_scr(&mut hardware, &memory, S_HW_READ);
        assert_eq!(hardware.next_event(), 10240);
        assert_eq!(memory.borrow().read_byte(SDHR), 0x00);
        assert_eq!(memory.borrow().read_byte(SDLR), 0x00);
        assert_eq!(memory.borrow().read_byte(SCR), S_HW_READ | S_HW_STAT);

        // Wait for the hardware read to complete
        hardware.run(hardware.next_event());
        assert_eq!(hardware.next_event(), u64::MAX);
        assert_eq!(memory.borrow().read_byte(SDHR), 0x10);
        assert_eq!(memory.borrow().read_byte(SDLR), 0x02);
        assert_eq!(memory.borrow().read_byte(SCR), S_HW_READ);
    }

    #[test]
    fn aborts_hardware_read() {
        let (mut hardware, memory) = get_hardware();

        // "start" pushing a button on the controller
        let state = hardware.get_controller_state();
        state.store(0x1002, Ordering::Relaxed);

        // Kick off the hardware read
        set_scr(&mut hardware, &memory, S_HW_READ);
        assert_eq!(hardware.next_event(), 10240);
        assert_eq!(memory.borrow().read_byte(SDHR), 0x00);
        assert_eq!(memory.borrow().read_byte(SDLR), 0x00);
        assert_eq!(memory.borrow().read_byte(SCR), S_HW_READ | S_HW_STAT);

        // run, but abort the hardware read before it should go off
        hardware.run(5120);
        set_scr(&mut hardware, &memory, S_HW_ABORT);
        hardware.run(10240);
        assert_eq!(hardware.next_event(), u64::MAX);
        assert_eq!(memory.borrow().read_byte(SDHR), 0x00);
        assert_eq!(memory.borrow().read_byte(SDLR), 0x00);
        assert_eq!(memory.borrow().read_byte(SCR), S_HW_READ | S_HW_ABORT);
    }
}
