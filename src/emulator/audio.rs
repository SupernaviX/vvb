use crate::emulator::memory::Memory;
use ringbuf::{Consumer, Producer, RingBuffer};

pub struct AudioController {
    cycle: u64,
    buffer: Option<Producer<i16>>,
    waveforms: [[i16; 32]; 5],
    channels: [Channel; 5],
}

#[derive(Default)]
struct Channel {
    enabled: bool,
    waveform: usize,
    frequency: usize,
    frequency_counter: usize,
    index: usize,
}
impl Channel {
    fn set_settings(&mut self, enabled: bool, _auto: bool, _interval: usize) {
        self.enabled = enabled;
        self.frequency_counter = 0;
        self.index = 0;
    }
    fn set_waveform(&mut self, waveform: usize) {
        if self.enabled {
            return;
        }
        self.waveform = waveform;
        self.frequency_counter = 0;
    }
    fn set_frequency(&mut self, frequency: usize) {
        self.frequency = 2048 - frequency;
        self.frequency_counter = 0;
    }
    fn next(&mut self, waveforms: &[[i16; 32]; 5]) -> i16 {
        if !self.enabled {
            return 0;
        }
        self.frequency_counter += 120; // ~120 base cycles per audio frame
        while self.frequency_counter > self.frequency {
            self.index += 1;
            if self.index == 32 {
                self.index = 0;
            }
            self.frequency_counter -= self.frequency;
        }
        waveforms[self.waveform][self.index]
    }
}

impl AudioController {
    pub fn new() -> AudioController {
        AudioController {
            cycle: 0,
            buffer: None,
            waveforms: [[0; 32]; 5],
            channels: Default::default(),
        }
    }

    pub fn init(&mut self) {
        self.cycle = 0;
        self.buffer = None;
        self.waveforms = [[0; 32]; 5];
        self.channels = Default::default();
    }

    pub fn get_player(&mut self) -> AudioPlayer {
        let buffer = RingBuffer::new(41700);
        let (producer, consumer) = buffer.split();
        self.buffer = Some(producer);
        AudioPlayer { buffer: consumer }
    }

    pub fn process_event(&mut self, memory: &mut Memory, address: usize) {
        let value = memory.read_byte(address);
        if address < 0x01000280 {
            let rel_addr = address - 0x01000000;
            let waveform = rel_addr / 128;
            let index = (rel_addr % 128) / 4;
            log::debug!("Waveform {}[{}] := 0x{:02x}", waveform + 1, index, value);
            self.waveforms[waveform][index] = (((value as i16) & 0x3f) - 32) * 32;
            return;
        }
        if address < 0x01000300 {
            log::debug!(
                "Modulation[{}] := 0x{:02x}",
                (address - 0x01000280) / 4,
                value
            );
            return;
        }
        if 0x01000400 <= address && address < 0x01000580 {
            let rel_addr = address - 0x01000400;
            match (rel_addr / 64, rel_addr % 64) {
                (channel, 0x00) => {
                    let enabled = value & 0x80 != 0;
                    let auto = value & 0x20 != 0;
                    let interval = value as usize & 0x1f;
                    log::debug!(
                        "Channel {} sound interval: enabled={} auto={} interval={}",
                        channel + 1,
                        enabled,
                        auto,
                        interval
                    );
                    if channel < 5 {
                        self.channels[channel].set_settings(enabled, auto, interval);
                    }
                }
                (channel, 0x04) => {
                    let left_vol = (value >> 4) & 0x0f;
                    let right_vol = value & 0x0f;
                    log::debug!(
                        "Channel {} volume: left={} right={}",
                        channel + 1,
                        left_vol,
                        right_vol
                    );
                }
                (channel, 0x08) => {
                    let low = value as usize;
                    let high = memory.read_byte(address + 4) as usize;
                    let frequency = ((high & 0x07) << 8) + low;
                    log::debug!("Channel {} frequency (low): {}", channel + 1, frequency);
                    if channel < 5 {
                        self.channels[channel].set_frequency(frequency);
                    }
                }
                (channel, 0x0c) => {
                    let low = memory.read_byte(address - 4) as usize;
                    let high = value as usize;
                    let frequency = ((high & 0x07) << 8) + low;
                    log::debug!("Channel {} frequency (high): {}", channel + 1, frequency);
                    if channel < 5 {
                        self.channels[channel].set_frequency(frequency);
                    }
                }
                (channel, 0x10) => {
                    let val = value >> 4;
                    let dir = if value & 0x08 != 0 { "grow" } else { "decay" };
                    let interval = value & 0x07;
                    log::debug!(
                        "Channel {} envelope: value={} dir={} interval={}",
                        channel + 1,
                        val,
                        dir,
                        interval
                    );
                }
                (channel, 0x14) => {
                    let repeat = value & 0x02 != 0;
                    let enabled = value & 0x01 != 0;
                    log::debug!(
                        "Channel {} envelope: repeat={} enabled={}",
                        channel + 1,
                        repeat,
                        enabled
                    );

                    if channel == 4 {
                        let enabled = value & 0x40 != 0;
                        let repeat = value & 0x20 != 0;
                        let func = if value & 0x10 != 0 {
                            "modulation"
                        } else {
                            "sweep"
                        };
                        log::debug!(
                            "Channel 5 modifications: enabled={} repeat={} func={}",
                            enabled,
                            repeat,
                            func
                        );
                    }
                }
                (channel @ 0..=4, 0x18) => {
                    let wave = value as usize & 0x07;
                    log::debug!("Channel {} waveform: {}", channel + 1, wave + 1);
                    self.channels[channel].set_waveform(wave);
                }
                (4, 0x1c) => {
                    let clock = value >> 7;
                    let interval = (value >> 4) & 0x07;
                    let dir = if value & 0x08 != 0 { "add" } else { "subtract" };
                    let shift = value & 0x07;
                    log::debug!(
                        "Sweep/modulation: clock={} interval={} dir={} shift={}",
                        clock,
                        interval,
                        dir,
                        shift
                    );
                }
                _ => log::warn!(
                    "Unknown audio register: [0x{:08x}] := 0x{:02x}",
                    address,
                    value
                ),
            };
            return;
        }
        if address == 0x01000580 && value != 0 {
            log::debug!("Stop all sound!");
            return;
        }
    }

    pub fn run(&mut self, _memory: &mut Memory, target_cycle: u64) {
        let mut values = Vec::with_capacity((target_cycle - self.cycle) as usize / 480);
        let waveforms = &self.waveforms;
        while self.cycle < target_cycle {
            let new_value = self.channels.iter_mut().map(|c| c.next(waveforms)).sum();
            values.push(new_value);
            self.cycle += 480; // approximate number of cycles per frame
        }
        if let Some(buffer) = self.buffer.as_mut() {
            buffer.push_slice(&values);
        }
    }
}

pub struct AudioPlayer {
    buffer: Consumer<i16>,
}

impl AudioPlayer {
    pub fn play(&mut self, frames: &mut [i16]) {
        let count = self.buffer.pop_slice(frames);

        // If we don't know what to play, play nothing
        for missing in &mut frames[count..] {
            *missing = 0;
        }
    }
}
