pub mod audio;
mod cpu;
use cpu::{Event, CPU};
mod hardware;
use hardware::Hardware;
pub mod memory;
use memory::Memory;
pub mod video;
use video::{Eye, FrameChannel, Video};

use crate::emulator::audio::{AudioController, AudioPlayer};
use anyhow::Result;
use log::debug;
use std::cmp;

pub struct Emulator {
    cycle: u64,
    tick_calls: u64,
    memory: Memory,
    cpu: CPU,
    audio: AudioController,
    video: Video,
    hardware: Hardware,
}
impl Emulator {
    fn new() -> Emulator {
        Emulator {
            cycle: 0,
            tick_calls: 0,
            memory: Memory::new(),
            cpu: CPU::new(),
            audio: AudioController::new(),
            video: Video::new(),
            hardware: Hardware::new(),
        }
    }

    pub fn get_frame_channel(&mut self) -> FrameChannel {
        self.video.get_frame_channel()
    }

    pub fn get_audio_player(&mut self) -> AudioPlayer {
        self.audio.get_player()
    }

    pub fn load_game_pak(&mut self, rom: &[u8], sram: &[u8]) -> Result<()> {
        self.memory.load_game_pak(rom, sram)?;
        self.reset();
        Ok(())
    }

    pub fn read_sram(&self, buffer: &mut [u8]) -> Result<()> {
        self.memory.read_sram(buffer);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.tick_calls = 0;
        self.cpu.init();
        self.audio.init();
        self.video.init(&mut self.memory);
        self.hardware.init(&mut self.memory);
        log::debug!(
            "{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x}",
            self.memory.read_halfword(0xfffffff0),
            self.memory.read_halfword(0xfffffff2),
            self.memory.read_halfword(0xfffffff4),
            self.memory.read_halfword(0xfffffff6),
            self.memory.read_halfword(0xfffffff8),
            self.memory.read_halfword(0xfffffffa),
            self.memory.read_halfword(0xfffffffc),
            self.memory.read_halfword(0xfffffffe),
        );
    }

    pub fn tick(&mut self, nanoseconds: u64, input_state: u16) -> Result<()> {
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
        self.hardware.process_inputs(&mut self.memory, input_state);

        while self.cycle < target_cycle {
            // Find how long we can run before something interesting happens
            let next_event_cycle = cmp::min(
                target_cycle,
                cmp::min(self.hardware.next_event(), self.video.next_event()),
            );

            // Run the CPU for at least that many cycles
            // (specifically, until next_event_cycle + however long it takes to finish the current op)
            // This is safe as long as it doesn't START a new op AFTER that interesting cycle
            let cpu_result = self.cpu.run(&mut self.memory, next_event_cycle)?;

            // Have the other components catch up
            let cpu_cycle = cpu_result.cycle;
            self.audio.run(&mut self.memory, cpu_cycle);
            self.video.run(&mut self.memory, cpu_cycle)?;
            self.hardware.run(&mut self.memory, cpu_cycle);

            // If the CPU wrote somewhere interesting during execution, it would stop and return an event
            // Do what we have to do based on which event was returned
            match cpu_result.event {
                Some(Event::AudioWrite { address }) => {
                    self.audio.process_event(&mut self.memory, address);
                }
                Some(Event::DisplayControlWrite { address }) => {
                    self.video.process_event(&mut self.memory, address);
                }
                Some(Event::HardwareWrite { address }) => {
                    self.hardware.process_event(&mut self.memory, address);
                }
                _ => (),
            };

            // Components are caught up and their events are handled, now apply any pending interrupts
            if let Some(interrupt) = self.video.active_interrupt() {
                self.cpu.request_interrupt(&interrupt);
            }
            if let Some(interrupt) = self.hardware.active_interrupt() {
                self.cpu.request_interrupt(&interrupt);
            }

            self.cycle = cpu_cycle;
        }
        Ok(())
    }

    pub fn load_image(&self, left_eye: &[u8], right_eye: &[u8]) -> Result<()> {
        self.video.load_frame(Eye::Left, left_eye);
        self.video.send_frame(Eye::Left)?;
        self.video.load_frame(Eye::Right, right_eye);
        self.video.send_frame(Eye::Right)?;
        Ok(())
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
    use paste::paste;

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

    java_func!(Emulator_nativeTick, tick, jint, jint);
    fn tick(env: &JNIEnv, this: jobject, nanoseconds: jint, input_state: jint) -> Result<()> {
        let mut this = get_emulator(env, this)?;
        this.tick(nanoseconds as u64, input_state as u16)
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
