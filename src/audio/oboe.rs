mod manager;
use manager::{AudioOutputStreamFactory, AudioStreamManager};

use crate::emulator::audio::AudioPlayer;
use anyhow::Result;
use oboe::{AudioStream, AudioStreamBuilder, DataCallbackResult, Output, PerformanceMode, SampleRateConversionQuality, SharingMode, Stereo, AudioOutputStreamSafe, Error};

pub fn init(sample_rate: i32, frames_per_burst: i32) {
    oboe::DefaultStreamValues::set_sample_rate(sample_rate);
    oboe::DefaultStreamValues::set_frames_per_burst(frames_per_burst);
}

struct OboeStreamConfiguration {
    player: AudioPlayer,
}
impl AudioOutputStreamFactory for OboeStreamConfiguration {
    type Format = f32;
    type ChannelCount = Stereo;
    fn build_stream(&self) -> AudioStreamBuilder<Output, Stereo, f32> {
        AudioStreamBuilder::default()
            .set_format::<f32>()
            .set_channel_count::<Stereo>()
            .set_performance_mode(PerformanceMode::LowLatency)
            .set_sharing_mode(SharingMode::Shared)
            // virtual boy sample rate is mercifully low
            .set_sample_rate(41667)
            .set_sample_rate_conversion_quality(SampleRateConversionQuality::Fastest)
    }

    fn on_error_before_close(&mut self, _audio_stream: &mut dyn AudioOutputStreamSafe, error: Error) {
        log::warn!("Audio stream error: {}", error);
    }

    fn on_audio_ready(&mut self, _audio_stream: &mut dyn AudioOutputStreamSafe, data: &mut [(f32, f32)]) -> DataCallbackResult {
        self.player.play(data);
        DataCallbackResult::Continue
    }
}

pub struct OboeAudio {
    manager: AudioStreamManager<OboeStreamConfiguration>,
}
unsafe impl Send for OboeAudio {}
impl OboeAudio {
    pub fn new(player: AudioPlayer) -> Result<Self> {
        let config = OboeStreamConfiguration { player };
        Ok(Self {
            manager: AudioStreamManager::new(config)?,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        log::info!("audio start");
        self.manager.with_stream_do(|stream| stream.request_start())?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        log::info!("audio stop");
        self.manager.with_stream_do(|stream| stream.request_stop())?;
        Ok(())
    }
}
