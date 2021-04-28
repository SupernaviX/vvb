use anyhow::Result;
use jni::sys::jobject;
use jni::JNIEnv;
use log::error;
use std::fmt::Display;
use std::sync::MutexGuard;

pub fn to_java_exception<T, E>(env: &JNIEnv, res: Result<T, E>)
where
    E: Display,
{
    match res {
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

const POINTER_FIELD: &str = "_pointer";
pub type JavaGetResult<'a, T> = Result<MutexGuard<'a, T>>;

pub fn java_init<T: 'static + Send>(env: &JNIEnv, this: jobject, value: T) -> Result<()> {
    env.set_rust_field(this, POINTER_FIELD, value)?;
    Ok(())
}
pub fn java_get<'a, T: 'static + Send>(env: &'a JNIEnv, this: jobject) -> JavaGetResult<'a, T> {
    let res: MutexGuard<T> = env.get_rust_field(this, POINTER_FIELD)?;
    Ok(res)
}
pub fn java_take<T: 'static + Send>(env: &JNIEnv, this: jobject) -> Result<()> {
    env.take_rust_field(this, POINTER_FIELD)?;
    Ok(())
}
pub trait EnvExtensions {
    fn get_int(&self, this: jobject, field: &str) -> Result<i32>;
    fn get_percent(&self, this: jobject, field: &str) -> Result<f32>;
    fn get_color(&self, this: jobject, field: &str) -> Result<(u8, u8, u8)>;
}
impl<'a> EnvExtensions for JNIEnv<'a> {
    fn get_int(&self, this: jobject, field: &str) -> Result<i32> {
        let res = self.get_field(this, field, "I")?.i()?;
        Ok(res)
    }
    fn get_percent(&self, this: jobject, field: &str) -> Result<f32> {
        let res = self.get_int(this, field)?;
        Ok((res as f32) / 100.0)
    }
    fn get_color(&self, this: jobject, field: &str) -> Result<(u8, u8, u8)> {
        let color = self.get_int(this, field)?;
        // android passes color as ARGB
        Ok(((color >> 16) as u8, (color >> 8) as u8, color as u8))
    }
}

#[macro_export]
macro_rules! java_func {
    ($name:ident, $func:ident) => {
        crate::java_func!(name $name func $func params ());
    };
    ($name:ident, $func:ident, $param0:ty) => {
        crate::java_func!(name $name func $func params (p0: $param0));
    };
    ($name:ident, $func:ident, $param0:ty, $param1:ty) => {
        crate::java_func!(name $name func $func params (p0: $param0, p1: $param1));
    };
    (name $name:ident func $func:ident params ($($pname:ident: $ptype:ty),*)) => {
        paste::paste! {
            #[no_mangle]
            pub unsafe extern "C" fn [<Java_com_simongellis_vvb_ $name>](env: JNIEnv, this: jobject $(, $pname: $ptype)*) {
                let result = $func(&env, this $(, $pname)*);
                crate::jni_helpers::to_java_exception(&env, result);
            }
        }
    };
}

#[macro_export]
macro_rules! emulator_func {
    ($name:ident, $( $params:tt ),+) => {
        paste::paste! {
            crate::java_func!([<emulator_ $name>], $( $params ),+ );
        }
    }
}
