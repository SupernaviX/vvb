use jni::sys::{JavaVM, jobject};

pub fn initialize(vm: *const JavaVM, context: jobject) {
    #[cfg(target_os = "android")]
    unsafe  {
        Cardboard_initializeAndroid(vm, context);
    }
}

#[cfg(target_os = "android")]
#[link(name="cardboard_api")]
extern "C" {
    #[link_name="Cardboard_initializeAndroid"]
    pub fn Cardboard_initializeAndroid(vm: *const JavaVM, context: jobject);
}
