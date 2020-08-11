mod cardboard;
pub use cardboard::api::Cardboard;
use cardboard::api::QrCode;
use cardboard::CardboardRenderer;

mod gl;

mod screen;
use screen::VBScreenRenderer;

use log::debug;

pub struct Renderer {
    screen_size: (i32, i32),
    vb_screen: Option<VBScreenRenderer>,
    cardboard: Option<CardboardRenderer>,
    cardboard_stale: bool,
}
impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            screen_size: (0, 0),
            vb_screen: None,
            cardboard: None,
            cardboard_stale: true,
        }
    }
    pub fn on_surface_created(&mut self, title_screen: &[u8]) -> Result<(), String> {
        self.cardboard_stale = true;
        self.vb_screen = Some(VBScreenRenderer::new(title_screen)?);

        let device_params = QrCode::get_saved_device_params();
        debug!("Device params: {:?}", device_params);

        Ok(())
    }

    pub fn ensure_device_params(&mut self) {
        self.cardboard_stale = true;
        if let None = QrCode::get_saved_device_params() {
            QrCode::scan_qr_code_and_save_device_params();
        }
    }

    pub fn change_device_params(&mut self) {
        self.cardboard_stale = true;
        QrCode::scan_qr_code_and_save_device_params();
    }

    pub fn on_surface_changed(&mut self, screen_width: i32, screen_height: i32) {
        self.screen_size = (screen_width, screen_height);
        match self.vb_screen.as_mut() {
            Some(screen) => screen.on_surface_changed(screen_width, screen_height),
            None => {}
        }
        self.cardboard_stale = true;
    }

    pub fn on_draw_frame(&mut self) -> Result<(), String> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        if !self.update_device_params()? {
            return Ok(());
        }
        self.cardboard
            .as_ref()
            .unwrap()
            .render(|| self.vb_screen.as_ref().unwrap().render())?;
        Ok(())
    }

    fn update_device_params(&mut self) -> Result<bool, String> {
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

#[rustfmt::skip::macros(java_func)]
pub mod jni {
    use super::Renderer;
    use crate::{java_func, jni_helpers};
    use jni::objects::JByteBuffer;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;
    use paste::paste;

    fn get_renderer<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, Renderer> {
        jni_helpers::java_get(env, this)
    }

    java_func!(Renderer_nativeConstructor, constructor);
    fn constructor(env: &JNIEnv, this: jobject) -> Result<(), String> {
        jni_helpers::java_init(env, this, Renderer::new())
    }

    java_func!(Renderer_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<(), String> {
        jni_helpers::java_take::<Renderer>(env, this)
    }

    java_func!(Renderer_nativeOnSurfaceCreated, on_surface_created, JByteBuffer);
    fn on_surface_created(
        env: &JNIEnv,
        this: jobject,
        title_screen: JByteBuffer,
    ) -> Result<(), String> {
        let buf = env
            .get_direct_buffer_address(title_screen)
            .map_err(|err| err.to_string())?;
        let mut this = get_renderer(env, this)?;
        this.on_surface_created(buf)
    }

    java_func!(Renderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(
        env: &JNIEnv,
        this: jobject,
        width: jint,
        height: jint,
    ) -> Result<(), String> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height);
        Ok(())
    }

    java_func!(Renderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<(), String> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }

    java_func!(Renderer_nativeEnsureDeviceParams, ensure_device_params);
    fn ensure_device_params(env: &JNIEnv, this: jobject) -> Result<(), String> {
        let mut this = get_renderer(env, this)?;
        this.ensure_device_params();
        Ok(())
    }

    java_func!(Renderer_nativeChangeDeviceParams, change_device_params);
    fn change_device_params(env: &JNIEnv, this: jobject) -> Result<(), String> {
        let mut this = get_renderer(env, this)?;
        this.change_device_params();
        Ok(())
    }
}
