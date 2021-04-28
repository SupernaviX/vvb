use crate::video::gl;
use crate::video::gl::types::{GLint, GLsizei, GLuint};
use crate::video::gl::utils::{check_error, AsVoidptr};
use anyhow::Result;
use log::error;

unsafe fn init_gl_texture(id: GLuint, size: (i32, i32)) -> Result<()> {
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    check_error("generate a texture")?;
    gl::BindTexture(gl::TEXTURE_2D, id);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::LUMINANCE as GLint,
        size.0 as GLsizei,
        size.1 as GLsizei,
        0,
        gl::LUMINANCE,
        gl::UNSIGNED_BYTE,
        std::ptr::null(),
    );
    check_error("load a texture")?;

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
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
    Ok(())
}

pub struct Textures {
    pub ids: Vec<GLuint>,
    size: (i32, i32),
}
impl Textures {
    pub fn new(count: usize, size: (i32, i32)) -> Self {
        Self {
            ids: vec![0; count],
            size,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        unsafe {
            gl::GenTextures(self.ids.len() as GLint, self.ids.as_mut_ptr());
            for id in &self.ids {
                init_gl_texture(*id, self.size)?;
            }
            Ok(())
        }
    }

    pub fn update(&mut self, index: usize, buffer: &[u8]) -> Result<()> {
        let id = self.ids[index];
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + index as GLuint);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.size.0,
                self.size.1,
                gl::LUMINANCE,
                gl::UNSIGNED_BYTE,
                buffer.as_voidptr(),
            );
            check_error("update a texture")
        }
    }

    fn cleanup(&mut self) -> Result<()> {
        let textures = self.ids.len() as GLint;
        unsafe {
            gl::DeleteTextures(textures, self.ids.as_ptr());
        }
        for id in self.ids.iter_mut() {
            *id = 0;
        }
        check_error("cleaning up textures")
    }
}
impl Drop for Textures {
    fn drop(&mut self) {
        // Can't return a result from a Drop,
        // so just log if anything goes awry
        if let Err(message) = self.cleanup() {
            error!("{}", message);
        }
    }
}
