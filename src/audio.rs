use crate::emulator::audio::AudioPlayer;

use anyhow::Result;
#[cfg(target_os = "android")]
use oboe::{
    AudioOutputCallback, AudioOutputStream, AudioStream, AudioStreamAsync, AudioStreamBuilder,
    DataCallbackResult, Mono, Output, PerformanceMode, SampleRateConversionQuality, SharingMode,
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
    type FrameType = (i16, Mono);

    // Implement sound data output callback
    fn on_audio_ready(
        &mut self,
        stream: &mut dyn AudioOutputStream,
        frames: &mut [i16],
    ) -> DataCallbackResult {
        // Configure our wave generator
        if !self.is_initialized() {
            self.init(stream.get_sample_rate());
        }

        match self.play(frames) {
            Ok(()) => DataCallbackResult::Continue,
            Err(err) => {
                log::error!("{}", err);
                DataCallbackResult::Stop
            }
        }
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
            .set_format::<i16>()
            // select channels configuration
            .set_channel_count::<Mono>()
            // virtual boy sample rate is mercifully low
            .set_sample_rate(41700)
            .set_sample_rate_conversion_quality(SampleRateConversionQuality::Fastest)
            // set our generator as callback
            .set_callback(player)
            // open the output stream
            .open_stream()?;
        Ok(Audio(stream))
    }

    fn play(&mut self) -> Result<()> {
        self.0.start()?;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        self.0.pause()?;
        Ok(())
    }
}

#[cfg(not(target_os = "android"))]
impl Audio {
    fn new(_player: AudioPlayer) -> Result<Audio> {
        Ok(Audio)
    }
    fn play(&mut self) -> Result<()> {
        Ok(())
    }
    fn pause(&mut self) -> Result<()> {
        Ok(())
    }
}

#[rustfmt::skip::macros(java_func)]
pub mod jni {
    use super::Audio;
    use crate::emulator::Emulator;
    use crate::{java_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::jobject;
    use jni::JNIEnv;
    use paste::paste;

    fn get_audio<'a>(env: &'a JNIEnv, this: jobject) -> jni_helpers::JavaGetResult<'a, Audio> {
        jni_helpers::java_get(env, this)
    }

    java_func!(Audio_nativeConstructor, constructor, jobject);
    fn constructor(env: &JNIEnv, this: jobject, emulator: jobject) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(env, emulator)?;
        let audio = Audio::new(emulator.get_audio_player())?;
        jni_helpers::java_init(env, this, audio)
    }

    java_func!(Audio_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<Audio>(env, this)
    }

    java_func!(Audio_nativePlay, play);
    fn play(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_audio(env, this)?;
        this.play()
    }

    java_func!(Audio_nativePause, pause);
    fn pause(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_audio(env, this)?;
        this.pause()
    }
}
