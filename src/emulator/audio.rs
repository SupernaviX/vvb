use crate::emulator::memory::Memory;
use ringbuf::{Consumer, Producer, RingBuffer};

const CPU_CYCLES_PER_FRAME: u64 = 480;
const PCM_BASE_CYCLES_PER_FRAME: usize = 120;
const NOISE_BASE_CYCLES_PER_FRAME: usize = 12;
const FRAMES_PER_INTERVAL_CYCLE: usize = 160;
const FRAMES_PER_ENVELOPE_CYCLE: usize = 640;

#[derive(Debug)]
enum Direction {
    Decay,
    Grow,
}
impl Default for Direction {
    fn default() -> Self {
        Direction::Decay
    }
}

enum ChannelType {
    PCM { waveform: usize, index: usize },
    Noise { tap: u16, register: u16 },
}
impl ChannelType {
    fn reset(&mut self) {
        match self {
            ChannelType::PCM { index, .. } => {
                *index = 0;
            }
            ChannelType::Noise { register, .. } => {
                *register = 0xffff;
            }
        }
    }
}

#[derive(Default)]
struct Envelope {
    value: u16,
    direction: Direction,
    interval: usize,
    enabled: bool,
    repeat: bool,
    repeat_value: u16,
    counter: usize,
}
impl Envelope {
    fn set(&mut self, value: u16, direction: Direction, interval: usize) {
        self.value = value;
        self.repeat_value = value;
        self.direction = direction;
        self.interval = (interval + 1) * FRAMES_PER_ENVELOPE_CYCLE;
        self.counter = 0;
    }
    fn set_modification(&mut self, enabled: bool, repeat: bool) {
        self.enabled = enabled;
        self.repeat = repeat;
        self.counter = 0;
    }
    fn tick(&mut self) {
        if !self.enabled {
            return;
        }
        self.counter += 1;
        if self.counter >= self.interval {
            self.counter -= self.interval;
            let (value, done) = match self.direction {
                Direction::Grow => {
                    if self.value == 15 {
                        (15, true)
                    } else {
                        (self.value + 1, false)
                    }
                }
                Direction::Decay => {
                    if self.value == 0 {
                        (0, true)
                    } else {
                        (self.value - 1, false)
                    }
                }
            };
            self.value = value;
            if done && self.repeat {
                self.value = self.repeat_value;
            }
        }
    }
}

struct Channel {
    enabled: bool,
    enabled_counter: Option<usize>,
    frequency: usize,
    frequency_counter: usize,
    envelope: Envelope,
    volume: (u16, u16),
    channel_type: ChannelType,
}
impl Channel {
    fn default_set() -> [Self; 6] {
        [
            Channel::pcm(),
            Channel::pcm(),
            Channel::pcm(),
            Channel::pcm(),
            Channel::pcm(),
            Channel::noise(),
        ]
    }
    fn pcm() -> Self {
        Channel::new(ChannelType::PCM {
            waveform: 0,
            index: 0,
        })
    }
    fn noise() -> Self {
        Channel::new(ChannelType::Noise {
            tap: 14,
            register: 0xffff,
        })
    }
    fn new(channel_type: ChannelType) -> Self {
        Channel {
            enabled: false,
            enabled_counter: None,
            frequency: 2048,
            frequency_counter: 0,
            envelope: Default::default(),
            volume: (0, 0),
            channel_type,
        }
    }
    fn set_enabled(&mut self, enabled: bool, auto: bool, interval: usize) {
        self.enabled = enabled;
        if auto {
            self.enabled_counter = Some(FRAMES_PER_INTERVAL_CYCLE * (interval + 1));
        } else {
            self.enabled_counter = None;
        }
        self.frequency_counter = 0;
        self.envelope.counter = 0;
        self.channel_type.reset();
    }
    fn set_waveform(&mut self, waveform_index: usize) {
        if let ChannelType::PCM { waveform, .. } = &mut self.channel_type {
            *waveform = waveform_index;
        }
        self.frequency_counter = 0;
    }
    fn set_tap(&mut self, new_tap: u8) {
        if let ChannelType::Noise { tap, register } = &mut self.channel_type {
            // Each valid input maps to a bit, which results in a different sequence length
            *tap = match new_tap {
                0 => 14, // length 32767
                1 => 10, // length 1953
                2 => 13, // length 254
                3 => 4,  // length 217
                4 => 8,  // length 73
                5 => 6,  // length 63
                6 => 9,  // length 42
                _ => 11, // length 28
            };
            *register = 0xffff;
        }
    }
    fn set_frequency(&mut self, frequency: usize) {
        self.frequency = 2048 - frequency;
        self.frequency_counter = 0;
    }

    fn next(&mut self, waveforms: &[[u16; 32]; 5]) -> (u16, u16) {
        if let Some(counter) = self.enabled_counter.as_mut() {
            if *counter > 0 {
                *counter -= 1;
            } else {
                self.enabled = false;
                self.enabled_counter = None;
            }
        }
        if !self.enabled {
            return (0, 0);
        }
        let sample = self.sample(waveforms);
        let left = self.amplitude(self.volume.0) * sample;
        let right = self.amplitude(self.volume.1) * sample;
        self.envelope.tick();
        (left, right)
    }

