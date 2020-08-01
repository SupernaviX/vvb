#![allow(non_snake_case)]

mod renderer;

use jni::JNIEnv;
use jni::objects::JObject;
use jni::sys::{jint};
use jni::errors::Error as JNIError;
use std::sync::MutexGuard;
use renderer::Renderer;
use log::{debug,Level};
use android_logger::{self,Config};

const STATE_FIELD: &str = "_rendererPtr";

trait ResultExt<T, E> {
    fn to_java_exception(&self, env: &JNIEnv);
}
impl<T, E> ResultExt<T, E> for Result<T, E> where E: ToString {
    fn to_java_exception(&self, env: &JNIEnv) {
        match self {
            Ok(_) => (),
            Err(error) => {
                env.throw(error.to_string()).expect("Throwing an error itself caused an error");
            }
        }
    }
}

fn read_state<F: FnOnce(MutexGuard<Renderer>) -> Result<(), String>>(env: &JNIEnv, this: &JObject, action: F) -> () {
    env.get_rust_field(*this, STATE_FIELD)
        .map_err(|err| { err.to_string() })
        .and_then(|state: MutexGuard<Renderer>| { action(state) })
        .to_java_exception(&env);
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnCreate() -> () {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
    debug!("Hello from vvb");
}


#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnDestroy(env: JNIEnv, this: JObject) -> () {
    let res: Result<Renderer, JNIError> = env.take_rust_field(this, STATE_FIELD);
    res.to_java_exception(&env);
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnSurfaceCreated(env: JNIEnv, this: JObject) -> () {
    Renderer::new()
        .and_then(|state| {
            env.set_rust_field(this, STATE_FIELD, state)
                .map_err(|err| { err.to_string() })
        })
        .to_java_exception(&env);
}


#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnSurfaceChanged(env: JNIEnv, this: JObject, width: jint, height: jint) -> () {
    read_state(&env, &this, |mut state| {
        state.on_surface_changed(width, height);
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnDrawFrame(env: JNIEnv, this: JObject) -> () {
    read_state(&env, &this, |state| {
        state.on_draw_frame()
    });
}
