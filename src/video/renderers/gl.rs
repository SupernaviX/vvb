use crate::video::gl;
pub use crate::video::gl::types::GLfloat;
use crate::video::gl::types::{GLboolean, GLchar, GLenum, GLint, GLshort, GLsizei, GLuint};
use crate::video::gl::utils::{check_error, temp_array, AsVoidptr};
use anyhow::Result;
use cgmath::{self, Matrix4, SquareMatrix};
use log::error;
use std::ffi::{CStr, CString};

const GL_TRUE: GLboolean = 1;
const GL_FALSE: GLboolean = 0;

#[rustfmt::skip]
const POS_VERTICES: [GLfloat; 8] = [
    -0.5, 0.5,
    -0.5, -0.5,
    0.5, -0.5,
    0.5, 0.5
];

#[rustfmt::skip]
const TEX_VERTICES: [GLfloat; 8] = [
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
        CString::new($string).map_err(|_| anyhow::anyhow!("Could not build c string!"))
    };
}

fn matrix_as_f4v<S: Clone>(mat: cgmath::Matrix4<S>) -> Vec<S> {
    let mut res = Vec::with_capacity(16);
    let rows: [[S; 4]; 4] = mat.into();
    for row in rows.iter() {
        res.extend_from_slice(row);
    }
    res
}
fn color_as_4fv(color: (u8, u8, u8)) -> [GLfloat; 4] {
    [
        color.0 as GLfloat / 255.0,
        color.1 as GLfloat / 255.0,
        color.2 as GLfloat / 255.0,
        1.0,
    ]
}

const VERTEX_SHADER: &str = "\
attribute vec4 a_Pos;
attribute vec2 a_TexCoord;
uniform mat4 u_MV;
varying vec2 v_TexCoord;
void main() {
    gl_Position = u_MV * a_Pos;
    v_TexCoord = a_TexCoord;
}
";

const FRAGMENT_SHADER: &str = "\
precision mediump float;
varying vec2 v_TexCoord;
uniform sampler2D u_Texture;
uniform vec4 u_Color;
void main() {
    gl_FragColor = u_Color * texture2D(u_Texture, v_TexCoord).r;
}
";

