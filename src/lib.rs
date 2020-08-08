#![allow(non_snake_case)]

mod renderer;

use android_logger::{self, Config};
use jni::errors::Error as JNIError;
use jni::objects::JByteBuffer;
use jni::sys::{jint, jobject};
use jni::JNIEnv;
use log::{debug, error, Level};
use renderer::{Cardboard, Renderer};
use std::fmt::Display;
use std::sync::MutexGuard;

const STATE_FIELD: &str = "_rendererPtr";

trait ResultExt<T, E> {
    fn to_java_exception(&self, env: &JNIEnv);
}
impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: Display,
{
    fn to_java_exception(&self, env: &JNIEnv) {
        match self {
            Ok(_) => (),
            Err(error) => {
                let str = format!("{}", error);
                error!("{}", str);
                match env.throw(str) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Throwing an error itself caused an error! {}", e);
                    }
                }
            }
        }
    }
}

fn get_state<'a>(env: &'a JNIEnv, this: jobject) -> Result<MutexGuard<'a, Renderer>, String> {
    let res: MutexGuard<Renderer> = env
        .get_rust_field(this, STATE_FIELD)
        .map_err(|err| err.to_string())?;
    Ok(res)
}
fn read_state<F: FnOnce(MutexGuard<Renderer>) -> Result<(), String>>(
    env: &JNIEnv,
    this: jobject,
    action: F,
) -> () {
    env.get_rust_field(this, STATE_FIELD)
        .map_err(|err| err.to_string())
        .and_then(|state: MutexGuard<Renderer>| action(state))
        .to_java_exception(&env);
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_simongellis_vvb_MainActivity_nativeOnCreate(
    env: JNIEnv,
    this: jobject,
) -> () {
    onCreate(&env, this).to_java_exception(&env);
}

fn onCreate(env: &JNIEnv, this: jobject) -> Result<(), String> {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
    debug!("Hello from vvb");

    let vm = env.get_java_vm().map_err(|err| err.to_string())?;
    Cardboard::initialize(vm.get_java_vm_pointer(), this);

    env.set_rust_field(this, STATE_FIELD, Renderer::new())
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_simongellis_vvb_MainActivity_nativeOnResume(
    env: JNIEnv,
    this: jobject,
) -> () {
    read_state(&env, this, |mut state| {
        state.on_resume();
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_simongellis_vvb_MainActivity_nativeSwitchViewer(
    env: JNIEnv,
    this: jobject,
) -> () {
    read_state(&env, this, |mut state| {
        state.switch_viewer();
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_simongellis_vvb_MainActivity_nativeOnDestroy(
    env: JNIEnv,
    this: jobject,
) -> () {
    let res: Result<Renderer, JNIError> = env.take_rust_field(this, STATE_FIELD);
    res.to_java_exception(&env);
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_simongellis_vvb_MainActivity_nativeOnSurfaceCreated(
    env: JNIEnv,
    this: jobject,
    title_screen: JByteBuffer,
) -> () {
    onSurfaceCreated(&env, this, title_screen).to_java_exception(&env);
}

fn onSurfaceCreated(env: &JNIEnv, this: jobject, title_screen: JByteBuffer) -> Result<(), String> {
    let buf = env
        .get_direct_buffer_address(title_screen)
        .map_err(|err| err.to_string())?;
    let mut state = get_state(env, this)?;
    state.on_surface_created(buf)
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_simongellis_vvb_MainActivity_nativeOnSurfaceChanged(
    env: JNIEnv,
    this: jobject,
    width: jint,
    height: jint,
) -> () {
    read_state(&env, this, |mut state| {
        state.on_surface_changed(width, height);
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern "C" fn Java_com_simongellis_vvb_MainActivity_nativeOnDrawFrame(
    env: JNIEnv,
    this: jobject,
) -> () {
    read_state(&env, this, |mut state| state.on_draw_frame());
}
