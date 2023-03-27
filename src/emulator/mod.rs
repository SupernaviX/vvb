pub mod audio;
use audio::{AudioController, AudioPlayer};
mod cpu;
use cpu::{Cpu, Event, EventHandler};
mod hardware;
use hardware::Hardware;
pub mod memory;
use memory::{Memory, Region};
mod state;
use state::{GlobalState, SaveStateData};
pub mod video;
use video::{Eye, FrameBufferConsumers, Video};

use anyhow::Result;
use log::{debug, info};
use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;

pub struct Emulator {
    cycle: u64,
    tick_calls: u64,
    memory: Rc<RefCell<Memory>>,
    cpu: Cpu<EmulatorEventHandler>,
    audio: Rc<RefCell<AudioController>>,
    video: Rc<RefCell<Video>>,
    hardware: Rc<RefCell<Hardware>>,
}
unsafe impl Send for Emulator {} // Never actually sent to other threads so it's fine
impl Emulator {
    fn new() -> Emulator {
        let memory = Rc::new(RefCell::new(Memory::new()));
        let audio = Rc::new(RefCell::new(AudioController::new(Rc::clone(&memory))));
        let video = Rc::new(RefCell::new(Video::new(Rc::clone(&memory))));
        let hardware = Rc::new(RefCell::new(Hardware::new(Rc::clone(&memory))));
        let handler = EmulatorEventHandler {
            audio: Rc::clone(&audio),
            video: Rc::clone(&video),
            hardware: Rc::clone(&hardware),
        };
        let cpu = Cpu::new(Rc::clone(&memory), handler);
        Emulator {
            cycle: 0,
            tick_calls: 0,
            memory,
            cpu,
            audio,
            video,
            hardware,
        }
    }

    pub fn claim_frame_buffer_consumers(&mut self) -> FrameBufferConsumers {
        self.video.borrow_mut().claim_frame_buffer_consumers()
    }

    pub fn claim_audio_player(&mut self, buffer_size: usize, volume: f32) -> AudioPlayer {
        self.audio.borrow_mut().claim_player(volume, buffer_size)
    }

    pub fn claim_controller_state(&mut self) -> Arc<AtomicU16> {
        self.hardware.borrow_mut().claim_controller_state()
    }

    pub fn load_game_pak(&mut self, rom: &[u8], sram: &[u8]) -> Result<()> {
        self.memory.borrow_mut().load_game_pak(rom, sram)?;
        self.reset();
        info!("Game pak successfully loaded!");
        Ok(())
    }

    pub fn unload_game_pak(&mut self) {
        self.memory.borrow_mut().unload_game_pak();
        self.reset();
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.tick_calls = 0;
        info!("Resetting CPU module...");
        self.cpu.init();
        info!("Resetting audio module...");
        self.audio.borrow_mut().init();
        info!("Resetting video module...");
        self.video.borrow_mut().init();
        info!("Resetting hardware module...");
        self.hardware.borrow_mut().init();
        let memory = self.memory.borrow();
        debug!(
            "{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x}",
            memory.read_halfword(0xfffffff0),
            memory.read_halfword(0xfffffff2),
            memory.read_halfword(0xfffffff4),
            memory.read_halfword(0xfffffff6),
            memory.read_halfword(0xfffffff8),
            memory.read_halfword(0xfffffffa),
            memory.read_halfword(0xfffffffc),
            memory.read_halfword(0xfffffffe),
        );
    }

    pub fn read_sram(&self, buffer: &mut [u8]) -> Result<()> {
        if let Some(sram) = self.memory.borrow().read_region(Region::Sram) {
            buffer.copy_from_slice(sram);
        }
        Ok(())
    }

    pub fn save_state(&self, filename: &str) -> Result<()> {
        const REGIONS: [Region; 5] = [
            Region::Vram,
            Region::Audio,
            Region::Hardware,
            Region::Dram,
            Region::Sram,
        ];

        let memory = self.memory.borrow();
        let video = self.video.borrow();
        let hardware = self.hardware.borrow();
        let audio = self.audio.borrow();

        let mut data = vec![SaveStateData::Global(GlobalState {
            cycle: self.cycle,
            tick_calls: self.tick_calls,
        })];
        let memory_state = REGIONS.iter().copied().map(|region| {
            SaveStateData::Memory(region, memory.read_region(region).unwrap().to_vec())
        });
        data.extend(memory_state);
        data.push(SaveStateData::Cpu(Box::new(self.cpu.save_state())));
        data.push(SaveStateData::Audio(Box::new(audio.save_state())));
        data.push(SaveStateData::Video(video.save_state()));
        data.push(SaveStateData::Hardware(hardware.save_state()));

        state::save_state(filename, &data)
    }

