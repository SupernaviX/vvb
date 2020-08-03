#![allow(non_snake_case)]

mod renderer;

use jni::JNIEnv;
use jni::sys::{jint,jobject};
use jni::errors::Error as JNIError;
use std::sync::MutexGuard;
use renderer::{Renderer, Cardboard};
use log::{debug,error,Level};
use android_logger::{self,Config};
use jni::objects::JByteBuffer;

const STATE_FIELD: &str = "_rendererPtr";

trait ResultExt<T, E> {
    fn to_java_exception(&self, env: &JNIEnv);
}
impl<T, E> ResultExt<T, E> for Result<T, E> where E: ToString {
    fn to_java_exception(&self, env: &JNIEnv) {
        match self {
            Ok(_) => (),
            Err(error) => {
                let str = error.to_string();
                error!("{}", str);
                match env.throw(str) {
                    Ok(_) => (),
                    Err(e) => {
                        error!("Throwing an error itself caused an error! {}", e.to_string());
                    }
                }
            }
        }
    }
}

fn read_state<F: FnOnce(MutexGuard<Renderer>) -> Result<(), String>>(env: &JNIEnv, this: jobject, action: F) -> () {
    env.get_rust_field(this, STATE_FIELD)
        .map_err(|err| { err.to_string() })
        .and_then(|state: MutexGuard<Renderer>| { action(state) })
        .to_java_exception(&env);
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnCreate(env: JNIEnv, this: jobject) -> () {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
    debug!("Hello from vvb");
    env.get_java_vm()
        .map(|vm| { Cardboard::initialize(vm.get_java_vm_pointer(), this) })
        .to_java_exception(&env);
}


#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnDestroy(env: JNIEnv, this: jobject) -> () {
    let res: Result<Renderer, JNIError> = env.take_rust_field(this, STATE_FIELD);
    res.to_java_exception(&env);
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnSurfaceCreated(env: JNIEnv, this: jobject, title_screen: JByteBuffer) -> () {
    env.get_direct_buffer_address(title_screen).map_err(|err| { err.to_string() })
        .and_then(|title_screen| { Renderer::new(title_screen) })
        .and_then(|state| {
            env.set_rust_field(this, STATE_FIELD, state).map_err(|err| { err.to_string() })
        })
        .to_java_exception(&env);
}


#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnSurfaceChanged(env: JNIEnv, this: jobject, width: jint, height: jint) -> () {
    read_state(&env, this, |mut state| {
        state.on_surface_changed(width, height);
        Ok(())
    });
}

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_nativeOnDrawFrame(env: JNIEnv, this: jobject) -> () {
    read_state(&env, this, |state| {
        state.on_draw_frame()
    });
}
