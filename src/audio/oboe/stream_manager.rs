use anyhow::Result;
use num_traits::FromPrimitive;
use oboe::{AudioStreamBuilder, DataCallbackResult, Error, IsFrameType, Output, Unspecified};
use oboe_sys as ffi;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::slice::from_raw_parts_mut;
use std::sync::{Arc, Mutex};

pub trait StreamConfiguration {
    type FrameType: IsFrameType;
    fn configure_stream(
        &self,
        builder: AudioStreamBuilder<Output, Unspecified, Unspecified>,
    ) -> AudioStreamBuilder<
        Output,
        <Self::FrameType as IsFrameType>::ChannelCount,
        <Self::FrameType as IsFrameType>::Format,
    >;
    fn on_audio_ready(
        &mut self,
        data: &mut [<Self::FrameType as IsFrameType>::Type],
    ) -> DataCallbackResult;
}

pub struct StreamManager<T: StreamConfiguration> {
    // hold onto the context here.
    // individual streams "own" it, but this lives longer than any stream
    // so we want to trigger cleanup when THIS is dropped
    #[allow(dead_code)]
    context: Box<StreamContext<T>>,
    stream: Arc<Mutex<*mut ffi::oboe_AudioStream>>,
}
// Mark StreamManager as Send because it is:
// 1. "stream" is an Arc<Mutex<...>> so it's thread-safe
// 2. after initialization, "context" is only ever accessed by the single active stream
unsafe impl<T: StreamConfiguration> Send for StreamManager<T> {}
impl<T: StreamConfiguration> StreamManager<T> {
    pub fn new(config: T) -> Result<Self> {
        let mut context = Box::new(StreamContext::new(config));
        context.init()?;
        let stream = context.get_stream();
        Ok(Self { context, stream })
    }
    pub fn start(&mut self) -> Result<()> {
        let stream = self.stream.lock().unwrap();
        wrap_status(unsafe { ffi::oboe_AudioStream_requestStart(*stream) })?;
        Ok(())
    }
    pub fn stop(&mut self) -> Result<()> {
        let stream = self.stream.lock().unwrap();
        wrap_status(unsafe { ffi::oboe_AudioStream_requestStop(*stream) })?;
        Ok(())
    }
}
impl<T: StreamConfiguration> Drop for StreamManager<T> {
    fn drop(&mut self) {
        // Close and delete the active stream
        let mut stream = self.stream.lock().unwrap();
        if let Err(code) =
            wrap_status(unsafe { ffi::oboe_AudioStream_close(*stream as *mut c_void) })
        {
            log::warn!("Error when closing stream: {}", code);
        }
        unsafe { ffi::oboe_AudioStream_delete(*stream) }
        *stream = std::ptr::null_mut();
    }
}

