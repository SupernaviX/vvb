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

#[macro_export]
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
            pub unsafe extern "C" fn [<Java_com_simongellis_vvb_ $name>](env: JNIEnv, this: jobject $(, $pname: $ptype)*) {
                let result = $func(&env, this $(, $pname)*);
                crate::jni_helpers::to_java_exception(&env, result);
            }
        }
    };
}
