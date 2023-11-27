use crate::video::gl;
use crate::video::gl::types::{
    GLboolean, GLchar, GLenum, GLfloat, GLint, GLshort, GLsizei, GLuint,
};
use crate::video::gl::utils::{check_error, temp_array, AsVoidptr};
use anyhow::Result;
use log::error;
use std::ffi::{CStr, CString};

const GL_TRUE: GLboolean = 1;
const GL_FALSE: GLboolean = 0;

#[rustfmt::skip]
const SQUARE_POS_VERTICES: [GLfloat; 8] = [
    -0.5, 0.5,
    -0.5, -0.5,
    0.5, -0.5,
    0.5, 0.5
];

#[rustfmt::skip]
const SQUARE_TEX_VERTICES: [GLfloat; 8] = [
    0.0, 0.0,
    0.0, 1.0,
    1.0, 1.0,
    1.0, 0.0
];

const SQUARE_INDICES: [GLshort; 6] = [0, 1, 2, 0, 2, 3];

const VERTEX_SIZE: GLsizei = 2;
const VERTEX_STRIDE: GLsizei = 0;

macro_rules! c_string {
    ($string:expr) => {
        CString::new($string).unwrap()
    };
}

pub fn make_shader(type_: GLenum, source: &str) -> Result<GLuint> {
    unsafe {
        let shader_id = gl::CreateShader(type_);
        let shader_str = c_string!(source);
        let shader_arr = [shader_str.as_ptr()];
        let shader_source = shader_arr.as_ptr();
        gl::ShaderSource(shader_id, 1, shader_source, std::ptr::null());
        check_error("load a shader's source")?;
        gl::CompileShader(shader_id);
        check_error("compile a shader")?;
        check_shader(type_, shader_id)?;
        Ok(shader_id)
    }
}

unsafe fn check_shader(type_: GLenum, shader_id: GLuint) -> Result<()> {
    let status = temp_array(|ptr| gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, ptr)) as GLboolean;
    check_error("checking compile status of a shader")?;
    if status == GL_TRUE {
        return Ok(());
    }

    let length = temp_array(|ptr| {
        gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, ptr);
    });
    check_error("finding info log length for a shader")?;
    if length < 0 {
        return Err(anyhow::anyhow!("Invalid shader info log length"));
    }
    let mut buf = vec![0; length as usize];
    let buf_ptr = buf.as_mut_ptr() as *mut GLchar;
    gl::GetShaderInfoLog(shader_id, length, std::ptr::null_mut(), buf_ptr);
    let cstr = CStr::from_bytes_with_nul(buf.as_slice())?;

    let log = cstr.to_str()?;
    Err(anyhow::anyhow!(
        "Error compiling shader type {:04X}! <{}>",
        type_,
        log.trim()
    ))
}

pub struct Program {
    id: GLuint,
    vertex_shader: &'static str,
    fragment_shader: &'static str,
}
#[allow(clippy::wrong_self_convention)]
impl Program {
    pub fn new(vertex_shader: &'static str, fragment_shader: &'static str) -> Self {
        Self {
            id: 0,
            vertex_shader,
            fragment_shader,
        }
    }
    pub fn init(&mut self) -> Result<()> {
        unsafe {
            self.id = gl::CreateProgram();
            if self.id == 0 {
                let error = gl::GetError();
                return Err(anyhow::anyhow!(
                    "Could not create OpenGL program (error code was 0x{:04X})",
                    error
                ));
            }

            let vertex_shader = make_shader(gl::VERTEX_SHADER, self.vertex_shader)?;
            gl::AttachShader(self.id, vertex_shader);
            check_error("attach the vertex shader")?;

            let fragment_shader = make_shader(gl::FRAGMENT_SHADER, self.fragment_shader)?;
            gl::AttachShader(self.id, fragment_shader);
            check_error("attach the fragment shader")?;

            gl::LinkProgram(self.id);
            check_error("link a program")?;
            gl::UseProgram(self.id);
            check_error("use a program")?;

            Ok(())
        }
    }

    pub fn set_viewport(&self, size: (GLint, GLint)) -> Result<()> {
        unsafe {
            gl::Viewport(0, 0, size.0, size.1);
            check_error("set the viewport")
        }
    }

    pub fn get_attribute_location(&self, name: &str) -> GLuint {
        let name = c_string!(name);
        unsafe { gl::GetAttribLocation(self.id, name.as_ptr()) as GLuint }
    }

    pub fn get_uniform_location(&self, name: &str) -> GLint {
        let name = c_string!(name);
        unsafe { gl::GetUniformLocation(self.id, name.as_ptr()) }
    }

    pub fn set_uniform_vector(&self, location: GLint, value: &[GLfloat; 4]) {
        unsafe {
            gl::Uniform4fv(location, 1, value.as_ptr());
        }
    }

    pub fn set_uniform_vector_array(&self, location: GLint, values: &[[GLfloat; 4]]) {
        unsafe {
            gl::Uniform4fv(
                location,
                values.len() as GLint,
                values.as_ptr() as *const GLfloat,
            );
        }
    }
    pub fn set_uniform_matrix(&self, location: GLint, value: &[GLfloat; 16]) {
        unsafe {
            gl::UniformMatrix4fv(location, 1, GL_FALSE, value.as_ptr());
        }
    }

    pub fn set_uniform_texture(&self, location: GLint, id: GLuint) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::Uniform1i(location, 0);
        }
    }

    pub fn set_uniform_texture_array(&self, location: GLint, ids: &[GLuint]) {
        unsafe {
            for (i, id) in ids.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + i as GLuint);
                gl::BindTexture(gl::TEXTURE_2D, *id);
            }
            let slots: Vec<GLint> = (0..ids.len() as GLint).collect();
            gl::Uniform1iv(location, slots.len() as GLsizei, slots.as_ptr());
        }
    }

    pub fn start_render(&self) -> Result<()> {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            check_error("clear the screen")?;
            gl::UseProgram(self.id);
            check_error("use a program")
        }
    }

    pub fn draw_square(&self, position_location: GLuint, tex_coord_location: GLuint) -> Result<()> {
        unsafe {
            let pos_pointer = SQUARE_POS_VERTICES.as_voidptr();
            gl::VertexAttribPointer(
                position_location,
                VERTEX_SIZE,
                gl::FLOAT,
                gl::FALSE,
                VERTEX_STRIDE,
                pos_pointer,
            );
            check_error("pass position data to the shader")?;

            let tex_pointer = SQUARE_TEX_VERTICES.as_voidptr();
            gl::VertexAttribPointer(
                tex_coord_location,
                VERTEX_SIZE,
                gl::FLOAT,
                gl::FALSE,
                VERTEX_STRIDE,
                tex_pointer,
            );
            check_error("pass texture data to the shader")?;

            gl::EnableVertexAttribArray(position_location);
            gl::EnableVertexAttribArray(tex_coord_location);

            gl::DrawElements(
                gl::TRIANGLES,
                SQUARE_INDICES.len() as i32,
                gl::UNSIGNED_SHORT,
                SQUARE_INDICES.as_voidptr(),
            );
            check_error("render a texture")
        }
    }
    fn cleanup(&mut self) -> Result<()> {
        unsafe {
            if gl::IsProgram(self.id) == GL_TRUE {
                gl::DeleteProgram(self.id)
            }
        }
        self.id = 0;
        check_error("clean up a Program")
    }
}
impl Drop for Program {
    fn drop(&mut self) {
        // Can't return a result from a Drop,
        // so just log if anything goes awry
        if let Err(message) = self.cleanup() {
            error!("{}", message);
        }
    }
}
