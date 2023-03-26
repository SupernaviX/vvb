use anyhow::Result;
use jni::objects::{JByteBuffer, JObject};
use jni::JNIEnv;
use log::error;
use std::fmt::Display;
use std::slice;
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
                    error!("Throwing an error itself caused an error! {:?}", e);
                }
            }
        }
    }
}

const POINTER_FIELD: &str = "_pointer";
pub type JavaGetResult<'a, T> = Result<MutexGuard<'a, T>>;

pub fn java_init<T: 'static + Send>(env: &JNIEnv, this: JObject, value: T) -> Result<()> {
    unsafe {
        env.set_rust_field(this, POINTER_FIELD, value)?;
    }
    Ok(())
}
pub fn java_get<'a, T: 'static + Send>(env: &'a JNIEnv, this: JObject<'a>) -> JavaGetResult<'a, T> {
    let res: MutexGuard<T> = unsafe { env.get_rust_field(this, POINTER_FIELD)? };
    Ok(res)
}
pub fn java_take<T: 'static + Send>(env: &JNIEnv, this: JObject) -> Result<()> {
    unsafe {
        env.take_rust_field(this, POINTER_FIELD)?;
    }
    Ok(())
}
pub trait EnvExtensions {
    fn get_integer_value(&self, integer: JObject) -> Result<Option<i32>>;
    fn get_int(&self, this: JObject, field: &str) -> Result<i32>;
    fn get_percent(&self, this: JObject, field: &str) -> Result<f32>;
    fn get_color(&self, this: JObject, field: &str) -> Result<(u8, u8, u8)>;
    fn get_direct_buffer(&self, buf: JByteBuffer) -> Result<&mut [u8]>;
}
impl<'a> EnvExtensions for JNIEnv<'a> {
    fn get_integer_value(&self, integer: JObject) -> Result<Option<i32>> {
        if integer.is_null() {
            return Ok(None);
        }
        let value = self.call_method(integer, "intValue", "()I", &[])?.i()?;
        Ok(Some(value))
    }
    fn get_int(&self, this: JObject, field: &str) -> Result<i32> {
        let res = self.get_field(this, field, "I")?.i()?;
        Ok(res)
    }
    fn get_percent(&self, this: JObject, field: &str) -> Result<f32> {
        let res = self.get_field(this, field, "F")?.f()?;
        Ok(res)
    }
    fn get_color(&self, this: JObject, field: &str) -> Result<(u8, u8, u8)> {
        let color = self.get_int(this, field)?;
        // android passes color as ARGB
        Ok(((color >> 16) as u8, (color >> 8) as u8, color as u8))
    }
    fn get_direct_buffer(&self, buf: JByteBuffer) -> Result<&mut [u8]> {
        let ptr = self.get_direct_buffer_address(buf)?;
        let len = self.get_direct_buffer_capacity(buf)?;
        unsafe { Ok(slice::from_raw_parts_mut(ptr, len)) }
    }
}

#[macro_export]
macro_rules! jni_func {
    ($name:ident, $func:ident) => {
        $crate::jni_func!(name $name func $func params ());
    };
    ($name:ident, $func:ident, $param0:ty) => {
        $crate::jni_func!(name $name func $func params (p0: $param0));
    };
    ($name:ident, $func:ident, $param0:ty, $param1:ty) => {
        $crate::jni_func!(name $name func $func params (p0: $param0, p1: $param1));
    };
    ($name:ident, $func:ident, $param0:ty, $param1:ty, $param2:ty) => {
        $crate::jni_func!(name $name func $func params (p0: $param0, p1: $param1, p2: $param2));
    };
    (name $name:ident func $func:ident params ($($pname:ident: $ptype:ty),*)) => {
        paste::paste! {
            #[no_mangle]
            pub unsafe extern "C" fn [<Java_com_simongellis_vvb_emulator_ $name>](env: JNIEnv, this: JObject $(, $pname: $ptype)*) {
                let result = $func(&env, this $(, $pname)*);
                $crate::jni_helpers::to_java_exception(&env, result);
            }
        }
    };
}