    pub fn load_state(&mut self, filename: &str) -> Result<()> {
        let mut memory = self.memory.borrow_mut();
        let mut video = self.video.borrow_mut();
        let mut hardware = self.hardware.borrow_mut();
        let mut audio = self.audio.borrow_mut();
        let data = state::load_state(filename)?;
        for datum in data {
            match datum {
                SaveStateData::Global(state) => {
                    self.cycle = state.cycle;
                    self.tick_calls = state.tick_calls;
                }
                SaveStateData::Memory(region, data) => {
                    memory.write_region(region).unwrap().copy_from_slice(&data)
                }
                SaveStateData::Cpu(state) => self.cpu.load_state(&state),
                SaveStateData::Audio(state) => audio.load_state(&state),
                SaveStateData::Video(state) => video.load_state(&state),
                SaveStateData::Hardware(state) => hardware.load_state(&state),
            }
        }
        Ok(())
    }

    pub fn tick(&mut self, nanoseconds: u64) -> Result<()> {
        let cycles = nanoseconds / 50;
        let target_cycle = self.cycle + cycles;

        // Log average tick size every 5 seconds
        self.tick_calls += 1;
        let old_sec = self.cycle / 100_000_000;
        let new_sec = target_cycle / 100_000_000;
        if old_sec != new_sec {
            debug!("Current PC: 0x{:08x}", self.cpu.pc);
            debug!("Current PSW: 0x{:08x}", self.cpu.sys_registers[5]);
            debug!("Cycles per tick: {}", target_cycle / self.tick_calls);
        }

        while self.cycle < target_cycle {
            // Find how long we can run before something interesting happens
            let next_event_cycle = cmp::min(
                target_cycle,
                cmp::min(
                    self.hardware.borrow().next_event(),
                    self.video.borrow().next_event(),
                ),
            );

            // Run the CPU for at least that many cycles
            // (specifically, until next_event_cycle + however long it takes to finish the current op)
            // This is safe as long as it doesn't START a new op AFTER that interesting cycle
            let cpu_result = self.cpu.run(next_event_cycle)?;

            // Have the other components catch up
            let cpu_cycle = cpu_result.cycle;
            self.audio.borrow_mut().run(cpu_cycle);
            self.video.borrow_mut().run(cpu_cycle)?;
            self.hardware.borrow_mut().run(cpu_cycle);

            // Components are caught up and their events are handled, now apply any pending interrupts
            if let Some(exception) = self.video.borrow().active_interrupt() {
                self.cpu.raise_exception(exception);
            }
            if let Some(exception) = self.hardware.borrow().active_interrupt() {
                self.cpu.raise_exception(exception);
            }

            self.cycle = cpu_cycle;
        }
        Ok(())
    }

    pub fn load_image(&self, left_eye: &[u8], right_eye: &[u8]) -> Result<()> {
        let video = self.video.borrow_mut();
        video.load_and_send_frame(Eye::Left, left_eye);
        video.load_and_send_frame(Eye::Right, right_eye);
        Ok(())
    }
}

struct EmulatorEventHandler {
    audio: Rc<RefCell<AudioController>>,
    video: Rc<RefCell<Video>>,
    hardware: Rc<RefCell<Hardware>>,
}
impl EventHandler for EmulatorEventHandler {
    fn handle(&mut self, event: Event, cycle: u64) -> Result<bool> {
        // If the CPU wrote somewhere interesting during execution, it would stop and return an event
        // Do what we have to do based on which event was returned
        match event {
            Event::AudioWrite { address } => {
                let mut audio = self.audio.borrow_mut();
                audio.run(cycle);
                audio.process_event(address);
                Ok(true)
            }
            Event::DisplayControlWrite { address } => {
                let mut video = self.video.borrow_mut();
                video.run(cycle)?;
                Ok(video.process_event(address))
            }
            Event::HardwareWrite { address } => {
                let mut hardware = self.hardware.borrow_mut();
                hardware.run(cycle);
                Ok(hardware.process_event(address))
            }
            _ => Ok(false),
        }
    }
}

