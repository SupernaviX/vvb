use super::storage::Storage;
use crate::emulator::cpu::Interrupt;

const SDLR: usize = 0x02000010;
const SDHR: usize = 0x02000014;
const TLR: usize = 0x02000018;
const THR: usize = 0x0200001c;
const TCR: usize = 0x02000020;

// TCR bits
const T_INTERVAL: i8 = 0x10;
const T_INTERRUPT: i8 = 0x08;
const T_CLEAR_ZERO: i8 = 0x04;
const T_IS_ZERO: i8 = 0x02;
const T_ENABLED: i8 = 0x01;

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

    pub fn init(&mut self, storage: &mut Storage) {
        self.cycle = 0;
        self.next_tick = u64::MAX;
        self.reload_value = 0;
        self.zero_flag = false;
        self.interrupt = None;
        storage.write_byte(TCR, 0);
        storage.write_halfword(TLR, 0xff);
        storage.write_halfword(THR, 0xff);
    }

    pub fn process_inputs(&self, storage: &mut Storage, input_state: u16) {
        // Always set flag 0x02 on the lower register, "real" controllers do
        let sdlr = input_state as i16 & 0xff | 0x02;
        let sdhr = (input_state >> 8) as i16 & 0xff;
        storage.write_halfword(SDLR, sdlr);
        storage.write_halfword(SDHR, sdhr);
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
    pub fn process_event(&mut self, storage: &mut Storage, address: usize) {
        if address == THR || address == TLR {
            // A game set the timer, so start counting down from the new value
            self.reload_value = self.read_timer(storage);
            self.compute_next_tick(storage);
        }
        if address == TCR {
            // A game wrote to the timer control register, do what we gotta do
            self.update_timer_settings(storage);
        }
    }

    fn update_timer_settings(&mut self, storage: &mut Storage) {
        let tcr = storage.read_byte(TCR);
        if (tcr & T_ENABLED) == 0 {
            // Stop counting down
            self.next_tick = u64::MAX;
            return;
        } else {
            self.compute_next_tick(storage);
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
        self.correct_tcr(storage);
    }

    pub fn run(&mut self, storage: &mut Storage, target_cycle: u64) {
        while self.cycle < target_cycle {
            self.cycle = std::cmp::min(target_cycle, self.next_tick);
            if self.cycle == self.next_tick {
                let new_timer_value = self.read_timer(storage) - 1;
                if new_timer_value == 0 {
                    // Timer's going off!

                    // Set the flag that says the timer went off
                    self.zero_flag = true;

                    // Fire the interrupt
                    self.interrupt = Some(Interrupt {
                        code: 0xfe10,
                        level: 2,
                        handler: 0xfffffe10,
                    });

                    // Reset the timer
                    self.write_timer(storage, self.reload_value);
                    self.compute_next_tick(storage);
                } else {
                    // Keep on ticking
                    self.write_timer(storage, new_timer_value);
                    self.compute_next_tick(storage);
                }
            }
        }
        self.correct_tcr(storage);
    }

    fn read_timer(&self, storage: &Storage) -> u16 {
        return (storage.read_halfword(THR) as u16) << 8 | storage.read_halfword(TLR) as u16;
    }

    fn write_timer(&self, storage: &mut Storage, value: u16) {
        storage.write_halfword(THR, (value >> 8) as i16);
        storage.write_halfword(TLR, (value & 0xff) as i16);
    }

    fn compute_next_tick(&mut self, storage: &mut Storage) {
        let tcr = storage.read_byte(TCR);
        let enabled = (tcr & T_ENABLED) != 0;
        if enabled && self.reload_value != 0 {
            let interval = if (tcr & T_INTERVAL) == 1 { 400 } else { 2000 };
            self.next_tick = self.cycle + interval;
        } else {
            self.next_tick = u64::MAX;
        }
    }

    fn correct_tcr(&self, storage: &mut Storage) {
        // update TCR to what the CPU expects
        let mut tcr = storage.read_byte(TCR);
        if self.zero_flag {
            tcr |= T_IS_ZERO;
        } else {
            tcr &= !T_IS_ZERO;
        }
        tcr |= T_CLEAR_ZERO;
        storage.write_byte(TCR, tcr);
    }
}

#[cfg(test)]
mod tests {
    use crate::emulator::hardware::{
        Hardware, TCR, THR, TLR, T_CLEAR_ZERO, T_ENABLED, T_INTERRUPT, T_IS_ZERO,
    };
    use crate::emulator::storage::Storage;

    fn set_tcr(hardware: &mut Hardware, storage: &mut Storage, value: i8) {
        storage.write_byte(TCR, value);
        hardware.process_event(storage, TCR);
    }

    fn set_timer(hardware: &mut Hardware, storage: &mut Storage, value: u16) {
        storage.write_halfword(THR, (value >> 8) as i16);
        hardware.process_event(storage, THR);
        storage.write_halfword(TLR, value as i16);
        hardware.process_event(storage, TLR);
    }

    #[test]
    fn does_nothing_interesting_when_timer_is_off() {
        let mut hardware = Hardware::new();
        let mut storage = Storage::new();

        set_tcr(&mut hardware, &mut storage, 0);
        assert_eq!(hardware.next_event(), u64::MAX);
        hardware.run(&mut storage, 1000000);
        assert_eq!(hardware.next_event(), u64::MAX);
    }

    #[test]
    fn does_nothing_when_timer_is_on_but_time_is_unset() {
        let mut hardware = Hardware::new();
        let mut storage = Storage::new();

        set_tcr(&mut hardware, &mut storage, T_ENABLED);
        assert_eq!(hardware.next_event(), u64::MAX);
        hardware.run(&mut storage, 1000000);
        assert_eq!(hardware.next_event(), u64::MAX);
    }

    #[test]
    fn counts_down_when_timer_is_configured_properly() {
        let mut hardware = Hardware::new();
        let mut storage = Storage::new();

        set_timer(&mut hardware, &mut storage, 3);
        assert_eq!(storage.read_byte(TCR), 0);
        assert_eq!(hardware.read_timer(&storage), 3);
        assert_eq!(hardware.next_event(), u64::MAX);

        set_tcr(&mut hardware, &mut storage, T_ENABLED);
        assert_eq!(storage.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(&storage), 3);
        assert_eq!(hardware.next_event(), 2000);

        hardware.run(&mut storage, 1000);
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(storage.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(&storage), 3);
        assert_eq!(hardware.next_event(), 2000);

        hardware.run(&mut storage, hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(storage.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(&storage), 2);
        assert_eq!(hardware.next_event(), 4000);

        hardware.run(&mut storage, hardware.next_event());
        assert!(hardware.active_interrupt().is_none());
        assert_eq!(storage.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert_eq!(hardware.read_timer(&storage), 1);
        assert_eq!(hardware.next_event(), 6000);

        hardware.run(&mut storage, hardware.next_event());
        assert!(hardware.active_interrupt().is_some());
        assert_eq!(storage.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO);
        assert_eq!(hardware.read_timer(&storage), 3);
        assert_eq!(hardware.next_event(), 8000);
    }

    #[test]
    fn handles_interrupt_cycle() {
        let mut hardware = Hardware::new();
        let mut storage = Storage::new();

        set_timer(&mut hardware, &mut storage, 3);
        set_tcr(&mut hardware, &mut storage, T_ENABLED | T_INTERRUPT);
        assert!(hardware.active_interrupt().is_none());

        // interrupt fires when clock goes off
        hardware.run(&mut storage, 6000);
        assert_eq!(
            storage.read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO | T_INTERRUPT
        );
        assert!(hardware.active_interrupt().is_some());

        // interrupt keeps firing while unacknowledged
        hardware.run(&mut storage, 7000);
        assert_eq!(
            storage.read_byte(TCR),
            T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO | T_INTERRUPT
        );
        assert!(hardware.active_interrupt().is_some());

        // acknowledge interrupt
        set_tcr(&mut hardware, &mut storage, T_ENABLED);
        hardware.run(&mut storage, 8000);
        assert_eq!(storage.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO | T_IS_ZERO);
        assert!(hardware.active_interrupt().is_none());

        // clear zero
        set_tcr(&mut hardware, &mut storage, T_ENABLED | T_CLEAR_ZERO);
        hardware.run(&mut storage, 9000);
        assert_eq!(storage.read_byte(TCR), T_ENABLED | T_CLEAR_ZERO);
        assert!(hardware.active_interrupt().is_none());
    }
}