use crate::emulator::memory::Memory;
use log::debug;
use ringbuf::{Consumer, Producer, RingBuffer};
use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

const CPU_CYCLES_PER_FRAME: u64 = 480;
const FRAMES_PER_SECOND: f32 = 20_000_000. / (CPU_CYCLES_PER_FRAME as f32);

const PCM_BASE_CYCLES_PER_FRAME: usize = (5_000_000. / FRAMES_PER_SECOND) as usize;
const NOISE_BASE_CYCLES_PER_FRAME: usize = (500_000. / FRAMES_PER_SECOND) as usize;
const FRAMES_PER_INTERVAL_CYCLE: usize = (FRAMES_PER_SECOND / 260.4) as usize;
const FRAMES_PER_ENVELOPE_CYCLE: usize = (FRAMES_PER_SECOND / 65.1) as usize;
const FREQ_MOD_BASE_CLOCK_0: usize = (FRAMES_PER_SECOND / 1041.6) as usize;
const FREQ_MOD_BASE_CLOCK_1: usize = (FRAMES_PER_SECOND / 130.2) as usize;

const ANALOG_FILTER_RC_CONSTANT: f32 = 0.022;
const ANALOG_FILTER_DECAY_RATE: f32 =
    ANALOG_FILTER_RC_CONSTANT / (ANALOG_FILTER_RC_CONSTANT + 1. / FRAMES_PER_SECOND);

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
enum Direction {
    Decay,
    Grow,
}
impl Default for Direction {
    fn default() -> Self {
        Direction::Decay
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
enum ChannelType {
    Pcm { waveform: usize, index: usize },
    Noise { tap: u16, register: u16 },
}
impl ChannelType {
    fn base_cycles_per_frame(&self) -> usize {
        match self {
            ChannelType::Pcm { .. } => PCM_BASE_CYCLES_PER_FRAME,
            ChannelType::Noise { .. } => NOISE_BASE_CYCLES_PER_FRAME,
        }
    }
    fn tick(&mut self) {
        match self {
            ChannelType::Pcm { index, .. } => {
                *index += 1;
                if *index == 32 {
                    *index = 0;
                }
            }
            ChannelType::Noise { tap, register } => {
                let tap_mask = 1 << *tap;
                let bit = ((*register & 0x0080) >> 7) ^ ((*register & tap_mask) >> *tap);
                *register = (*register << 1) | bit;
            }
        }
    }
    fn reset(&mut self) {
        match self {
            ChannelType::Pcm { index, .. } => {
                *index = 0;
            }
            ChannelType::Noise { register, .. } => {
                *register = 0xffff;
            }
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
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
    fn reset(&mut self) {
        self.counter = 0;
    }

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

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
enum ModFunction {
    Sweep,
    Modulation,
}
impl Default for ModFunction {
    fn default() -> Self {
        ModFunction::Sweep
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Default)]
struct Modification {
    enabled: bool,
    counter: usize,
    interval: usize,
    func: ModFunction,
    sweep_dir: Direction,
    sweep_shift: usize,
    mod_base: u16,
    mod_index: usize,
    mod_repeat: bool,
}
impl Modification {
    fn tick(&mut self) -> bool {
        if !self.enabled || self.interval == 0 {
            return false;
        }
        self.counter += 1;
        if self.counter >= self.interval {
            self.counter -= self.interval;
            true
        } else {
            false
        }
    }
    fn apply(&mut self, value: u16, mod_data: &[i16; 32]) -> u16 {
        match self.func {
            ModFunction::Sweep => self.apply_sweep(value),
            ModFunction::Modulation => self.apply_modulation(mod_data),
        }
    }
    fn apply_sweep(&mut self, value: u16) -> u16 {
        let delta = value >> self.sweep_shift;
        match self.sweep_dir {
            Direction::Grow => value + delta,
            Direction::Decay => value - delta,
        }
    }

    fn apply_modulation(&mut self, mod_data: &[i16; 32]) -> u16 {
        let res = mod_data[self.mod_index] + self.mod_base as i16;
        if self.mod_index < 31 {
            self.mod_index += 1;
        } else if self.mod_repeat {
            self.mod_index = 0;
        }
        res as u16
    }
}

fn set_low_byte(value: &mut u16, byte: u8) {
    *value = *value & 0x0700 | (byte as u16);
}
fn set_high_byte(value: &mut u16, byte: u8) {
    *value = *value & 0x00ff | ((byte as u16) << 8);
}

#[derive(Copy, Clone, Default, Serialize, Deserialize)]
struct Frequency {
    current_value: u16,
    most_recent_value: u16,
    counter: usize,
    modification: Option<Modification>,
}
impl Frequency {
    fn with_mod() -> Self {
        Frequency {
            modification: Some(Default::default()),
            ..Default::default()
        }
    }
    fn reset(&mut self) {
        self.counter = 0;
        if let Some(modification) = self.modification.as_mut() {
            modification.counter = 0;
            modification.mod_index = 0;
        }
    }
    fn set_low_byte(&mut self, value: u8) {
        set_low_byte(&mut self.current_value, value);
        set_low_byte(&mut self.most_recent_value, value);
        self.on_set();
    }
    fn set_high_byte(&mut self, value: u8) {
        set_high_byte(&mut self.current_value, value);
        set_high_byte(&mut self.most_recent_value, value);
        self.on_set();
    }
    fn on_set(&mut self) {
        self.counter = 0;
        if let Some(modification) = self.modification.as_mut() {
            modification.mod_base = self.most_recent_value;
        }
    }
    // returns number of updates, and whether the channel has shut off
    fn tick(&mut self, cycles: usize, mod_data: &[i16; 32]) -> (u16, bool) {
        // Frequency modification is computed before the tick
        let (new_value, cut_off) = self.tick_mod(mod_data);
        if cut_off {
            return (0, true);
        }

        let mut result = 0;
        self.counter += cycles;
        let change_cycles = 2048 - self.current_value as usize;
        while self.counter >= change_cycles {
            result += 1;
            self.counter -= change_cycles;
        }

        // Frequency modification happens after the tick
        self.current_value = new_value;

        (result, false)
    }

    // return new frequency, and whether the channel should get cut off
    fn tick_mod(&mut self, mod_data: &[i16; 32]) -> (u16, bool) {
        if let Some(modification) = self.modification.as_mut() {
            let modify = modification.tick();
            if modify {
                let new_value = modification.apply(self.current_value, mod_data);
                return (new_value, new_value > 2047);
            }
        }
        (self.current_value, false)
    }

    fn setup_mod_1(&mut self, enabled: bool, repeat: bool, func: ModFunction) {
        if let Some(modification) = self.modification.as_mut() {
            modification.enabled = enabled;
            modification.mod_repeat = repeat;
            modification.func = func;
        }
    }

    fn setup_mod_2(&mut self, clock: u8, interval: usize, dir: Direction, shift: usize) {
        if let Some(modification) = self.modification.as_mut() {
            modification.interval = match clock {
                0 => interval * FREQ_MOD_BASE_CLOCK_0,
                _ => interval * FREQ_MOD_BASE_CLOCK_1,
            };
            modification.sweep_dir = dir;
            modification.sweep_shift = shift;
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
struct Channel {
    enabled: bool,
    enabled_counter: Option<usize>,
    frequency: Frequency,
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
            Channel {
                frequency: Frequency::with_mod(),
                ..Channel::pcm()
            },
            Channel::noise(),
        ]
    }
    fn pcm() -> Self {
        Channel::new(ChannelType::Pcm {
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
            frequency: Default::default(),
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
        self.frequency.reset();
        self.envelope.reset();
        self.channel_type.reset();
    }
    fn set_waveform(&mut self, waveform_index: usize) {
        if let ChannelType::Pcm { waveform, .. } = &mut self.channel_type {
            *waveform = waveform_index;
        }
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
            // HACK: this behavior isn't documented anywhere, but stopping this channel now
            // fixes Hyper Fighter and doesn't break anything else I've tried
            if !self.envelope.enabled && self.envelope.value == 7 && self.volume == (15, 15) {
                self.enabled = false;
            }
        }
    }

    fn next(&mut self, waveforms: &[[u16; 32]; 5], mod_data: &[i16; 32]) -> (u16, u16) {
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
        let sample = self.sample(waveforms, mod_data);
        if !self.enabled {
            return (0, 0);
        }
        let left = self.amplitude(self.volume.0) * sample;
        let right = self.amplitude(self.volume.1) * sample;
        self.envelope.tick();
        (left, right)
    }

    fn amplitude(&self, volume: u16) -> u16 {
        let amplitude = (self.envelope.value * volume) >> 3;
        if self.envelope.value != 0 && volume != 0 {
            amplitude + 1
        } else {
            0
        }
    }

    fn sample(&mut self, waveforms: &[[u16; 32]; 5], mod_data: &[i16; 32]) -> u16 {
        let cycles = self.channel_type.base_cycles_per_frame();
        let (ticks, shutoff) = self.frequency.tick(cycles, mod_data);
        if shutoff {
            self.enabled = false;
            return 0;
        }
        for _ in 0..ticks {
            self.channel_type.tick();
        }
        match &self.channel_type {
            ChannelType::Pcm { waveform, index } => match waveforms.get(*waveform) {
                Some(waveform) => waveform[*index],
                None => 0,
            },
            ChannelType::Noise { register, .. } => {
                if *register & 0x0001 != 0 {
                    0
                } else {
                    63
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AudioState {
    cycle: u64,
    prev_input: (f32, f32),
    prev_output: (f32, f32),
    waveforms: [[u16; 32]; 5],
    mod_data: [i16; 32],
    channels: [Channel; 6],
}
impl Default for AudioState {
    fn default() -> Self {
        Self {
            cycle: 0,
            prev_input: (0., 0.),
            prev_output: (0., 0.),
            waveforms: [[0; 32]; 5],
            mod_data: [0; 32],
            channels: Channel::default_set(),
        }
    }
}

pub struct AudioController {
    cycle: u64,
    prev_input: (f32, f32),
    prev_output: (f32, f32),
    waveforms: [[u16; 32]; 5],
    mod_data: [i16; 32],
    channels: [Channel; 6],
    memory: Rc<RefCell<Memory>>,
    buffer: Option<Producer<(f32, f32)>>,
}

impl AudioController {
    pub fn new(memory: Rc<RefCell<Memory>>) -> AudioController {
        let state = AudioState::default();
        AudioController {
            cycle: state.cycle,
            prev_input: state.prev_input,
            prev_output: state.prev_output,
            waveforms: state.waveforms,
            mod_data: state.mod_data,
            channels: state.channels,
            memory,
            buffer: None,
        }
    }

    pub fn init(&mut self) {
        self.load_state(&AudioState::default());
    }

    pub fn save_state(&self) -> AudioState {
        AudioState {
            cycle: self.cycle,
            prev_input: self.prev_input,
            prev_output: self.prev_output,
            waveforms: self.waveforms,
            mod_data: self.mod_data,
            channels: self.channels,
        }
    }

    pub fn load_state(&mut self, state: &AudioState) {
        self.cycle = state.cycle;
        self.prev_input = state.prev_input;
        self.prev_output = state.prev_output;
        self.waveforms = state.waveforms;
        self.mod_data = state.mod_data;
        self.channels = state.channels;
    }

    pub fn claim_player(&mut self, volume: f32, buffer_size: usize) -> AudioPlayer {
        let capacity = buffer_size * 833;
        let buffer = RingBuffer::new(capacity);
        let (producer, consumer) = buffer.split();
        self.buffer = Some(producer);
        AudioPlayer {
            buffer: consumer,
            volume,
            prev_value: (0., 0.),
        }
    }

    pub fn process_event(&mut self, address: usize) {
        if address & 0x00000003 != 0 {
            return
        }
        let memory = self.memory.borrow();
        let address = address & 0x010007ff;
        let value = memory.read_byte(address);
        match address {
            0x01000000..=0x0100027f => {
                // Load waveform data (if all channels are disabled)
                if self.channels.iter().all(|c| !c.enabled) {
                    let rel_addr = address - 0x01000000;
                    let waveform = rel_addr / 128;
                    let index = (rel_addr % 128) / 4;
                    debug!(
                        "0x{:08x} = 0x{:02x} (load waveform ({}, {}) = {})",
                        address,
                        value,
                        waveform,
                        index,
                        value & 0x3f
                    );
                    self.waveforms[waveform][index] = (value as u16) & 0x3f;
                }
            }
            0x01000280..=0x010002ff => {
                // Load modulation data (if the channel using it is disabled)
                debug!("0x{:08x} = 0x{:02x} (load mod data)", address, value);
                if !self.channels[4].enabled {
                    let index = (address - 0x01000280) / 4;
                    self.mod_data[index] = value as i8 as i16;
                }
            }
            0x01000400..=0x0100057f => {
                // various channel controls
                let rel_addr = address - 0x01000400;
                let channel = rel_addr / 64;

                // Each channel is controlled by 64-byte address spaces with near-identical layouts
                match rel_addr % 64 {
                    0x00 => {
                        // Channel enabled/disabled + auto-interval config
                        let enabled = value & 0x80 != 0;
                        let auto = value & 0x20 != 0;
                        let interval = value as usize & 0x1f;
                        debug!("0x{:08x} = 0x{:02x} (channel {} enablement enabled={} auto={} interval={})", address, value, channel + 1, enabled, auto, interval);
                        self.channels[channel].set_enabled(enabled, auto, interval);
                    }
                    0x04 => {
                        // Channel stereo volume
                        let left_vol = (value as u16 >> 4) & 0x0f;
                        let right_vol = value as u16 & 0x0f;
                        debug!(
                            "0x{:08x} = 0x{:02x} (channel {} volume left={} right={})",
                            address,
                            value,
                            channel + 1,
                            left_vol,
                            right_vol
                        );
                        self.channels[channel].volume = (left_vol, right_vol);
                    }
                    0x08 => {
                        // Channel frequency (low byte)
                        debug!(
                            "0x{:08x} = 0x{:02x} (channel {} frequency low={})",
                            address,
                            value,
                            channel + 1,
                            value
                        );
                        self.channels[channel].frequency.set_low_byte(value);
                    }
                    0x0c => {
                        // Channel frequency (high byte)
                        debug!(
                            "0x{:08x} = 0x{:02x} (channel {} frequency high={})",
                            address,
                            value,
                            channel + 1,
                            value
                        );
                        self.channels[channel].frequency.set_high_byte(value & 0x07);
                    }
                    0x10 => {
                        // Channel envelope settings
                        let env_value = (value >> 4) as u16;
                        let direction = if value & 0x08 != 0 {
                            Direction::Grow
                        } else {
                            Direction::Decay
                        };
                        let interval = value as usize & 0x07;
                        debug!("0x{:08x} = 0x{:02x} (channel {} envelope settings env_value={} direction={:?} interval={})", address, value, channel + 1, env_value, direction, interval);
                        self.channels[channel]
                            .envelope
                            .set(env_value, direction, interval);
                    }
                    0x14 => {
                        // Channel envelope modification settings
                        let enabled = value & 0x01 != 0;
                        let repeat = value & 0x02 != 0;
                        debug!(
                            "0x{:08x} = 0x{:02x} (channel {} envelope mod enabled={} repeat={})",
                            address,
                            value,
                            channel + 1,
                            enabled,
                            repeat
                        );
                        self.channels[channel]
                            .envelope
                            .set_modification(enabled, repeat);

                        if channel == 4 {
                            // Channel 5 has additional envelope specification settings
                            // This is to support sweep/modulation control
                            let enabled = value & 0x40 != 0;
                            let repeat = value & 0x20 != 0;
                            let func = if value & 0x10 != 0 {
                                ModFunction::Modulation
                            } else {
                                ModFunction::Sweep
                            };
                            debug!(
                                "0x{:08x} = 0x{:02x} (channel {} envelope bonus enabled={} repeat={} func={:?})",
                                address,
                                value,
                                channel + 1,
                                enabled,
                                repeat,
                                func
                            );
                            self.channels[4]
                                .frequency
                                .setup_mod_1(enabled, repeat, func);
                        }
                        if channel == 5 {
                            // This sets the "tap" for the noise channel (channel 6)
                            let tap = (value >> 4) & 0x07;
                            debug!(
                                "0x{:08x} = 0x{:02x} (channel {} tap = {})",
                                address,
                                value,
                                channel + 1,
                                tap
                            );
                            self.channels[5].set_tap(tap);
                        }
                    }
                    0x18 if channel < 5 => {
                        // Set active waveform for the PCM channels (everything but 6)
                        let wave = value as usize & 0x0f;
                        debug!(
                            "0x{:08x} = 0x{:02x} (channel {} active waveform is {})",
                            address,
                            value,
                            channel + 1,
                            wave
                        );
                        self.channels[channel].set_waveform(wave);
                    }
                    0x1c if channel == 4 => {
                        // Sweep/modulation settings specifically for channel 5
                        let clock = value >> 7;
                        let interval = (value as usize >> 4) & 0x07;
                        let dir = if value & 0x08 != 0 {
                            Direction::Grow
                        } else {
                            Direction::Decay
                        };
                        let shift = value as usize & 0x07;
                        debug!(
                            "0x{:08x} = 0x{:02x} (channel {} envelope mod bonus 2 clock={} interval={} dir={:?} shift={})",
                            address,
                            value,
                            channel + 1,
                            clock,
                            interval,
                            dir,
                            shift
                        );
                        self.channels[4]
                            .frequency
                            .setup_mod_2(clock, interval, dir, shift);
                    }
                    _ => { /* unknown channels are harmless */ }
                }
            }
            0x01000580 => {
                // Stop all sound
                debug!("0x{:08x} = 0x{:02x} (STOP AT ONCE)", address, value);
                if value & 1 != 0 {
                    for channel in &mut self.channels {
                        channel.set_enabled(false, false, 0);
                    }
                }
            }
            _ => { /* unknown channels are harmless */ }
        };
    }

    pub fn run(&mut self, target_cycle: u64) {
        let mut values = Vec::new();
        let waveforms = &self.waveforms;
        let mod_data = &self.mod_data;
        while self.cycle < target_cycle {
            let frame = self
                .channels
                .iter_mut()
                .map(|c| c.next(waveforms, mod_data))
                .fold((0, 0), |acc, val| (acc.0 + val.0, acc.1 + val.1));

            let input = self.normalize_frame(frame);
            let output = self.apply_analog_filter(input);
            values.push(output);
            self.prev_input = input;
            self.prev_output = output;

            self.cycle += CPU_CYCLES_PER_FRAME;
        }
        if let Some(buffer) = self.buffer.as_mut() {
            buffer.push_slice(&values);
        }
    }

    fn normalize_frame(&self, frame: (u16, u16)) -> (f32, f32) {
        fn to_float(input: u16) -> f32 {
            (input >> 4) as f32 / 685.
        }
        (to_float(frame.0), to_float(frame.1))
    }

    fn apply_analog_filter(&self, input: (f32, f32)) -> (f32, f32) {
        fn to_analog(input: f32, prev_input: f32, prev_output: f32) -> f32 {
            ANALOG_FILTER_DECAY_RATE * (prev_output + input - prev_input)
        }
        (
            to_analog(input.0, self.prev_input.0, self.prev_output.0),
            to_analog(input.1, self.prev_input.1, self.prev_output.1),
        )
    }
}

pub struct AudioPlayer {
    buffer: Consumer<(f32, f32)>,
    prev_value: (f32, f32),
    volume: f32,
}

impl AudioPlayer {
    pub fn play(&mut self, frames: &mut [(f32, f32)]) {
        let count = self.buffer.pop_slice(frames);
        for frame in &mut frames[..count] {
            frame.0 *= self.volume;
            frame.1 *= self.volume;
        }
        // If we don't know what to play, play that last thing again
        let value = if count == 0 {
            self.prev_value
        } else {
            frames[count - 1]
        };
        self.prev_value = value;
        for missing in &mut frames[count..] {
            *missing = (value.0 * self.volume, value.1 * self.volume);
        }
    }
}