#[rustfmt::skip::macros(jni_func)]
pub mod jni {
    use super::Emulator;
    use crate::jni_helpers::{JavaBinding, JavaGetResult};
    use crate::{jni_func, EnvExtensions};
    use anyhow::Result;
    use jni::objects::{JByteBuffer, JObject, JString};
    use jni::sys::jint;
    use jni::JNIEnv;
    use log::info;
    use std::convert::TryInto;

    static EMULATOR_BINDING: JavaBinding<Emulator> = JavaBinding::new();

    pub fn get_emulator<'a>(env: &'a mut JNIEnv, this: JObject<'a>) -> JavaGetResult<'a, Emulator> {
        EMULATOR_BINDING.get_value(env, this)
    }

    jni_func!(Emulator_nativeConstructor, constructor);
    fn constructor(env: &mut JNIEnv, this: JObject) -> Result<()> {
        EMULATOR_BINDING.init_value(env, this, Emulator::new())
    }

    jni_func!(Emulator_nativeDestructor, destructor);
    fn destructor(env: &mut JNIEnv, this: JObject) -> Result<()> {
        EMULATOR_BINDING.drop_value(env, this)
    }

    jni_func!(Emulator_nativeLoadGamePak, load_game_pak, JByteBuffer, JByteBuffer);
    fn load_game_pak<'a>(
        env: &'a mut JNIEnv,
        this: JObject<'a>,
        rom: JByteBuffer,
        sram: JByteBuffer,
    ) -> Result<()> {
        info!("Loading game pak");
        let rom = env.get_direct_buffer(rom)?;
        info!("ROM length: {} byte(s)", rom.len());
        let sram = env.get_direct_buffer(sram)?;
        info!("SRAM length: {} byte(s)", sram.len());
        let mut this = get_emulator(env, this)?;
        info!("Beginning game pak load...");
        this.load_game_pak(rom, sram)
    }

    jni_func!(Emulator_nativeUnloadGamePak, unload_game_pak);
    fn unload_game_pak(env: &mut JNIEnv, this: JObject) -> Result<()> {
        info!("Unloading game pak");
        let mut this = get_emulator(env, this)?;
        this.unload_game_pak();
        Ok(())
    }

    jni_func!(Emulator_nativeReset, reset);
    fn reset(env: &mut JNIEnv, this: JObject) -> Result<()> {
        info!("Resetting game");
        let mut this = get_emulator(env, this)?;
        this.reset();
        Ok(())
    }

    jni_func!(Emulator_nativeTick, tick, jint);
    fn tick(env: &mut JNIEnv, this: JObject, nanoseconds: jint) -> Result<()> {
        let mut this = get_emulator(env, this)?;
        this.tick(nanoseconds as u64)
    }

    jni_func!(Emulator_nativeReadSRAM, read_sram, JByteBuffer);
    fn read_sram(env: &mut JNIEnv, this: JObject, buffer: JByteBuffer) -> Result<()> {
        let buffer = env.get_direct_buffer(buffer)?;
        let this = get_emulator(env, this)?;
        this.read_sram(buffer)
    }

    jni_func!(Emulator_nativeSaveState, save_state, JString);
    fn save_state(env: &mut JNIEnv, this: JObject, filename: JString) -> Result<()> {
        info!("Saving...");
        let filename: String = env.get_string(&filename)?.try_into()?;
        let this = get_emulator(env, this)?;
        this.save_state(&filename)?;
        info!("Saved!");
        Ok(())
    }

    jni_func!(Emulator_nativeLoadState, load_state, JString);
    fn load_state(env: &mut JNIEnv, this: JObject, filename: JString) -> Result<()> {
        info!("Loading...");
        let filename: String = env.get_string(&filename)?.try_into()?;
        let mut this = get_emulator(env, this)?;
        this.load_state(&filename)?;
        info!("Loaded!");
        Ok(())
    }

    jni_func!(Emulator_nativeLoadImage, load_image, JByteBuffer, JByteBuffer);
    fn load_image(
        env: &mut JNIEnv,
        this: JObject,
        left_eye: JByteBuffer,
        right_eye: JByteBuffer,
    ) -> Result<()> {
        let left_eye = env.get_direct_buffer(left_eye)?;
        let right_eye = env.get_direct_buffer(right_eye)?;
        let this = get_emulator(env, this)?;
        this.load_image(left_eye, right_eye)
    }
}
