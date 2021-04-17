use crate::emulator::video::FrameChannel;

mod cardboard;
pub use cardboard::api::Cardboard;
pub use cardboard::api::QrCode;
use cardboard::CardboardRenderer;

mod gl;

mod screen;
use screen::VBScreenRenderer;

use anyhow::Result;
use log::debug;
use std::sync::mpsc::TryRecvError;

pub struct Renderer {
    screen_size: (i32, i32),
    vb_screen: Option<VBScreenRenderer>,
    cardboard: Option<CardboardRenderer>,
    cardboard_stale: bool,
    frame_channel: FrameChannel,
    settings: Settings,
}
impl Renderer {
    pub fn new(frame_channel: FrameChannel, settings: Settings) -> Renderer {
        Renderer {
            screen_size: (0, 0),
            vb_screen: None,
            cardboard: None,
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
        self.cardboard.take();

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
        self.cardboard
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
        match CardboardRenderer::new(self.screen_size) {
            Ok(Some(cardboard)) => {
                self.cardboard = Some(cardboard);
                self.cardboard_stale = false;
                Ok(true)
            }
            Ok(None) => {
                self.cardboard = None;
                Ok(false)
            }
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug)]
pub struct Settings {
    screen_zoom: f32,
    vertical_offset: f32,
    color: (u8, u8, u8),
}

#[rustfmt::skip::macros(emulator_func)]
pub mod jni {
    use super::{Renderer, Settings};
    use crate::emulator::Emulator;
    use crate::{emulator_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;

    fn get_renderer<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, Renderer> {
        jni_helpers::java_get(env, this)
    }

    fn get_settings(env: &JNIEnv, this: jobject) -> Result<Settings> {
        let screen_zoom_percent = env.get_field(this, "_screenZoom", "I")?.i()?;
        let vertical_offset = env.get_field(this, "_verticalOffset", "I")?.i()?;
        let color = env.get_field(this, "_color", "I")?.i()?;

        // android passes color as ARGB
        let color = ((color >> 16) as u8, (color >> 8) as u8, color as u8);

        Ok(Settings {
            screen_zoom: (screen_zoom_percent as f32) / 100.0,
            vertical_offset: (vertical_offset as f32) / 100.0,
            color,
        })
    }

    emulator_func!(Renderer_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(&env, emulator)?;
        let settings = get_settings(&env, settings)?;
        let renderer = Renderer::new(emulator.get_frame_channel(), settings);
        jni_helpers::java_init(env, this, renderer)
    }

    emulator_func!(Renderer_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<Renderer>(env, this)
    }

    emulator_func!(Renderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    emulator_func!(Renderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(env: &JNIEnv, this: jobject, width: jint, height: jint) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height);
        Ok(())
    }

    emulator_func!(Renderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }

    emulator_func!(Renderer_nativeEnsureDeviceParams, ensure_device_params);
    fn ensure_device_params(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.ensure_device_params();
        Ok(())
    }

    emulator_func!(Renderer_nativeChangeDeviceParams, change_device_params);
    fn change_device_params(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.change_device_params();
        Ok(())
    }
}
