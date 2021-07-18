use oboe::{
    AudioOutputCallback, AudioOutputStreamSafe, AudioStreamAsync, AudioStreamBuilder,
    DataCallbackResult, Error, IsChannelCount, IsFormat, IsFrameType, Output, Status,
};
use std::sync::{Arc, Mutex};

/// A limited version of Oboe's AudioOutputCallback which also knows how to construct its preferred stream.
pub trait ManagedAudioOutputCallback
where
    Self: Send,
    (Self::Format, Self::ChannelCount): IsFrameType,
{
    type Format: IsFormat;
    type ChannelCount: IsChannelCount;
    fn build_stream(&self) -> AudioStreamBuilder<Output, Self::ChannelCount, Self::Format>;
    #[allow(unused_variables)]
    fn on_error_before_close(
        &mut self,
        audio_stream: &mut dyn AudioOutputStreamSafe,
        error: Error,
    ) {
    }
    fn on_audio_ready(
        &mut self,
        audio_stream: &mut dyn AudioOutputStreamSafe,
        data: &mut [<(Self::Format, Self::ChannelCount) as IsFrameType>::Type],
    ) -> DataCallbackResult;
}

type ManagedAudioStreamAsync<Callback> =
    AudioStreamAsync<Output, ReconnectingAudioOutputCallback<Callback>>;

/// A wrapper for ManagedAudioOutputCallback which handles stream reconnection as needed.
/// May or may not contain the passed-in callback at any given time.
pub struct ReconnectingAudioOutputCallback<Callback: ManagedAudioOutputCallback>
where
    (Callback::Format, Callback::ChannelCount): IsFrameType,
{
    inner: Option<Callback>,
    current: Arc<Mutex<Option<ManagedAudioStreamAsync<Callback>>>>,
}

impl<Callback: ManagedAudioOutputCallback> AudioOutputCallback
    for ReconnectingAudioOutputCallback<Callback>
where
    (Callback::Format, Callback::ChannelCount): IsFrameType,
{
    type FrameType = (Callback::Format, Callback::ChannelCount);

    fn on_error_before_close(
        &mut self,
        audio_stream: &mut dyn AudioOutputStreamSafe,
        error: Error,
    ) {
        match self.inner.as_mut() {
            Some(inner) => inner.on_error_before_close(audio_stream, error),
            None => (),
        }
    }

    fn on_error_after_close(
        &mut self,
        _audio_stream: &mut dyn AudioOutputStreamSafe,
        error: Error,
    ) {
        if error != Error::Disconnected {
            return;
        }
        let inner = match self.inner.take() {
            Some(inner) => inner,
            None => return,
        };
        let builder = inner.build_stream();
        let callback = ReconnectingAudioOutputCallback {
            inner: Some(inner),
            current: Arc::clone(&self.current),
        };
        let stream = builder.set_callback(callback).open_stream();

        let stream = match stream {
            Ok(stream) => Some(stream),
            Err(error) => {
                log::error!("Could not open new stream: {}", error);
                None
            }
        };
        *self.current.lock().unwrap() = stream;
    }

    fn on_audio_ready(
        &mut self,
        audio_stream: &mut dyn AudioOutputStreamSafe,
        audio_data: &mut [<Self::FrameType as IsFrameType>::Type],
    ) -> DataCallbackResult {
        match self.inner.as_mut() {
            Some(inner) => inner.on_audio_ready(audio_stream, audio_data),
            None => DataCallbackResult::Stop,
        }
    }
}

// safety: AudioOutputStream isn't Send because it uses pointers internally,
// but we're using it with mutexes so everything should turn out OK
unsafe impl<Callback: ManagedAudioOutputCallback> Send for ReconnectingAudioOutputCallback<Callback> where
    (Callback::Format, Callback::ChannelCount): IsFrameType
{
}

/// Holds a reference to a self-reconnecting Oboe audio output stream.
pub struct AudioStreamManager<Callback: ManagedAudioOutputCallback>
where
    (Callback::Format, Callback::ChannelCount): IsFrameType,
{
    current: Arc<Mutex<Option<ManagedAudioStreamAsync<Callback>>>>,
}

impl<Callback: ManagedAudioOutputCallback> AudioStreamManager<Callback>
where
    (Callback::Format, Callback::ChannelCount): IsFrameType,
{
    pub fn new(inner: Callback) -> Result<Self, Error> {
        let current = Arc::new(Mutex::new(None));
        let builder = inner.build_stream();
        let callback = ReconnectingAudioOutputCallback {
            inner: Some(inner),
            current: Arc::clone(&current),
        };
        let stream = builder.set_callback(callback).open_stream()?;
        *current.lock().unwrap() = Some(stream);
        Ok(Self { current })
    }

    pub fn with_stream_do<
        F: FnOnce(&mut AudioStreamAsync<Output, ReconnectingAudioOutputCallback<Callback>>) -> Status,
    >(
        &mut self,
        action: F,
    ) -> Status {
        let mut current = self.current.lock().unwrap();
        match current.as_mut() {
            Some(stream) => action(stream),
            None => Ok(()),
        }
    }
}
// safety: AudioOutputStream isn't Send because it uses pointers internally,
// but we're using it with mutexes so everything should turn out OK
unsafe impl<Callback: ManagedAudioOutputCallback> Send for AudioStreamManager<Callback> where
    (Callback::Format, Callback::ChannelCount): IsFrameType
{
}
