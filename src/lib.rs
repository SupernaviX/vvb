#![feature(const_default_impls)]
#![feature(const_trait_impl)]
#![allow(clippy::missing_safety_doc)] // because auto-generated code
#![allow(clippy::unnecessary_wraps)] // JNI interop is easier if everything returns Result

mod audio;
mod controller;
pub mod emulator;
mod jni_helpers;
mod video;

use android_logger::{self, Config};
use anyhow::Result;
use jni::objects::JObject;
use jni::JNIEnv;
use log::{info, LevelFilter};
use video::{Cardboard, QrCode};

use crate::jni_helpers::EnvExtensions;
pub use audio::jni::*;
pub use controller::jni::*;
pub use emulator::jni::*;
pub use video::jni::*;

jni_func!(
    VvbLibrary_nativeInitialize,
    init,
    JObject<'a>,
    JObject<'a>,
    JObject<'a>
);
fn init<'a>(
    env: &mut JNIEnv<'a>,
    _this: JObject,
    context: JObject<'a>,
    sample_rate: JObject<'a>,
    frames_per_burst: JObject<'a>,
) -> Result<()> {
    android_logger::init_once(Config::default().with_max_level(LevelFilter::Info));
    info!("Hello from vvb");

    let vm = env.get_java_vm()?;
    Cardboard::initialize(vm.get_java_vm_pointer(), context);

    let sample_rate = env.get_integer_value(sample_rate)?;
    let frames_per_burst = env.get_integer_value(frames_per_burst)?;

    audio::init(sample_rate, frames_per_burst);

    Ok(())
}

jni_func!(VvbLibrary_nativeChangeDeviceParams, change_device_params);
fn change_device_params(_env: &mut JNIEnv, _this: JObject) -> Result<()> {
    QrCode::scan_qr_code_and_save_device_params();
    Ok(())
}
