mod emulator;
mod jni_helpers;
mod renderer;

use android_logger::{self, Config};
use jni::sys::jobject;
use jni::JNIEnv;
use log::{debug, Level};
use paste::paste;
use renderer::Cardboard;

pub use emulator::jni::*;
pub use renderer::jni::*;

java_func!(MainActivity_nativeInitialize, init);
fn init(env: &JNIEnv, this: jobject) -> Result<(), String> {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
    debug!("Hello from vvb");

    let vm = env.get_java_vm().map_err(|err| err.to_string())?;
    Cardboard::initialize(vm.get_java_vm_pointer(), this);
    Ok(())
}
