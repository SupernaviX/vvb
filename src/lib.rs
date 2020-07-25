#![cfg(target_os="android")]
#![allow(non_snake_case)]

use jni::JNIEnv;
use jni::objects::JObject;
use jni::sys::jstring;

#[no_mangle]
pub unsafe extern fn Java_com_simongellis_vvb_MainActivity_stringFromRust(env: JNIEnv, _: JObject) -> jstring {
    let output = env.new_string("Hello from Rust!").unwrap();
    output.into_inner()
}