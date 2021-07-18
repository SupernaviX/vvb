use anyhow::Result;
use oboe::{AudioStreamBuilder, AudioOutputCallback, IsFrameType, Output, DataCallbackResult, AudioStreamAsync, AudioOutputStreamSafe, Error, Status, IsFormat, IsChannelCount};
use std::sync::{Arc, Mutex};

pub trait AudioOutputStreamFactory
where
    Self: Send,
    (Self::Format, Self::ChannelCount): IsFrameType
{
    type Format: IsFormat;
    type ChannelCount: IsChannelCount;
    fn build_stream(&self) -> AudioStreamBuilder<Output, Self::ChannelCount, Self::Format>;
    #[allow(unused_variables)]
    fn on_error_before_close(&mut self, audio_stream: &mut dyn AudioOutputStreamSafe, error: Error) {}
    fn on_audio_ready(&mut self, audio_stream: &mut dyn AudioOutputStreamSafe, data: &mut [<(Self::Format, Self::ChannelCount) as IsFrameType>::Type]) -> DataCallbackResult;
}

type ManagedAudioStreamAsync<Config> = AudioStreamAsync<Output, AudioStreamReconnectCallback<Config>>;
pub struct AudioStreamReconnectCallback<Config> {
    config: Option<Config>,
    current: Arc<Mutex<Option<ManagedAudioStreamAsync<Config>>>>,
}

impl <Config: AudioOutputStreamFactory> AudioOutputCallback for AudioStreamReconnectCallback<Config>
where (Config::Format, Config::ChannelCount): IsFrameType{
    type FrameType = (Config::Format, Config::ChannelCount);

    fn on_error_before_close(&mut self, audio_stream: &mut dyn AudioOutputStreamSafe, error: Error) {
        match self.config.as_mut() {
            Some(config) => config.on_error_before_close(audio_stream, error),
            None => ()
        }
    }

    fn on_error_after_close(&mut self, _audio_stream: &mut dyn AudioOutputStreamSafe, error: Error) {
        if error != Error::Disconnected {
            return
        }
        let config = match self.config.take() {
            Some(config) => config,
            None => return
        };
        let builder = config.build_stream();
        let callback = AudioStreamReconnectCallback {
            config: Some(config),
            current: Arc::clone(&self.current)
        };
        let stream = builder
            .set_callback(callback)
            .open_stream();

        let stream = match stream {
            Ok(stream) => Some(stream),
            Err(error) => {
                log::error!("Could not open new stream: {}", error);
                None
            }
        };
        *self.current.lock().unwrap() = stream;
    }

    fn on_audio_ready(&mut self, audio_stream: &mut dyn AudioOutputStreamSafe, audio_data: &mut [<Self::FrameType as IsFrameType>::Type]) -> DataCallbackResult {
        match self.config.as_mut() {
            Some(config) => config.on_audio_ready(audio_stream, audio_data),
            None => DataCallbackResult::Stop
        }
    }
}

unsafe impl<Config: AudioOutputStreamFactory> Send for AudioStreamReconnectCallback<Config>
where
    (Config::Format, Config::ChannelCount): IsFrameType {}

pub struct AudioStreamManager<Config> {
    current: Arc<Mutex<Option<ManagedAudioStreamAsync<Config>>>>
}

impl <Config: AudioOutputStreamFactory> AudioStreamManager<Config>
where
    (Config::Format, Config::ChannelCount): IsFrameType {
    pub fn new(config: Config) -> Result<Self> {
        let current = Arc::new(Mutex::new(None));
        let builder = config.build_stream();
        let callback = AudioStreamReconnectCallback {
            config: Some(config),
            current: Arc::clone(&current)
        };
        let stream = builder.set_callback(callback).open_stream()?;
        *current.lock().unwrap() = Some(stream);
        Ok(Self { current })
    }

    pub fn with_stream_do<F: FnOnce(&mut AudioStreamAsync<Output, AudioStreamReconnectCallback<Config>>) -> Status>(&mut self, action: F) -> Result<()> {
        let mut current = self.current.lock().unwrap();
        match current.as_mut() {
            Some(stream) => action(stream)?,
            None => ()
        };
        Ok(())
    }
}