use crate::video::cardboard::{
    CardboardEye, DistortionRenderer, LensDistortion, QrCode, TextureDescription,
};

use crate::video::gl;
use crate::video::gl::types::{GLint, GLuint};
use crate::video::gl::utils::{check_error, temp_array};

use anyhow::Result;
use log::error;

pub struct DistortionWrapper {
    #[allow(dead_code)]
    lens_distortion: LensDistortion,
    distortion_renderer: DistortionRenderer,
    texture: GLuint,
    framebuffer: GLuint,
    screen_size: (i32, i32),
    left_eye: TextureDescription,
    right_eye: TextureDescription,
}
impl DistortionWrapper {
    pub fn new(screen_size: (i32, i32)) -> Result<Option<Self>> {
        let params = QrCode::get_saved_device_params();
        if params.is_none() {
            return Ok(None);
        }
        let params = params.unwrap();

        let texture = unsafe { temp_array(|ptr| gl::GenTextures(1, ptr)) };
        check_error("create a texture for cardboard")?;
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as GLint,
            );
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as GLint,
                screen_size.0,
                screen_size.1,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
        }
        check_error("prepare a texture for cardboard")?;

        let framebuffer = unsafe { temp_array(|ptr| gl::GenTextures(1, ptr)) };
        check_error("create a framebuffer for cardboard")?;
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture,
                0,
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        check_error("prepare a renderbuffer for cardboard")?;

        let lens_distortion = LensDistortion::create(&params, screen_size.0, screen_size.1);
        let left_mesh = lens_distortion.get_distortion_mesh(CardboardEye::kLeft);
        let right_mesh = lens_distortion.get_distortion_mesh(CardboardEye::kRight);

        let distortion_renderer = DistortionRenderer::create();
        distortion_renderer.set_mesh(&left_mesh, CardboardEye::kLeft);
        distortion_renderer.set_mesh(&right_mesh, CardboardEye::kRight);

        Ok(Some(Self {
            lens_distortion,
            distortion_renderer,
            texture,
            framebuffer,
            screen_size,
            left_eye: TextureDescription {
                texture,
                left_u: 0.0,
                right_u: 0.5,
                top_v: 1.0,
                bottom_v: 0.0,
            },
            right_eye: TextureDescription {
                texture,
                left_u: 0.5,
                right_u: 1.0,
                top_v: 1.0,
                bottom_v: 0.0,
            },
        }))
    }

    pub fn render<F: FnOnce() -> Result<()>>(&self, render_contents: F) -> Result<()> {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        }
        render_contents()?;
        self.distortion_renderer.render_eye_to_display(
            0,
            (0, 0),
            self.screen_size,
            &self.left_eye,
            &self.right_eye,
        );
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        check_error("render to cardboard")
    }
}
impl Drop for DistortionWrapper {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.framebuffer);
            gl::DeleteTextures(1, &self.texture);

            // Can't return a result from a Drop,
            // so just log if anything goes awry
            if let Err(message) = check_error("cleaning up a CardboardRenderer") {
                error!("{}", message);
            }
        }
    }
}
