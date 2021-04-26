use crate::emulator::video::FrameChannel;
use crate::video::cardboard::QrCode;

mod distortion_wrapper;
use distortion_wrapper::DistortionWrapper;

use super::common::{Settings, StereoDisplay};

use anyhow::Result;
use log::debug;
use std::sync::mpsc::TryRecvError;

pub struct CardboardRenderer {
    screen_size: (i32, i32),
    display: StereoDisplay,
    distortion: Option<DistortionWrapper>,
    cardboard_stale: bool,
    frame_channel: FrameChannel,
}
impl CardboardRenderer {
    pub fn new(frame_channel: FrameChannel, settings: &Settings) -> Self {
        Self {
            screen_size: (0, 0),
            display: StereoDisplay::new(&settings),
            distortion: None,
            cardboard_stale: true,
            frame_channel,
        }
    }

    pub fn on_surface_created(&mut self) -> Result<()> {
        // If cardboard is already initialized, drop it.
        // This method is called when the GLSurfaceView is first initialized,
        // so if it already has values then those values reference already-freed resources,
        // and freeing it AFTER creating a new one will drop resources the new one is using.
        self.distortion.take();

        self.display.init()?;
        self.cardboard_stale = true;

        let device_params = QrCode::get_saved_device_params();
        debug!("Device params: {:?}", device_params);

        Ok(())
    }

    pub fn ensure_device_params(&mut self) {
        self.cardboard_stale = true;
        if QrCode::get_saved_device_params().is_none() {
            QrCode::scan_qr_code_and_save_device_params();
        }
    }

    pub fn on_surface_changed(&mut self, screen_width: i32, screen_height: i32) -> Result<()> {
        self.screen_size = (screen_width, screen_height);
        self.display.resize((screen_width, screen_height))?;
        self.cardboard_stale = true;
        Ok(())
    }

    pub fn on_draw_frame(&mut self) -> Result<()> {
        self.update_screen()?;
        if !self.update_device_params()? {
            return Ok(());
        }
        self.distortion
            .as_ref()
            .unwrap()
            .render(|| self.display.render())?;
        Ok(())
    }

    fn update_screen(&mut self) -> Result<()> {
        match self.frame_channel.try_recv() {
            Ok(frame) => self.display.update(frame),
            Err(TryRecvError::Empty) => Ok(()),
            Err(TryRecvError::Disconnected) => Err(anyhow::anyhow!("Emulator has shut down")),
        }
    }

    fn update_device_params(&mut self) -> Result<bool> {
        if !self.cardboard_stale {
            return Ok(true);
        }
        match DistortionWrapper::new(self.screen_size) {
            Ok(Some(distortion)) => {
                self.distortion = Some(distortion);
                self.cardboard_stale = false;
                Ok(true)
            }
            Ok(None) => {
                self.distortion = None;
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }
}

#[rustfmt::skip::macros(emulator_func)]
pub mod jni {
    use super::super::common::get_settings;
    use super::CardboardRenderer;
    use crate::emulator::Emulator;
    use crate::{emulator_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;

    fn get_renderer<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, CardboardRenderer> {
        jni_helpers::java_get(env, this)
    }

    emulator_func!(CardboardRenderer_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(&env, emulator)?;
        let settings = get_settings(&env, settings)?;
        let renderer = CardboardRenderer::new(emulator.get_frame_channel(), &settings);
        jni_helpers::java_init(env, this, renderer)
    }

    emulator_func!(CardboardRenderer_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<CardboardRenderer>(env, this)
    }

    emulator_func!(CardboardRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    emulator_func!(CardboardRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(env: &JNIEnv, this: jobject, width: jint, height: jint) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    emulator_func!(CardboardRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }

    emulator_func!(CardboardRenderer_nativeEnsureDeviceParams, ensure_device_params);
    fn ensure_device_params(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.ensure_device_params();
        Ok(())
    }
}
