use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

struct Controller {
    state: Arc<AtomicU16>,
}
impl Controller {
    pub fn new(state: Arc<AtomicU16>) -> Controller {
        Controller { state }
    }

    pub fn update(&mut self, state: u16) {
        self.state.store(state, Ordering::Relaxed);
    }
}

#[rustfmt::skip::macros(jni_func)]
pub mod jni {
    use super::Controller;
    use crate::emulator::Emulator;
    use crate::{jni_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;

    fn get_controller<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, Controller> {
        jni_helpers::java_get(env, this)
    }

    jni_func!(Controller_nativeConstructor, constructor, jobject);
    fn constructor(env: &JNIEnv, this: jobject, emulator: jobject) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(env, emulator)?;
        let controller = Controller::new(emulator.get_controller_state());
        jni_helpers::java_init(env, this, controller)
    }

    jni_func!(Controller_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<Controller>(env, this)
    }

    jni_func!(Controller_nativeUpdate, update, jint);
    fn update(env: &JNIEnv, this: jobject, state: jint) -> Result<()> {
        let mut this = get_controller(env, this)?;
        this.update(state as u16);
        Ok(())
    }
}
