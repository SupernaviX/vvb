use self::video::{Eye, EyeBuffer, Frame, FrameChannel, FRAME_SIZE};
use anyhow::Result;
use std::sync::{mpsc, Arc, Mutex};

pub mod video {
    use std::sync::{Arc, Mutex};

    pub const VB_WIDTH: usize = 384;
    pub const VB_HEIGHT: usize = 224;
    pub const FRAME_SIZE: usize = VB_WIDTH * VB_HEIGHT * 4;

    #[derive(Copy, Clone)]
    pub enum Eye {
        Left,
        Right,
    }
    pub type EyeBuffer = [u8; FRAME_SIZE];

    pub struct Frame {
        pub eye: Eye,
        pub buffer: Arc<Mutex<EyeBuffer>>,
    }

    pub type FrameChannel = std::sync::mpsc::Receiver<Frame>;
}

pub struct Emulator {
    frame_channel: Option<mpsc::Sender<Frame>>,
    buffers: [Arc<Mutex<EyeBuffer>>; 2],
}
impl Emulator {
    fn new() -> Emulator {
        Emulator {
            frame_channel: None,
            buffers: [
                Arc::new(Mutex::new([0; FRAME_SIZE])),
                Arc::new(Mutex::new([0; FRAME_SIZE])),
            ],
        }
    }

    pub fn get_frame_channel(&mut self) -> FrameChannel {
        let (tx, rx) = mpsc::channel();
        self.frame_channel = Some(tx);
        rx
    }

    pub fn load_image(&self, left_eye: &[u8], right_eye: &[u8]) -> Result<()> {
        self.load_frame(Eye::Left, left_eye);
        self.send_frame(Eye::Left)?;
        self.load_frame(Eye::Right, right_eye);
        self.send_frame(Eye::Right)?;
        Ok(())
    }

    fn load_frame(&self, eye: Eye, image: &[u8]) {
        let mut buffer = self.buffers[eye as usize]
            .lock()
            .expect("Buffer lock was poisoned!");
        for (place, data) in buffer.iter_mut().zip(image.iter()) {
            *place = *data;
        }
    }

    fn send_frame(&self, eye: Eye) -> Result<()> {
        if let Some(channel) = self.frame_channel.as_ref() {
            let buffer = &self.buffers[eye as usize];
            channel.send(Frame {
                eye,
                buffer: Arc::clone(buffer),
            })?;
        }
        Ok(())
    }
}

#[rustfmt::skip::macros(java_func)]
pub mod jni {
    use super::Emulator;
    use crate::{java_func, jni_helpers};
    use anyhow::Result;
    use jni::objects::JByteBuffer;
    use jni::sys::jobject;
    use jni::JNIEnv;
    use paste::paste;

    fn get_emulator<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, Emulator> {
        jni_helpers::java_get(env, this)
    }

    java_func!(Emulator_nativeConstructor, constructor);
    fn constructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_init(env, this, Emulator::new())
    }

    java_func!(Emulator_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<Emulator>(env, this)
    }

    java_func!(Emulator_nativeLoadImage, load_image, JByteBuffer, JByteBuffer);
    fn load_image(
        env: &JNIEnv,
        this: jobject,
        left_eye: JByteBuffer,
        right_eye: JByteBuffer,
    ) -> Result<()> {
        let left_eye = env.get_direct_buffer_address(left_eye)?;
        let right_eye = env.get_direct_buffer_address(right_eye)?;
        let this = get_emulator(env, this)?;
        this.load_image(left_eye, right_eye)
    }
}
