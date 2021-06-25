use crate::emulator::audio::AudioPlayer;

use anyhow::Result;
#[cfg(target_os = "android")]
use oboe::{
    AudioOutputCallback, AudioOutputStreamSafe, AudioStream, AudioStreamAsync, AudioStreamBuilder,
    DataCallbackResult, Output, PerformanceMode, SampleRateConversionQuality, SharingMode, Stereo,
};

pub fn init(sample_rate: i32, frames_per_burst: i32) {
    log::debug!(
        "Sample rate: {}, frames per burst: {}",
        sample_rate,
        frames_per_burst
    );
    #[cfg(target_os = "android")]
    {
        oboe::DefaultStreamValues::set_sample_rate(sample_rate);
        oboe::DefaultStreamValues::set_frames_per_burst(frames_per_burst);
    }
}

// Audio output callback trait implementation
#[cfg(target_os = "android")]
impl AudioOutputCallback for AudioPlayer {
    // Define type for frames which we would like to process
    type FrameType = (f32, Stereo);

    // Implement sound data output callback
    fn on_audio_ready(
        &mut self,
        _stream: &mut dyn AudioOutputStreamSafe,
        frames: &mut [(f32, f32)],
    ) -> DataCallbackResult {
        self.play(frames);
        DataCallbackResult::Continue
    }
}

#[cfg(target_os = "android")]
struct Audio(AudioStreamAsync<Output, AudioPlayer>);

#[cfg(not(target_os = "android"))]
struct Audio;

// The struct is !Send because it holds an AudioStream,
// but we only access that from one thread so it's fine
unsafe impl Send for Audio {}

#[cfg(target_os = "android")]
impl Audio {
    fn new(player: AudioPlayer) -> Result<Audio> {
        // Create playback stream
        let stream = AudioStreamBuilder::default()
            // select desired performance mode
            .set_performance_mode(PerformanceMode::LowLatency)
            // select desired sharing mode
            .set_sharing_mode(SharingMode::Shared)
            // select sound sample format
            .set_format::<f32>()
            // select channels configuration
            .set_channel_count::<Stereo>()
            // virtual boy sample rate is mercifully low
            .set_sample_rate(41667)
            .set_sample_rate_conversion_quality(SampleRateConversionQuality::Fastest)
            // set our generator as callback
            .set_callback(player)
            // open the output stream
            .open_stream()?;
        Ok(Audio(stream))
    }

    fn start(&mut self) -> Result<()> {
        self.0.request_start()?;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.0.request_stop()?;
        Ok(())
    }
}

#[cfg(not(target_os = "android"))]
impl Audio {
    fn new(_player: AudioPlayer) -> Result<Audio> {
        Ok(Audio)
    }
    fn start(&mut self) -> Result<()> {
        Ok(())
    }
    fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Settings {
    volume: f32,
    buffer_size: usize,
}

#[rustfmt::skip::macros(emulator_func)]
pub mod jni {
    use super::{Audio, Settings};
    use crate::emulator::Emulator;
    use crate::jni_helpers::EnvExtensions;
    use crate::{emulator_func, jni_helpers};
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

    emulator_func!(Audio_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(env, emulator)?;
        let settings = get_settings(&env, settings)?;
        let audio = Audio::new(emulator.get_audio_player(settings.buffer_size, settings.volume))?;
        jni_helpers::java_init(env, this, audio)
    }

    emulator_func!(Audio_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<Audio>(env, this)
    }

    emulator_func!(Audio_nativeStart, start);
    fn start(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_audio(env, this)?;
        this.start()
    }

    emulator_func!(Audio_nativeStop, stop);
    fn stop(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_audio(env, this)?;
        this.stop()
    }
}
