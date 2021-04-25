use super::common::{Settings, StereoDisplay};
use crate::emulator::video::FrameChannel;

use anyhow::Result;
use std::sync::mpsc::TryRecvError;

pub struct AnaglyphRenderer {
    display: StereoDisplay,
    frame_channel: FrameChannel,
}
impl AnaglyphRenderer {
    pub fn new(frame_channel: FrameChannel, settings: Settings) -> Self {
        AnaglyphRenderer {
            display: StereoDisplay::new(settings),
            frame_channel,
        }
    }

    pub fn on_surface_created(&mut self) -> Result<()> {
        self.display.init()
    }

    pub fn on_surface_changed(&mut self, width: i32, height: i32) -> Result<()> {
        self.display.resize((width, height))
    }

    pub fn on_draw_frame(&mut self) -> Result<()> {
        self.update_screen()?;
        self.display.render()
    }

    fn update_screen(&mut self) -> Result<()> {
        match self.frame_channel.try_recv() {
            Ok(frame) => self.display.update(frame),
            Err(TryRecvError::Empty) => Ok(()),
            Err(TryRecvError::Disconnected) => Err(anyhow::anyhow!("Emulator has shut down")),
        }
    }
}

#[rustfmt::skip::macros(emulator_func)]
pub mod jni {
    use super::super::common::get_settings;
    use super::AnaglyphRenderer;
    use crate::emulator::Emulator;
    use crate::{emulator_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;

    fn get_renderer<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, AnaglyphRenderer> {
        jni_helpers::java_get(env, this)
    }

    emulator_func!(AnaglyphRenderer_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(&env, emulator)?;
        let settings = get_settings(&env, settings)?;
        let renderer = AnaglyphRenderer::new(emulator.get_frame_channel(), settings);
        jni_helpers::java_init(env, this, renderer)
    }

    emulator_func!(AnaglyphRenderer_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<AnaglyphRenderer>(env, this)
    }

    emulator_func!(AnaglyphRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    emulator_func!(AnaglyphRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(env: &JNIEnv, this: jobject, width: jint, height: jint) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    emulator_func!(AnaglyphRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }
}
