#![allow(non_snake_case)]

mod renderer;

use android_logger::{self, Config};
use jni::objects::JByteBuffer;
use jni::sys::{jint, jobject};
use jni::JNIEnv;
use log::{debug, error, Level};
use paste::paste;
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

macro_rules! java_func {
    ($name:ident, $func:ident) => {
        java_func!(name $name func $func params ());
    };
    ($name:ident, $func:ident, $param0:ty) => {
        java_func!(name $name func $func params (p0: $param0));
    };
    ($name:ident, $func:ident, $param0:ty, $param1:ty) => {
        java_func!(name $name func $func params (p0: $param0, p1: $param1));
    };
    (name $name:ident func $func:ident params ($($pname:ident: $ptype:ty),*)) => {
        paste! {
            #[no_mangle]
            pub unsafe extern "C" fn [<Java_com_simongellis_vvb_MainActivity_ $name>](env: JNIEnv, this: jobject $(, $pname: $ptype)*) -> () {
                let result = $func(&env, this $(, $pname)*);
                result.to_java_exception(&env);
            }
        }
    };
}

fn get_state<'a>(env: &'a JNIEnv, this: jobject) -> Result<MutexGuard<'a, Renderer>, String> {
    let res: MutexGuard<Renderer> = env
        .get_rust_field(this, STATE_FIELD)
        .map_err(|err| err.to_string())?;
    Ok(res)
}

java_func!(nativeOnCreate, on_create);
fn on_create(env: &JNIEnv, this: jobject) -> Result<(), String> {
    android_logger::init_once(Config::default().with_min_level(Level::Debug));
    debug!("Hello from vvb");

    let vm = env.get_java_vm().map_err(|err| err.to_string())?;
    Cardboard::initialize(vm.get_java_vm_pointer(), this);

    env.set_rust_field(this, STATE_FIELD, Renderer::new())
        .map_err(|err| err.to_string())?;
    Ok(())
}

java_func!(nativeOnResume, on_resume);
fn on_resume(env: &JNIEnv, this: jobject) -> Result<(), String> {
    let mut state = get_state(env, this)?;
    state.on_resume();
    Ok(())
}

java_func!(nativeSwitchViewer, switch_viewer);
fn switch_viewer(env: &JNIEnv, this: jobject) -> Result<(), String> {
    let mut state = get_state(env, this)?;
    state.switch_viewer();
    Ok(())
}

java_func!(nativeOnDestroy, on_destroy);
fn on_destroy(env: &JNIEnv, this: jobject) -> Result<(), String> {
    env.take_rust_field(this, STATE_FIELD)
        .map_err(|err| err.to_string())
}

java_func!(nativeOnSurfaceCreated, on_surface_created, JByteBuffer);
fn on_surface_created(
    env: &JNIEnv,
    this: jobject,
    title_screen: JByteBuffer,
) -> Result<(), String> {
    let buf = env
        .get_direct_buffer_address(title_screen)
        .map_err(|err| err.to_string())?;
    let mut state = get_state(env, this)?;
    state.on_surface_created(buf)
}

java_func!(nativeOnSurfaceChanged, on_surface_changed, jint, jint);
fn on_surface_changed(
    env: &JNIEnv,
    this: jobject,
    width: jint,
    height: jint,
) -> Result<(), String> {
    let mut state = get_state(env, this)?;
    state.on_surface_changed(width, height);
    Ok(())
}

java_func!(nativeOnDrawFrame, on_draw_frame);
fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<(), String> {
    let mut state = get_state(env, this)?;
    state.on_draw_frame()
}
