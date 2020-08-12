mod emulator;
mod jni_helpers;
mod renderer;

use android_logger::{self, Config};
use anyhow::Result;
use jni::sys::jobject;
use jni::JNIEnv;
use log::{debug, Level};
use paste::paste;
use renderer::Cardboard;

pub use emulator::jni::*;
pub use renderer::jni::*;

java_func!(MainActivity_nativeInitialize, init);
fn init(env: &JNIEnv, this: jobject) -> Result<()> {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
    debug!("Hello from vvb");

    let vm = env.get_java_vm()?;
    Cardboard::initialize(vm.get_java_vm_pointer(), this);
    Ok(())
}