struct StreamContext<T: StreamConfiguration> {
    config: T,
    callback: Option<StreamManagerCallbackWrapperHandle>,
    stream: Arc<Mutex<*mut ffi::oboe_AudioStream>>,
}
impl<T: StreamConfiguration> StreamContext<T> {
    pub fn new(config: T) -> Self {
        Self {
            config,
            callback: None,
            stream: Arc::new(Mutex::new(std::ptr::null_mut())),
        }
    }
    pub fn init(&mut self) -> Result<()> {
        self.callback = Some(StreamManagerCallbackWrapperHandle::new(self));
        self.build_stream(false)?;
        Ok(())
    }
    pub fn get_stream(&self) -> Arc<Mutex<*mut ffi::oboe_AudioStream>> {
        Arc::clone(&self.stream)
    }
    pub fn on_error_before_close(&mut self, _stream: *mut ffi::oboe_AudioStream, error: Error) {
        log::warn!("Oboe error: {}", error);
    }
    pub fn on_error_after_close(&mut self, _stream: *mut ffi::oboe_AudioStream, error: Error) {
        if error == Error::Disconnected {
            self.build_stream(true).unwrap();
        }
    }
    pub fn on_audio_ready(
        &mut self,
        _stream: *mut ffi::oboe_AudioStream,
        data: &mut [<T::FrameType as IsFrameType>::Type],
    ) -> DataCallbackResult {
        self.config.on_audio_ready(data)
    }
    fn build_stream(&mut self, start: bool) -> Result<()> {
        let builder = self.config.configure_stream(AudioStreamBuilder::default());

        // Get the raw AudioStreamBuilder so that we can use our own callback in it
        // safety: oboe's AudioStreamBuilder is a repr(transparent) wrapper for this type
        let raw_builder: *mut ffi::oboe_AudioStreamBuilder =
            unsafe { std::mem::transmute(builder) };
        let callback = self.callback.as_ref().unwrap();
        unsafe {
            ffi::oboe_AudioStreamBuilder_setCallback(raw_builder, callback.raw());
        }

        // get that stream open
        let new_stream = unsafe {
            let mut ptr = MaybeUninit::uninit();
            wrap_status(ffi::oboe_AudioStreamBuilder_openStream(
                raw_builder,
                ptr.as_mut_ptr(),
            ))?;
            ptr.assume_init()
        };
        if start {
            // start er up
            wrap_status(unsafe { ffi::oboe_AudioStream_requestStart(new_stream) })?;
        }

        // save a ref to it so we can manipulate it later
        let mut stream = self.stream.lock().expect("mutex was poisoned!");
        *stream = new_stream;

        Ok(())
    }
}

#[repr(transparent)]
struct StreamManagerCallbackWrapperHandle(*mut ffi::oboe_AudioStreamCallbackWrapper);

impl StreamManagerCallbackWrapperHandle {
    pub fn new<T: StreamConfiguration>(context: &mut StreamContext<T>) -> Self {
        let handle = unsafe {
            ffi::oboe_AudioStreamCallbackWrapper_new(
                Some(on_audio_ready_output_wrapper::<T>),
                Some(on_error_before_close_output_wrapper::<T>),
                Some(on_error_after_close_output_wrapper::<T>),
            )
        };
        unsafe {
            ffi::oboe_AudioStreamCallbackWrapper_setContext(
                handle,
                context as *mut _ as *mut c_void,
            );
        }
        Self(handle)
    }

    pub fn raw(&self) -> *mut ffi::oboe_AudioStreamCallbackWrapper {
        self.0
    }
}
impl Drop for StreamManagerCallbackWrapperHandle {
    fn drop(&mut self) {
        unsafe { ffi::oboe_AudioStreamCallbackWrapper_delete(self.0) }
    }
}

unsafe extern "C" fn on_error_before_close_output_wrapper<T: StreamConfiguration>(
    context: *mut c_void,
    audio_stream: *mut ffi::oboe_AudioStream,
    error: ffi::oboe_Result,
) {
    let callback = &mut *(context as *mut StreamContext<T>);
    callback.on_error_before_close(audio_stream, FromPrimitive::from_i32(error).unwrap());
}

unsafe extern "C" fn on_error_after_close_output_wrapper<T: StreamConfiguration>(
    context: *mut c_void,
    audio_stream: *mut ffi::oboe_AudioStream,
    error: ffi::oboe_Result,
) {
    let callback = &mut *(context as *mut StreamContext<T>);
    callback.on_error_after_close(audio_stream, FromPrimitive::from_i32(error).unwrap());
}

unsafe extern "C" fn on_audio_ready_output_wrapper<T: StreamConfiguration>(
    context: *mut c_void,
    audio_stream: *mut ffi::oboe_AudioStream,
    audio_data: *mut c_void,
    num_frames: i32,
) -> ffi::oboe_DataCallbackResult {
    let audio_data = from_raw_parts_mut(
        audio_data as *mut <T::FrameType as IsFrameType>::Type,
        num_frames as usize,
    );

    let callback = &mut *(context as *mut StreamContext<T>);

    callback.on_audio_ready(audio_stream, audio_data) as i32
}

fn wrap_status(status: i32) -> Result<(), Error> {
    if status == ffi::oboe_Result_OK {
        Ok(())
    } else {
        Err(FromPrimitive::from_i32(status).unwrap())
    }
}
