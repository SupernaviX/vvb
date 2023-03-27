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
    use crate::emulator::jni::get_emulator;
    use crate::jni_helpers::{JavaBinding, JavaGetResult};
    use crate::{jni_func, EnvExtensions};
    use anyhow::Result;
    use jni::objects::JObject;
    use jni::JNIEnv;

    static AUDIO_BINDING: JavaBinding<Audio> = JavaBinding::new();

    fn get_audio<'a>(env: &'a mut JNIEnv, this: JObject<'a>) -> JavaGetResult<'a, Audio> {
        AUDIO_BINDING.get_value(env, this)
    }

    fn get_settings<'a>(env: &mut JNIEnv<'a>, this: JObject<'a>) -> Result<Settings> {
        let volume = env.get_percent(&this, "volume")?;
        let buffer_size = env.get_int(&this, "bufferSize")?;

        Ok(Settings {
            volume,
            buffer_size: buffer_size as usize,
        })
    }

    jni_func!(Audio_nativeConstructor, constructor, JObject<'a>, JObject<'a>);
    fn constructor<'a>(
        env: &mut JNIEnv<'a>,
        this: JObject<'a>,
        emulator: JObject<'a>,
        settings: JObject<'a>,
    ) -> Result<()> {
        let settings = get_settings(env, settings)?;
        let audio = {
            let mut emulator = get_emulator(env, emulator)?;
            Audio::new(emulator.claim_audio_player(settings.buffer_size, settings.volume))?
        };
        AUDIO_BINDING.init_value(env, this, audio)
    }

    jni_func!(Audio_nativeDestructor, destructor);
    fn destructor(env: &mut JNIEnv, this: JObject) -> Result<()> {
        AUDIO_BINDING.drop_value(env, this)
    }

    jni_func!(Audio_nativeStart, start);
    fn start(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_audio(env, this)?;
        this.start()
    }

    jni_func!(Audio_nativeStop, stop);
    fn stop(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_audio(env, this)?;
        this.stop()
    }
}
