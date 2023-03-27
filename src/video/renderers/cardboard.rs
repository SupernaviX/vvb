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
            base: StereoRenderLogic::new(settings),
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
        self.base.update(eye, buffer)?;
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
    use crate::emulator::jni::get_emulator;
    use crate::jni_helpers::{JavaBinding, JavaGetResult};
    use crate::video::renderers::common::Renderer;
    use crate::{jni_func, EnvExtensions};
    use anyhow::Result;
    use jni::objects::JObject;
    use jni::sys::jint;
    use jni::JNIEnv;
    use std::convert::TryInto;

    type CardboardRenderer = Renderer<CardboardRenderLogic>;

    static CARDBOARD_BINDING: JavaBinding<CardboardRenderer> = JavaBinding::new();

    pub fn get_settings<'a>(env: &mut JNIEnv<'a>, this: JObject<'a>) -> Result<Settings> {
        let screen_zoom = env.get_percent(&this, "screenZoom")?;
        let aspect_ratio = env.get_int(&this, "aspectRatio")?.try_into()?;
        let vertical_offset = env.get_percent(&this, "verticalOffset")?;
        let color = env.get_color(&this, "color")?;

        Ok(Settings {
            screen_zoom,
            aspect_ratio,
            vertical_offset,
            color,
        })
    }

    fn get_renderer<'a>(
        env: &'a mut JNIEnv,
        this: JObject<'a>,
    ) -> JavaGetResult<'a, CardboardRenderer> {
        CARDBOARD_BINDING.get_value(env, this)
    }

    jni_func!(CardboardRenderer_nativeConstructor, constructor, JObject<'a>, JObject<'a>);
    fn constructor<'a>(
        env: &mut JNIEnv<'a>,
        this: JObject<'a>,
        emulator: JObject<'a>,
        settings: JObject<'a>,
    ) -> Result<()> {
        let settings = get_settings(env, settings)?;
        let renderer = {
            let mut emulator = get_emulator(env, emulator)?;
            Renderer::new(
                emulator.claim_frame_buffer_consumers(),
                CardboardRenderLogic::new(&settings),
            )
        };
        CARDBOARD_BINDING.init_value(env, this, renderer)
    }

    jni_func!(CardboardRenderer_nativeDestructor, destructor);
    fn destructor(env: &mut JNIEnv, this: JObject) -> Result<()> {
        CARDBOARD_BINDING.drop_value(env, this)
    }

    jni_func!(CardboardRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    jni_func!(CardboardRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(
        env: &mut JNIEnv,
        this: JObject,
        width: jint,
        height: jint,
    ) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    jni_func!(CardboardRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }

    jni_func!(CardboardRenderer_nativeEnsureDeviceParams, ensure_device_params);
    fn ensure_device_params(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.logic.ensure_device_params();
        Ok(())
    }
}
