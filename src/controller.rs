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
    use crate::emulator::jni::get_emulator;
    use crate::jni_func;
    use crate::jni_helpers::{JavaBinding, JavaGetResult};
    use anyhow::Result;
    use jni::objects::JObject;
    use jni::sys::jint;
    use jni::JNIEnv;

    static CONTROLLER_BINDING: JavaBinding<Controller> = JavaBinding::new();

    fn get_controller<'a>(env: &'a mut JNIEnv, this: JObject<'a>) -> JavaGetResult<'a, Controller> {
        CONTROLLER_BINDING.get_value(env, this)
    }

    jni_func!(Controller_nativeConstructor, constructor, JObject);
    fn constructor(env: &mut JNIEnv, this: JObject, emulator: JObject) -> Result<()> {
        let controller = {
            let mut emulator = get_emulator(env, emulator)?;
            Controller::new(emulator.claim_controller_state())
        };
        CONTROLLER_BINDING.init_value(env, this, controller)
    }

    jni_func!(Controller_nativeDestructor, destructor);
    fn destructor(env: &mut JNIEnv, this: JObject) -> Result<()> {
        CONTROLLER_BINDING.drop_value(env, this)
    }

    jni_func!(Controller_nativeUpdate, update, jint);
    fn update(env: &mut JNIEnv, this: JObject, state: jint) -> Result<()> {
        let mut this = get_controller(env, this)?;
        this.update(state as u16);
        Ok(())
    }
}
