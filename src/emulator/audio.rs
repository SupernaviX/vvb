use crate::emulator::memory::Memory;
use anyhow::Result;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;

#[derive(Debug)]
enum AudioInstruction {
    Play { waveforms: Option<[[i16; 32]; 5]> },
    Stop,
}

pub struct AudioController {
    player: Option<mpsc::Sender<AudioInstruction>>,
    waveforms: [[i16; 32]; 5],
    waveforms_stale: bool,
}

impl AudioController {
    pub fn new() -> AudioController {
        AudioController {
            player: None,
            waveforms: [[0; 32]; 5],
            waveforms_stale: false,
        }
    }

    pub fn get_player(&mut self) -> AudioPlayer {
        let (tx, rx) = mpsc::channel();
        self.player = Some(tx);
        AudioPlayer {
            controller: rx,
            waveforms: self.waveforms,
            playing: false,
            index: 0,
        }
    }

    pub fn process_event(&mut self, memory: &mut Memory, address: usize) {
        let value = memory.read_byte(address);
        if address < 0x01000280 {
            let rel_addr = address - 0x01000000;
            let waveform = rel_addr / 128;
            let index = (rel_addr % 128) / 4;
            log::debug!("Waveform {}[{}] := 0x{:02x}", waveform + 1, index, value);
            self.waveforms[waveform][index] = (((value as i16) & 0x3f) - 32) * 32;
            self.waveforms_stale = true;
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
                        self.send_instruction(AudioInstruction::Play {
                            waveforms: if self.waveforms_stale {
                                Some(self.waveforms)
                            } else {
                                None
                            },
                        });
                        self.waveforms_stale = false;
                    } else {
                        self.send_instruction(AudioInstruction::Stop);
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
    waveforms: [[i16; 32]; 5],
    playing: bool,
    index: usize,
}

impl AudioPlayer {
    pub fn play(&mut self, frames: &mut [i16]) -> Result<()> {
        self.process_instructions()?;
        if !self.playing {
            for frame in frames {
                *frame = 0;
            }
            return Ok(());
        }
        let waveform = &self.waveforms[0];
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
            AudioInstruction::Play { waveforms } => {
                if let Some(waveforms) = waveforms {
                    self.waveforms = waveforms;
                }
                self.playing = true
            }
            AudioInstruction::Stop => self.playing = false,
        };
    }
}
