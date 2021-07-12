mod stream_manager;
use stream_manager::{StreamConfiguration, StreamManager};

use crate::emulator::audio::AudioPlayer;
use anyhow::Result;
use oboe::{
    AudioStreamBuilder, DataCallbackResult, Output, PerformanceMode, SampleRateConversionQuality,
    SharingMode, Stereo, Unspecified,
};

pub fn init(sample_rate: i32, frames_per_burst: i32) {
    oboe::DefaultStreamValues::set_sample_rate(sample_rate);
    oboe::DefaultStreamValues::set_frames_per_burst(frames_per_burst);
}

struct OboeStreamConfiguration {
    player: AudioPlayer,
}
impl StreamConfiguration for OboeStreamConfiguration {
    type FrameType = (f32, Stereo);

    fn configure_stream(
        &self,
        builder: AudioStreamBuilder<Output, Unspecified, Unspecified>,
    ) -> AudioStreamBuilder<Output, Stereo, f32> {
        builder
            .set_performance_mode(PerformanceMode::LowLatency)
            .set_sharing_mode(SharingMode::Shared)
            .set_format::<f32>()
            .set_channel_count::<Stereo>()
            // virtual boy sample rate is mercifully low
            .set_sample_rate(41667)
            .set_sample_rate_conversion_quality(SampleRateConversionQuality::Fastest)
    }

    fn on_audio_ready(&mut self, data: &mut [(f32, f32)]) -> DataCallbackResult {
        self.player.play(data);
        DataCallbackResult::Continue
    }
}

pub struct OboeAudio {
    manager: StreamManager<OboeStreamConfiguration>,
}
impl OboeAudio {
    pub fn new(player: AudioPlayer) -> Result<Self> {
        let config = OboeStreamConfiguration { player };
        Ok(Self {
            manager: StreamManager::new(config)?,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        log::info!("audio start");
        self.manager.start()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        log::info!("audio stop");
        self.manager.stop()?;
        Ok(())
    }
}
