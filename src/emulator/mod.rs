mod cpu;
use cpu::{Event, CPU};
mod hardware;
use hardware::Hardware;
mod storage;
use storage::Storage;
pub mod video;
use video::{Eye, FrameChannel, Video};

use anyhow::Result;
use log::debug;

pub struct Emulator {
    cycle: u64,
    tick_calls: u64,
    storage: Storage,
    cpu: CPU,
    video: Video,
    hardware: Hardware,
}
impl Emulator {
    fn new() -> Emulator {
        Emulator {
            cycle: 0,
            tick_calls: 0,
            storage: Storage::new(),
            cpu: CPU::new(),
            video: Video::new(),
            hardware: Hardware::new(),
        }
    }

    pub fn get_frame_channel(&mut self) -> FrameChannel {
        self.video.get_frame_channel()
    }

    pub fn load_game_pak_rom(&mut self, rom: &[u8]) -> Result<()> {
        self.storage.load_game_pak_rom(rom)?;
        self.reset();
        Ok(())
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.tick_calls = 0;
        self.cpu.reset();
        self.video.init(&mut self.storage);
        self.hardware.reset();
        log::debug!(
            "{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x}",
            self.storage.read_halfword(0xfffffff0),
            self.storage.read_halfword(0xfffffff2),
            self.storage.read_halfword(0xfffffff4),
            self.storage.read_halfword(0xfffffff6),
            self.storage.read_halfword(0xfffffff8),
            self.storage.read_halfword(0xfffffffa),
            self.storage.read_halfword(0xfffffffc),
            self.storage.read_halfword(0xfffffffe),
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
            debug!("Cycles per tick: {}", target_cycle / self.tick_calls);
        }
        self.hardware.process_inputs(&mut self.storage, input_state);

        while self.cycle < target_cycle {
            // Find how long we can run before something interesting happens
            let next_event_cycle = std::cmp::min(self.hardware.next_event(), target_cycle);

            // Run the CPU for at least that many cycles
            // (specifically, until next_event_cycle + however long it takes to finish the current op)
            // This is safe as long as it doesn't START a new op AFTER that interesting cycle
            let cpu_result = self.cpu.run(&mut self.storage, next_event_cycle)?;

            // Have the other components catch up
            let cpu_cycle = cpu_result.cycle;
            self.video.run(&mut self.storage, cpu_cycle)?;
            self.hardware.run(&mut self.storage, cpu_cycle);

            // If the CPU wrote somewhere interesting during execution, it would stop and return an event
            // Do what we have to do based on which event was returned
            match cpu_result.event {
                Some(Event::HardwareAccess { address }) => {
                    self.hardware.process_event(&mut self.storage, address);
                }
                None => (),
            };
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

    java_func!(Emulator_nativeLoadGamePakRom, load_game_pak_rom, JByteBuffer);
    fn load_game_pak_rom(env: &JNIEnv, this: jobject, rom: JByteBuffer) -> Result<()> {
        let rom = env.get_direct_buffer_address(rom)?;
        let mut this = get_emulator(env, this)?;
        this.load_game_pak_rom(rom)
    }

    java_func!(Emulator_nativeTick, tick, jint, jint);
    fn tick(env: &JNIEnv, this: jobject, nanoseconds: jint, input_state: jint) -> Result<()> {
        let mut this = get_emulator(env, this)?;
        this.tick(nanoseconds as u64, input_state as u16)
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
