use crate::emulator::video::Eye;
use crate::video::cardboard::QrCode;

use super::common::RenderLogic;
use super::stereo::{Settings, StereoRenderLogic};

mod distortion_wrapper;
use distortion_wrapper::DistortionWrapper;

use anyhow::Result;
use log::debug;

pub struct CardboardRenderLogic {
    screen_size: (i32, i32),
    base: StereoRenderLogic,
    distortion: Option<DistortionWrapper>,
    cardboard_stale: bool,
}
impl CardboardRenderLogic {
    pub fn new(settings: &Settings) -> Self {
        Self {
            screen_size: (0, 0),
            base: StereoRenderLogic::new(&settings),
            distortion: None,
            cardboard_stale: true,
        }
    }

    pub fn ensure_device_params(&mut self) {
        self.cardboard_stale = true;
        if QrCode::get_saved_device_params().is_none() {
            QrCode::scan_qr_code_and_save_device_params();
        }
    }

    fn has_device_params(&self) -> bool {
        !self.cardboard_stale && self.distortion.is_some()
    }

    fn update_device_params(&mut self) -> Result<()> {
        if !self.cardboard_stale {
            return Ok(());
        }
        match DistortionWrapper::new(self.screen_size) {
            Ok(Some(distortion)) => {
                self.distortion = Some(distortion);
                self.cardboard_stale = false;
                Ok(())
            }
            Ok(None) => {
                self.distortion = None;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

impl RenderLogic for CardboardRenderLogic {
    fn init(&mut self) -> Result<()> {
        // If cardboard is already initialized, drop it.
        // This method is called when the GLSurfaceView is first initialized,
        // so if it already has values then those values reference already-freed resources,
        // and freeing it AFTER creating a new one will drop resources the new one is using.
        self.distortion.take();

        self.base.init()?;
        self.cardboard_stale = true;

        let device_params = QrCode::get_saved_device_params();
        debug!("Device params: {:?}", device_params);

        Ok(())
    }

    fn resize(&mut self, screen_size: (i32, i32)) -> Result<()> {
        self.screen_size = screen_size;
        self.base.resize(screen_size)?;
        self.cardboard_stale = true;
        Ok(())
    }

    fn update(&mut self, eye: Eye, buffer: &[u8]) -> Result<()> {
        self.base.update(eye, &buffer)?;
        self.update_device_params()
    }

    fn draw(&self) -> Result<()> {
        if !self.has_device_params() {
            return Ok(());
        }
        self.distortion
            .as_ref()
            .unwrap()
            .draw(|| self.base.draw())?;
        Ok(())
    }
}

#[rustfmt::skip::macros(jni_func)]
pub mod jni {
    use super::{CardboardRenderLogic, Settings};
    use crate::emulator::Emulator;
    use crate::jni_helpers::EnvExtensions;
    use crate::video::renderers::common::Renderer;
    use crate::{jni_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;

    type CardboardRenderer = Renderer<CardboardRenderLogic>;

    pub fn get_settings(env: &JNIEnv, this: jobject) -> Result<Settings> {
        let screen_zoom = env.get_percent(this, "screenZoom")?;
        let vertical_offset = env.get_percent(this, "verticalOffset")?;
        let color = env.get_color(this, "color")?;

        Ok(Settings {
            screen_zoom,
            vertical_offset,
            color,
        })
    }

    fn get_renderer<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, CardboardRenderer> {
        jni_helpers::java_get(env, this)
    }

    jni_func!(CardboardRenderer_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(&env, emulator)?;
        let settings = get_settings(&env, settings)?;
        let renderer = Renderer::new(
            emulator.get_frame_channel(),
            CardboardRenderLogic::new(&settings),
        );
        jni_helpers::java_init(env, this, renderer)
    }

    jni_func!(CardboardRenderer_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<CardboardRenderer>(env, this)
    }

    jni_func!(CardboardRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    jni_func!(CardboardRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(env: &JNIEnv, this: jobject, width: jint, height: jint) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    jni_func!(CardboardRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }

    jni_func!(CardboardRenderer_nativeEnsureDeviceParams, ensure_device_params);
    fn ensure_device_params(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.logic.ensure_device_params();
        Ok(())
    }
}
