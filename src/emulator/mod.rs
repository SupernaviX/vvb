pub mod audio;
use audio::{AudioController, AudioPlayer};
mod cpu;
use cpu::{Event, EventHandler, CPU};
mod hardware;
use hardware::Hardware;
pub mod memory;
use memory::Memory;
pub mod video;
use video::{Eye, FrameChannel, Video};

use anyhow::Result;
use log::debug;
use std::cell::RefCell;
use std::cmp;
use std::rc::Rc;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;

pub struct Emulator {
    cycle: u64,
    tick_calls: u64,
    memory: Rc<RefCell<Memory>>,
    cpu: CPU<EmulatorEventHandler>,
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
        let cpu = CPU::new(Rc::clone(&memory), handler);
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

    pub fn get_frame_channel(&mut self) -> FrameChannel {
        self.video.borrow_mut().get_frame_channel()
    }

    pub fn get_audio_player(&mut self, buffer_size: usize, volume: i16) -> AudioPlayer {
        self.audio.borrow_mut().get_player(volume, buffer_size)
    }

    pub fn get_controller_state(&mut self) -> Arc<AtomicU16> {
        self.hardware.borrow_mut().get_controller_state()
    }

    pub fn load_game_pak(&mut self, rom: &[u8], sram: &[u8]) -> Result<()> {
        self.memory.borrow_mut().load_game_pak(rom, sram)?;
        self.reset();
        Ok(())
    }

    pub fn read_sram(&self, buffer: &mut [u8]) -> Result<()> {
        self.memory.borrow().read_sram(buffer);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.tick_calls = 0;
        self.cpu.init();
        self.audio.borrow_mut().init();
        self.video.borrow_mut().init();
        self.hardware.borrow_mut().init();
        let memory = self.memory.borrow();
        log::debug!(
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
        let video = self.video.borrow();
        video.load_frame(Eye::Left, left_eye);
        video.send_frame(Eye::Left)?;
        video.load_frame(Eye::Right, right_eye);
        video.send_frame(Eye::Right)?;
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

#[rustfmt::skip::macros(java_func)]
pub mod jni {
    use super::Emulator;
    use crate::{java_func, jni_helpers};
    use anyhow::Result;
    use jni::objects::JByteBuffer;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;

    fn get_emulator<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, Emulator> {
        jni_helpers::java_get(env, this)
    }

    java_func!(Emulator_nativeConstructor, constructor);
    fn constructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_init(env, this, Emulator::new())
    }

    java_func!(Emulator_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<Emulator>(env, this)
    }

    java_func!(Emulator_nativeLoadGamePak, load_game_pak, JByteBuffer, JByteBuffer);
    fn load_game_pak(
        env: &JNIEnv,
        this: jobject,
        rom: JByteBuffer,
        sram: JByteBuffer,
    ) -> Result<()> {
        let rom = env.get_direct_buffer_address(rom)?;
        let sram = env.get_direct_buffer_address(sram)?;
        let mut this = get_emulator(env, this)?;
        this.load_game_pak(rom, sram)
    }

    java_func!(Emulator_nativeTick, tick, jint);
    fn tick(env: &JNIEnv, this: jobject, nanoseconds: jint) -> Result<()> {
        let mut this = get_emulator(env, this)?;
        this.tick(nanoseconds as u64)
    }

    java_func!(Emulator_nativeReadSRAM, read_sram, JByteBuffer);
    fn read_sram(env: &JNIEnv, this: jobject, buffer: JByteBuffer) -> Result<()> {
        let this = get_emulator(env, this)?;
        let buffer = env.get_direct_buffer_address(buffer)?;
        this.read_sram(buffer)
    }

    java_func!(Emulator_nativeLoadImage, load_image, JByteBuffer, JByteBuffer);
    fn load_image(
        env: &JNIEnv,
        this: jobject,
        left_eye: JByteBuffer,
        right_eye: JByteBuffer,
    ) -> Result<()> {
        let left_eye = env.get_direct_buffer_address(left_eye)?;
        let right_eye = env.get_direct_buffer_address(right_eye)?;
        let this = get_emulator(env, this)?;
        this.load_image(left_eye, right_eye)
    }
}
