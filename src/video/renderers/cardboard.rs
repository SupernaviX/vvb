use crate::emulator::video::FrameChannel;
use crate::video::cardboard::QrCode;

mod distortion_wrapper;
use distortion_wrapper::DistortionWrapper;

use super::common::{Settings, VBScreenRenderer};

use anyhow::Result;
use log::debug;
use std::sync::mpsc::TryRecvError;

pub struct CardboardRenderer {
    screen_size: (i32, i32),
    vb_screen: Option<VBScreenRenderer>,
    distortion: Option<DistortionWrapper>,
    cardboard_stale: bool,
    frame_channel: FrameChannel,
    settings: Settings,
}
impl CardboardRenderer {
    pub fn new(frame_channel: FrameChannel, settings: Settings) -> Self {
        Self {
            screen_size: (0, 0),
            vb_screen: None,
            distortion: None,
            cardboard_stale: true,
            frame_channel,
            settings,
        }
    }

    pub fn on_surface_created(&mut self) -> Result<()> {
        // If vb_screen or cardboard are already initialized, drop them.
        // This method is called when the GLSurfaceView is first initialized,
        // so if they already have values then those values reference already-freed resources,
        // and freeing them AFTER creating new ones will drop resources the new ones are using.
        self.vb_screen.take();
        self.distortion.take();

        self.vb_screen = Some(VBScreenRenderer::new(&self.settings)?);
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

    pub fn change_device_params(&mut self) {
        self.cardboard_stale = true;
        QrCode::scan_qr_code_and_save_device_params();
    }

    pub fn on_surface_changed(&mut self, screen_width: i32, screen_height: i32) {
        self.screen_size = (screen_width, screen_height);
        self.vb_screen
            .as_mut()
            .unwrap()
            .on_surface_changed(screen_width, screen_height);
        self.cardboard_stale = true;
    }

    pub fn on_draw_frame(&mut self) -> Result<()> {
        self.update_screen()?;
        if !self.update_device_params()? {
            return Ok(());
        }
        self.distortion
            .as_ref()
            .unwrap()
            .render(|| self.vb_screen.as_ref().unwrap().render())?;
        Ok(())
    }

    fn update_screen(&mut self) -> Result<()> {
        match self.frame_channel.try_recv() {
            Ok(frame) => self.vb_screen.as_mut().unwrap().update(frame),
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
        let renderer = CardboardRenderer::new(emulator.get_frame_channel(), settings);
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
        this.on_surface_changed(width, height);
        Ok(())
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

    emulator_func!(CardboardRenderer_nativeChangeDeviceParams, change_device_params);
    fn change_device_params(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.change_device_params();
        Ok(())
    }
}