unsafe fn make_shader(type_: GLenum, source: &str) -> Result<GLuint> {
    let shader_id = gl::CreateShader(type_);
    let shader_str = c_string!(source)?;
    let shader_arr = [shader_str.as_ptr()];
    let shader_source = shader_arr.as_ptr();
    gl::ShaderSource(shader_id, 1, shader_source, std::ptr::null());
    check_error("load a shader's source")?;
    gl::CompileShader(shader_id);
    check_error("compile a shader")?;
    check_shader(type_, shader_id)?;
    Ok(shader_id)
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

unsafe fn init_gl_texture(texture_id: GLuint, texture_size: (i32, i32)) -> Result<()> {
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    check_error("generate a texture")?;
    gl::BindTexture(gl::TEXTURE_2D, texture_id);
    gl::TexImage2D(
        gl::TEXTURE_2D,
        0,
        gl::LUMINANCE as GLint,
        texture_size.0 as GLsizei,
        texture_size.1 as GLsizei,
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

#[derive(Debug)]
pub struct Program {
    program_id: GLuint,
    position_location: GLuint,
    tex_coord_location: GLuint,
    modelview_location: GLint,
    texture_location: GLint,
    color_location: GLint,
    texture_size: (i32, i32),

    texture_ids: [GLuint; 2],
    texture_color: [GLfloat; 4],
    model_views: [Vec<GLfloat>; 2],
}
impl Program {
    pub fn new(texture_size: (i32, i32), color: (u8, u8, u8)) -> Self {
        Self {
            // Initialize all the GL values to something invalid
            // All methods call gl::GetError pretty aggressively,
            // so forgetting to init should fail early and visibly
            program_id: 0,
            position_location: 0,
            tex_coord_location: 0,
            modelview_location: -1,
            texture_location: -1,
            color_location: -1,
            texture_size,

            texture_ids: [0; 2], // never valid textures
            texture_color: color_as_4fv(color),
            model_views: [
                matrix_as_f4v(Matrix4::identity()),
                matrix_as_f4v(Matrix4::identity()),
            ],
        }
    }

    pub fn init(&mut self) -> Result<()> {
        unsafe {
            let textures = self.texture_ids.len() as GLint;
            self.program_id = gl::CreateProgram();
            check_error("create a program")?;

            let vertex_shader = make_shader(gl::VERTEX_SHADER, VERTEX_SHADER)?;
            gl::AttachShader(self.program_id, vertex_shader);
            check_error("attach the vertex shader")?;

            let fragment_shader = make_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
            gl::AttachShader(self.program_id, fragment_shader);
            check_error("attach the fragment shader")?;

            gl::LinkProgram(self.program_id);
            check_error("link a program")?;
            gl::UseProgram(self.program_id);
            check_error("build a program")?;

            self.position_location =
                gl::GetAttribLocation(self.program_id, c_string!("a_Pos")?.as_ptr()) as GLuint;
            self.tex_coord_location =
                gl::GetAttribLocation(self.program_id, c_string!("a_TexCoord")?.as_ptr()) as GLuint;
            self.modelview_location =
                gl::GetUniformLocation(self.program_id, c_string!("u_MV")?.as_ptr());
            self.texture_location =
                gl::GetUniformLocation(self.program_id, c_string!("u_Texture")?.as_ptr());
            self.color_location =
                gl::GetUniformLocation(self.program_id, c_string!("u_Color")?.as_ptr());

            gl::GenTextures(textures, self.texture_ids.as_mut_ptr());
            for texture_id in &self.texture_ids {
                init_gl_texture(*texture_id, self.texture_size)?;
            }

            // Set color here, because it's the same for the entire life of the program
            gl::Uniform4fv(self.color_location, 1, self.texture_color.as_ptr());
        }
        Ok(())
    }

    pub fn resize(&mut self, size: (GLint, GLint)) -> Result<()> {
        unsafe {
            gl::Viewport(0, 0, size.0, size.1);
            check_error("setting the viewport")
        }
    }

    pub fn set_model_views(&mut self, model_views: [Matrix4<GLfloat>; 2]) {
        self.model_views = [matrix_as_f4v(model_views[0]), matrix_as_f4v(model_views[1])];
    }

    pub fn update(&mut self, texture: usize, buffer: &[u8]) -> Result<()> {
        let texture_id = self.texture_ids[texture];
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                self.texture_size.0,
                self.texture_size.1,
                gl::LUMINANCE,
                gl::UNSIGNED_BYTE,
                buffer.as_voidptr(),
            );
        }
        check_error("update a texture")?;
        Ok(())
    }

    pub fn render(&self) -> Result<()> {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            check_error("clear the screen")?;
            gl::UseProgram(self.program_id);
            check_error("use the program")?;

            for i in 0..self.texture_ids.len() {
                self.render_texture(i)?;
            }
            Ok(())
        }
    }

    unsafe fn render_texture(&self, index: usize) -> Result<()> {
        let texture_id = self.texture_ids[index];
        let model_view = &self.model_views[index];

        let pos_pointer = POS_VERTICES.as_voidptr();
        gl::VertexAttribPointer(
            self.position_location,
            VERTEX_SIZE,
            gl::FLOAT,
            gl::FALSE,
            VERTEX_STRIDE,
            pos_pointer,
        );
        check_error("pass position data to the shader")?;

        let tex_pointer = TEX_VERTICES.as_voidptr();
        gl::VertexAttribPointer(
            self.tex_coord_location,
            VERTEX_SIZE,
            gl::FLOAT,
            gl::FALSE,
            VERTEX_STRIDE,
            tex_pointer,
        );
        check_error("pass texture data to the shader")?;

        gl::EnableVertexAttribArray(self.position_location);
        gl::EnableVertexAttribArray(self.tex_coord_location);

        gl::UniformMatrix4fv(self.modelview_location, 1, GL_FALSE, model_view.as_ptr());
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::Uniform1i(self.texture_location, 0);

        gl::DrawElements(
            gl::TRIANGLES,
            SQUARE_INDICES.len() as i32,
            gl::UNSIGNED_SHORT,
            SQUARE_INDICES.as_voidptr(),
        );
        check_error("render a texture")?;
        Ok(())
    }

    // Idempotently cleans up resources used by this program
    fn cleanup(&mut self) -> Result<()> {
        unsafe {
            if gl::IsProgram(self.program_id) == GL_TRUE {
                gl::DeleteProgram(self.program_id);
            }
            let textures = self.texture_ids.len() as GLint;
            gl::DeleteTextures(textures, self.texture_ids.as_ptr());
            check_error("cleaning up a Program")
        }
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