    fn amplitude(&self, volume: u16) -> u16 {
        let amplitude = (self.envelope.value * volume) >> 3;
        if amplitude != 0 {
            amplitude + 1
        } else {
            0
        }
    }

    fn sample(&mut self, waveforms: &[[u16; 32]; 5]) -> u16 {
        match &mut self.channel_type {
            ChannelType::PCM { waveform, index } => {
                self.frequency_counter += PCM_BASE_CYCLES_PER_FRAME;
                while self.frequency_counter > self.frequency {
                    *index += 1;
                    if *index == 32 {
                        *index = 0;
                    }
                    self.frequency_counter -= self.frequency;
                }
                waveforms[*waveform][*index]
            }
            ChannelType::Noise { tap, register } => {
                let tap_mask = 1 << *tap;
                self.frequency_counter += NOISE_BASE_CYCLES_PER_FRAME;
                while self.frequency_counter > self.frequency {
                    let bit = ((*register & 0x0080) >> 7) ^ ((*register & tap_mask) >> *tap);
                    *register = (*register << 1) | bit;
                    self.frequency_counter -= self.frequency;
                }
                if *register & 0x0001 != 0 {
                    0
                } else {
                    63
                }
            }
        }
    }
}

pub struct AudioController {
    cycle: u64,
    buffer: Option<Producer<(i16, i16)>>,
    waveforms: [[u16; 32]; 5],
    channels: [Channel; 6],
}

impl AudioController {
    pub fn new() -> AudioController {
        AudioController {
            cycle: 0,
            buffer: None,
            waveforms: [[0; 32]; 5],
            channels: Channel::default_set(),
        }
    }

    pub fn init(&mut self) {
        self.cycle = 0;
        self.buffer = None;
        self.waveforms = [[0; 32]; 5];
        self.channels = Channel::default_set();
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
            self.waveforms[waveform][index] = (value as u16) & 0x3f;
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
                    self.channels[channel].set_enabled(enabled, auto, interval);
                }
                (channel, 0x04) => {
                    let left_vol = (value as u16 >> 4) & 0x0f;
                    let right_vol = value as u16 & 0x0f;
                    log::debug!(
                        "Channel {} volume: left={} right={}",
                        channel + 1,
                        left_vol,
                        right_vol
                    );
                    self.channels[channel].volume = (left_vol, right_vol);
                }
                (channel, 0x08) => {
                    let low = value as usize;
                    let high = memory.read_byte(address + 4) as usize;
                    let frequency = ((high & 0x07) << 8) + low;
                    log::debug!("Channel {} frequency (low): {}", channel + 1, frequency);
                    self.channels[channel].set_frequency(frequency);
                }
                (channel, 0x0c) => {
                    let low = memory.read_byte(address - 4) as usize;
                    let high = value as usize;
                    let frequency = ((high & 0x07) << 8) + low;
                    log::debug!("Channel {} frequency (high): {}", channel + 1, frequency);
                    self.channels[channel].set_frequency(frequency);
                }
                (channel, 0x10) => {
                    let val = (value >> 4) as u16;
                    let dir = if value & 0x08 != 0 {
                        Direction::Grow
                    } else {
                        Direction::Decay
                    };
                    let interval = value as usize & 0x07;
                    log::debug!(
                        "Channel {} envelope: value={} dir={:?} interval={}",
                        channel + 1,
                        val,
                        dir,
                        interval
                    );
                    self.channels[channel].envelope.set(val, dir, interval);
                }
                (channel, 0x14) => {
                    let enabled = value & 0x01 != 0;
                    let repeat = value & 0x02 != 0;
                    log::debug!(
                        "Channel {} envelope: enabled={} repeat={}",
                        channel + 1,
                        enabled,
                        repeat,
                    );
                    self.channels[channel]
                        .envelope
                        .set_modification(enabled, repeat);

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

                    if channel == 5 {
                        let tap = value >> 4 & 0x07;
                        log::debug!("Channel 6 tap: {}", tap);
                        self.channels[5].set_tap(tap);
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
        let mut values = Vec::new();
        let waveforms = &self.waveforms;
        while self.cycle < target_cycle {
            let frame = self
                .channels
                .iter_mut()
                .map(|c| c.next(waveforms))
                .fold((0, 0), |acc, val| (acc.0 + val.0, acc.1 + val.1));
            values.push(self.to_output_frame(frame));
            self.cycle += CPU_CYCLES_PER_FRAME;
        }
        if let Some(buffer) = self.buffer.as_mut() {
            buffer.push_slice(&values);
        }
    }

    fn to_output_frame(&self, frame: (u16, u16)) -> (i16, i16) {
        ((frame.0 >> 6) as i16 * 128, (frame.1 >> 6) as i16 * 128)
    }
}

pub struct AudioPlayer {
    buffer: Consumer<(i16, i16)>,
}

impl AudioPlayer {
    pub fn play(&mut self, frames: &mut [(i16, i16)]) {
        let count = self.buffer.pop_slice(frames);

        // If we don't know what to play, play nothing
        for missing in &mut frames[count..] {
            *missing = (0, 0);
        }
    }
}