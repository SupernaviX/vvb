#![allow(clippy::missing_safety_doc)] // because auto-generated code
#![allow(clippy::unnecessary_wraps)] // JNI interop is easier if everything returns Result

mod audio;
mod controller;
pub mod emulator;
mod jni_helpers;
mod video;

use android_logger::{self, Config};
use anyhow::Result;
use jni::sys::{jint, jobject};
use jni::JNIEnv;
use log::{info, Level};
use video::{Cardboard, QrCode};

pub use audio::jni::*;
pub use controller::jni::*;
pub use emulator::jni::*;
pub use video::jni::*;

jni_func!(VvbLibrary_nativeInitialize, init, jobject, jint, jint);
fn init(
    env: &JNIEnv,
    _this: jobject,
    context: jobject,
    sample_rate: jint,
    frames_per_burst: jint,
) -> Result<()> {
    android_logger::init_once(Config::default().with_min_level(Level::Info));
    info!("Hello from vvb");

    let vm = env.get_java_vm()?;
    Cardboard::initialize(vm.get_java_vm_pointer(), context);

    audio::init(sample_rate, frames_per_burst);

    Ok(())
}

jni_func!(VvbLibrary_nativeChangeDeviceParams, change_device_params);
fn change_device_params(_env: &JNIEnv, _this: jobject) -> Result<()> {
    QrCode::scan_qr_code_and_save_device_params();
    Ok(())
}
