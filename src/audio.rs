#[cfg(target_os = "android")]
mod oboe;
#[cfg(target_os = "android")]
type Audio = oboe::OboeAudio;

#[cfg(not(target_os = "android"))]
mod noop;
#[cfg(not(target_os = "android"))]
type Audio = noop::NoopAudio;

pub fn init(sample_rate: Option<i32>, frames_per_burst: Option<i32>) {
    log::info!(
        "Sample rate: {:?}, frames per burst: {:?}",
        sample_rate,
        frames_per_burst
    );
    #[cfg(target_os = "android")]
    oboe::init(sample_rate, frames_per_burst);
}

#[derive(Debug)]
pub struct Settings {
    volume: f32,
    buffer_size: usize,
}

#[rustfmt::skip::macros(jni_func)]
pub mod jni {
    use super::{Audio, Settings};
    use crate::emulator::Emulator;
    use crate::jni_helpers::EnvExtensions;
    use crate::{jni_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::jobject;
    use jni::JNIEnv;

    fn get_audio<'a>(env: &'a JNIEnv, this: jobject) -> jni_helpers::JavaGetResult<'a, Audio> {
        jni_helpers::java_get(env, this)
    }

    fn get_settings(env: &JNIEnv, this: jobject) -> Result<Settings> {
        let volume = env.get_percent(this, "volume")?;
        let buffer_size = env.get_int(this, "bufferSize")?;

        Ok(Settings {
            volume,
            buffer_size: buffer_size as usize,
        })
    }

    jni_func!(Audio_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(env, emulator)?;
        let settings = get_settings(env, settings)?;
        let audio = Audio::new(emulator.claim_audio_player(settings.buffer_size, settings.volume))?;
        jni_helpers::java_init(env, this, audio)
    }

    jni_func!(Audio_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<Audio>(env, this)
    }

    jni_func!(Audio_nativeStart, start);
    fn start(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_audio(env, this)?;
        this.start()
    }

    jni_func!(Audio_nativeStop, stop);
    fn stop(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_audio(env, this)?;
        this.stop()
    }
}
