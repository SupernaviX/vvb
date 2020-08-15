mod cpu;
use cpu::CPU;
mod storage;
use storage::Storage;
pub mod video;
use video::{Eye, FrameChannel, Video};

use anyhow::Result;
use log::debug;

pub struct Emulator {
    storage: Storage,
    video: Video,
}
impl Emulator {
    fn new() -> Emulator {
        Emulator {
            storage: Storage::new(),
            video: Video::new(),
        }
    }

    pub fn get_frame_channel(&mut self) -> FrameChannel {
        self.video.get_frame_channel()
    }

    pub fn load_game_pak_rom(&mut self, rom: &[u8]) -> Result<()> {
        self.storage.load_game_pak_rom(rom)?;
        log::debug!(
            "{:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x} {:04x}",
            self.storage.read_halfword(0xfffffff0),
            self.storage.read_halfword(0xfffffff2),
            self.storage.read_halfword(0xfffffff4),
            self.storage.read_halfword(0xfffffff6),
            self.storage.read_halfword(0xfffffff7),
            self.storage.read_halfword(0xfffffffa),
            self.storage.read_halfword(0xfffffffc),
            self.storage.read_halfword(0xfffffffe),
        );
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        debug!(
            "Before: PC=0x{:08x} registers={:x?}",
            self.storage.pc, self.storage.registers
        );
        CPU::run(&mut self.storage, 5)?;
        debug!(
            "After:  PC=0x{:08x} registers={:x?}",
            self.storage.pc, self.storage.registers
        );
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
    use jni::sys::jobject;
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

    java_func!(Emulator_nativeRun, run);
    fn run(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_emulator(env, this)?;
        this.run()
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
