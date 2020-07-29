#![allow(non_snake_case)]

mod renderer;

use jni::JNIEnv;
use jni::objects::JObject;
use jni::sys::{jint};
use std::sync::MutexGuard;
use renderer::Renderer;

const STATE_FIELD: &str = "_rendererPtr";

macro_rules! safe_unwrap {
    ($action:expr, $env:expr) => {
        match $action {
            Ok(res) => Some(res),
            Err(err) => {
                $env.throw(err.to_string());
                None
            }
        }
    }
}

fn read_state<F: FnOnce(MutexGuard<Renderer>) -> ()>(env: &JNIEnv, this: &JObject, action: F) -> () {
    match env.get_rust_field(*this, STATE_FIELD) {
        Ok(res) => action(res),
        Err(err) => {
            env.throw(err.to_string()).expect("Getting a rust field threw!");
        }
    }
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnDestroy(env: JNIEnv, this: JObject) -> () {
    let _: Option<Renderer> = safe_unwrap!(env.take_rust_field(this, STATE_FIELD), &env);
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnSurfaceCreated(env: JNIEnv, this: JObject) -> () {
    safe_unwrap!(Renderer::new(), env).and_then(|state| {
        safe_unwrap!(env.set_rust_field(this, STATE_FIELD, state), env)
    });
}


#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnSurfaceChanged(env: JNIEnv, this: JObject, width: jint, height: jint) -> () {
    read_state(&env, &this, |mut state| {
        state.on_surface_changed(width, height);
    });
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnDrawFrame(env: JNIEnv, this: JObject) -> () {
    read_state(&env, &this, |state| {
        state.on_draw_frame();
    });
}
