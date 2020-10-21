use anyhow::Result;
#[cfg(target_os = "android")]
use oboe::{
    AudioOutputCallback, AudioOutputStream, AudioStream, AudioStreamAsync, AudioStreamBuilder,
    DataCallbackResult, Mono, Output, PerformanceMode, SampleRateConversionQuality, SharingMode,
};
use std::f32::consts::PI;

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

// Structure for sound generator
struct SineWave {
    frequency: f32,
    gain: f32,
    waveform: Option<Vec<i16>>,
    index: usize,
}

#[allow(dead_code)]
impl SineWave {
    fn init(&mut self, sample_rate: i32) {
        let sample_count = sample_rate as f32 / self.frequency;
        let samples: Vec<i16> = (0..sample_count as usize)
            .map(|i| self.gain * f32::sin((i as f32) * 2.0 * PI / sample_count))
            .map(|sample| (sample * 32768.0) as i16)
            .collect();
        self.waveform = samples.into();
    }

    fn play(&mut self, frames: &mut [i16]) {
        let waveform = self.waveform.as_ref().unwrap();
        let mut buf_index = 0;
        while buf_index < frames.len() {
            let batch_size = (waveform.len() - self.index).min(frames.len() - buf_index);
            frames[buf_index..buf_index + batch_size]
                .copy_from_slice(&waveform[self.index..self.index + batch_size]);
            buf_index += batch_size;
            self.index += batch_size;
            if self.index >= waveform.len() {
                self.index = 0;
            }
        }
    }
}

// Default constructor for sound generator
impl Default for SineWave {
    fn default() -> Self {
        Self {
            frequency: 440.0,
            gain: 0.5,
            waveform: None,
            index: 0,
        }
    }
}

// Audio output callback trait implementation
#[cfg(target_os = "android")]
impl AudioOutputCallback for SineWave {
    // Define type for frames which we would like to process
    type FrameType = (i16, Mono);

    // Implement sound data output callback
    fn on_audio_ready(
        &mut self,
        stream: &mut dyn AudioOutputStream,
        frames: &mut [i16],
    ) -> DataCallbackResult {
        // Configure our wave generator
        if self.waveform.is_none() {
            self.init(stream.get_sample_rate());
        }

        self.play(frames);

        // Notify the oboe that stream is continued
        DataCallbackResult::Continue
    }
}

#[cfg(target_os = "android")]
struct Audio(AudioStreamAsync<Output, SineWave>);

#[cfg(not(target_os = "android"))]
struct Audio;

// The struct is !Send because it holds an AudioStream,
// but we only access that from one thread so it's fine
unsafe impl Send for Audio {}

#[cfg(target_os = "android")]
impl Audio {
    fn new() -> Result<Audio> {
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
            .set_callback(SineWave::default())
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
    fn new() -> Result<Audio> {
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
    use crate::{java_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::jobject;
    use jni::JNIEnv;
    use paste::paste;

    fn get_audio<'a>(env: &'a JNIEnv, this: jobject) -> jni_helpers::JavaGetResult<'a, Audio> {
        jni_helpers::java_get(env, this)
    }

    java_func!(Audio_nativeConstructor, constructor, jobject);
    fn constructor(env: &JNIEnv, this: jobject, _emulator: jobject) -> Result<()> {
        let audio = Audio::new()?;
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
