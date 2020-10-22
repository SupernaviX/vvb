use crate::emulator::memory::Memory;
use anyhow::Result;
use std::f32::consts::PI;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;

#[derive(Debug)]
enum AudioInstruction {
    Play,
    Stop,
}

pub struct AudioController {
    player: Option<mpsc::Sender<AudioInstruction>>,
}

impl AudioController {
    pub fn new() -> AudioController {
        AudioController { player: None }
    }

    pub fn get_player(&mut self) -> AudioPlayer {
        let (tx, rx) = mpsc::channel();
        self.player = Some(tx);
        AudioPlayer {
            controller: rx,
            frequency: 440.0,
            gain: 0.5,
            waveform: None,
            playing: false,
            index: 0,
        }
    }

    pub fn process_event(&mut self, memory: &mut Memory, address: usize) {
        let value = memory.read_byte(address);
        if address < 0x01000280 {
            let rel_addr = address - 0x01000000;
            log::debug!(
                "Waveform {}[{}] := 0x{:02x}",
                (rel_addr / 128) + 1,
                (rel_addr % 128) / 4,
                value
            );
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
                    let interval = value & 0x1f;
                    log::debug!(
                        "Channel {} sound interval: enabled={} auto={} interval={}",
                        channel + 1,
                        enabled,
                        auto,
                        interval
                    );

                    if enabled {
                        self.send_instruction(AudioInstruction::Play)
                    } else {
                        self.send_instruction(AudioInstruction::Stop)
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
                    let low = value as u16;
                    let high = memory.read_byte(address + 4) as u16;
                    let frequency = ((high & 0x07) << 8) + low;
                    log::debug!("Channel {} frequency (low): {}", channel + 1, frequency);
                }
                (channel, 0x0c) => {
                    let low = memory.read_byte(address - 4) as u16;
                    let high = value as u16;
                    let frequency = ((high & 0x07) << 8) + low;
                    log::debug!("Channel {} frequency (high): {}", channel + 1, frequency);
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
                    let wave = value & 0x07;
                    log::debug!("Channel {} waveform: {}", channel + 1, wave + 1);
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

    fn send_instruction(&mut self, instruction: AudioInstruction) {
        match &self.player {
            Some(channel) => match channel.send(instruction) {
                Ok(()) => (),
                Err(_) => self.player = None,
            },
            None => (),
        };
    }
}

pub struct AudioPlayer {
    controller: mpsc::Receiver<AudioInstruction>,
    frequency: f32,
    gain: f32,
    waveform: Option<Vec<i16>>,
    playing: bool,
    index: usize,
}

impl AudioPlayer {
    pub fn is_initialized(&self) -> bool {
        self.waveform.is_some()
    }
    pub fn init(&mut self, sample_rate: i32) {
        let sample_count = sample_rate as f32 / self.frequency;
        let samples: Vec<i16> = (0..sample_count as usize)
            .map(|i| self.gain * f32::sin((i as f32) * 2.0 * PI / sample_count))
            .map(|sample| (sample * 32768.0) as i16)
            .collect();
        self.waveform = samples.into();
    }

    pub fn play(&mut self, frames: &mut [i16]) -> Result<()> {
        self.process_instructions()?;
        if !self.playing {
            for frame in frames {
                *frame = 0;
            }
            return Ok(());
        }
        let waveform = self.waveform.as_ref().unwrap();
        let mut buf_index = 0;
        while buf_index < frames.len() {
            let batch_size = (waveform.len() - self.index).min(frames.len() - buf_index);
            frames[buf_index..buf_index + batch_size]
                .copy_from_slice(&waveform[self.index..self.index + batch_size]);
            buf_index += batch_size;
            self.index += batch_size;
            if self.index >= waveform.len() {
                self.index = 0;
            }
        }
        Ok(())
    }

    fn process_instructions(&mut self) -> Result<()> {
        loop {
            match self.controller.try_recv() {
                Ok(instruction) => self.handle_instruction(instruction),
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow::anyhow!("Lost connection to controller"))
                }
            };
        }
    }

    fn handle_instruction(&mut self, instruction: AudioInstruction) {
        match instruction {
            AudioInstruction::Play => self.playing = true,
            AudioInstruction::Stop => self.playing = false,
        };
    }
}
