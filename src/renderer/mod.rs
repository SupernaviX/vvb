#[cfg(target_os = "android")]
mod gl {
    #[link(name="GLESv2")]
    extern {}
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}
mod cardboard_api;
pub use cardboard_api::initialize;

#[cfg(target_os = "windows")]
mod gl_bindings;
#[cfg(target_os = "windows")]
mod gl {
    pub use super::gl_bindings::*;
}

use gl::types::{GLboolean,GLshort,GLuint,GLint,GLsizei,GLchar,GLvoid,GLfloat,GLenum};
use std::ffi::{CStr,CString};
use cgmath::{self,Matrix4,SquareMatrix,vec4};
use log::{debug};

const GL_TRUE: GLboolean = 1;
const GL_FALSE: GLboolean = 0;

macro_rules! c_string {
    ($string:expr) => {
        CString::new($string).map_err(|_| { "Could not build c string!" })
    };
}

fn gl_temp_array<T: Copy + Default, F: FnOnce(*mut T) -> ()>(cb: F) -> T {
    let mut tmp_array: [T;1] = Default::default();
    cb(tmp_array.as_mut_ptr());
    tmp_array[0]
}

trait AsVoidptr {
    fn as_voidptr(&self) -> *const GLvoid;
    fn as_mut_voidptr(&mut self) -> *mut GLvoid;
}

impl<T> AsVoidptr for [T] {
    fn as_voidptr(&self) -> *const GLvoid {
        self.as_ptr() as *const GLvoid
    }

    fn as_mut_voidptr(&mut self) -> *mut GLvoid {
        self.as_mut_ptr() as *mut GLvoid
    }
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
void main() {
    gl_FragColor = texture2D(u_Texture, v_TexCoord);
}
";

const VB_WIDTH: i32 = 384;
const VB_HEIGHT: i32 = 224;
const TEXTURE_WIDTH: GLfloat = (VB_WIDTH * 2) as GLfloat;
const TEXTURE_HEIGHT: GLfloat = VB_HEIGHT as GLfloat;

const POS_VERTICES: [GLfloat; 8] = [
    -0.5, 0.5,
    -0.5, -0.5,
    0.5, -0.5,
    0.5, 0.5
];

const TEX_VERTICES: [GLfloat; 8] = [
    0.0, 0.0,
    0.0, 1.0,
    1.0, 1.0,
    1.0, 0.0
];

const SQUARE_INDICES: [GLshort; 6] = [0, 1, 2, 0, 2, 3];

const VERTEX_SIZE: GLsizei = 2;
const VERTEX_STRIDE: GLsizei = 0;

pub struct Renderer {
    program_id: GLuint,
    position_location: GLuint,
    tex_coord_location: GLuint,
    modelview_location : GLint,
    texture_location: GLint,
    texture_id: GLuint,
    modelview: Vec<GLfloat>
}

unsafe fn make_shader(type_: GLenum, source: &str) -> Result<GLuint, String> {
    let shader_id = gl::CreateShader(type_);
    let shader_str = c_string!(source)?;
    let shader_source = [shader_str.as_ptr()].as_ptr();
    gl::ShaderSource(shader_id, 1, shader_source, 0 as *const _);
    check_error("load a shader's source")?;
    gl::CompileShader(shader_id);
    check_error("compile a shader")?;
    check_shader(type_, shader_id)?;
    Ok(shader_id)
}

unsafe fn check_shader(type_: GLenum, shader_id: GLuint) -> Result<(), String> {
    let status = gl_temp_array(|ptr| {
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, ptr)
    }) as GLboolean;
    check_error("checking compile status of a shader")?;
    if status == GL_TRUE {
        return Ok(());
    }

    let length = gl_temp_array(|ptr| {
        gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, ptr);
    });
    check_error("finding info log length for a shader")?;
    if length < 0 {
        return Err("Invalid shader info log length")?;
    }
    let mut buf = vec!(0; length as usize);
    let buf_ptr = buf.as_mut_ptr() as *mut GLchar;
    gl::GetShaderInfoLog(shader_id, length, 0 as *mut _, buf_ptr);
    let cstr = CStr::from_bytes_with_nul(buf.as_slice())
        .map_err(|err| { err.to_string() })?;

    let log = cstr.to_str()
        .map_err(|err| { err.to_string() })?;
    Err(format!("Error compiling shader type {:04X}! <{}>", type_, log.trim()))
}

fn check_error(action: &str) -> Result<(), String> {
    let error = unsafe { gl::GetError() };
    match error {
        gl::NO_ERROR => Ok(()),
        _ => Err(format!("OpenGL threw code 0x{:04X} while trying to {}!", error, action))
    }
}

unsafe fn create_gl_texture(data: &[u8]) -> Result<GLuint, String> {
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    let texture_id = gl_temp_array(|ptr| {
        gl::GenTextures(1, ptr);
    });
    check_error("generate a texture")?;
    gl::BindTexture(gl::TEXTURE_2D, texture_id);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as GLint, TEXTURE_WIDTH as GLsizei, TEXTURE_HEIGHT as GLsizei, 0, gl::RGBA, gl::UNSIGNED_BYTE, data.as_voidptr());
    check_error("load a texture")?;

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
    Ok(texture_id)
}

fn as_vec<S: Clone>(mat: cgmath::Matrix4<S>) -> Vec<S> {
    let mut res = Vec::with_capacity(16);
    let rows: [[S;4];4] = mat.into();
    for row in rows.iter() {
        res.extend_from_slice(row);
    }
    res
}

impl Renderer {
    pub fn new(title_screen: &[u8]) -> Result<Renderer, String> {
        let state = unsafe {
            let program_id = gl::CreateProgram();
            check_error("create a program")?;

            let vertex_shader = make_shader(gl::VERTEX_SHADER, VERTEX_SHADER)?;
            gl::AttachShader(program_id, vertex_shader);
            check_error("attach the vertex shader")?;

            let fragment_shader = make_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
            gl::AttachShader(program_id, fragment_shader);
            check_error("attach the fragment shader")?;

            gl::LinkProgram(program_id);
            check_error("link a program")?;
            gl::UseProgram(program_id);
            check_error("build a program")?;

            let position_location = gl::GetAttribLocation(program_id, c_string!("a_Pos")?.as_ptr()) as GLuint;
            let tex_coord_location = gl::GetAttribLocation(program_id, c_string!("a_TexCoord")?.as_ptr()) as GLuint;
            let modelview_location = gl::GetUniformLocation(program_id, c_string!("u_MV")?.as_ptr());
            let texture_location= gl::GetUniformLocation(program_id, c_string!("u_Texture")?.as_ptr());

            let texture_id = create_gl_texture(title_screen)?;

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            check_error("set the clear color")?;

            Renderer {
                program_id,
                position_location,
                tex_coord_location,
                modelview_location,
                texture_location,
                texture_id,
                modelview: as_vec(Matrix4::identity())
            }
        };

        Ok(state)
    }

    pub fn on_surface_changed(&mut self, screen_width: i32, screen_height: i32) {
        unsafe {
            gl::Viewport(0, 0, screen_width, screen_height);
        }
        let hsw = screen_width as GLfloat / 2.0;
        let hsh = screen_height as GLfloat / 2.0;
        let htw = TEXTURE_WIDTH / 2.0;
        let hth = TEXTURE_HEIGHT / 2.0;

        let projection = cgmath::ortho(-hsw, hsw, -hsh, hsh, -100.0, 100.0);

        // The texture should take up as much of the screen as possible
        let scale_to_fit = (hsw / htw).min(hsh / hth);

        let vm = projection * Matrix4::from_nonuniform_scale(TEXTURE_WIDTH * scale_to_fit, TEXTURE_HEIGHT * scale_to_fit, 0.0);

        let bottom_left = vm * vec4(-htw, -hth, 0.0, 1.0);
        let top_right = vm * vec4(htw, hth, 0.0, 1.0);
        debug!("Screen stretches from from {:?} to {:?}", bottom_left, top_right);
        self.modelview = as_vec(vm);
    }

    pub fn on_draw_frame(&self) -> Result<(), String> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            check_error("clear the screen")?;
            gl::UseProgram(self.program_id);
            check_error("use the program")?;

            let pos_pointer = POS_VERTICES.as_voidptr();
            gl::VertexAttribPointer(self.position_location, VERTEX_SIZE, gl::FLOAT, gl::FALSE, VERTEX_STRIDE, pos_pointer);
            check_error("pass position data to the shader")?;

            let tex_pointer = TEX_VERTICES.as_voidptr();
            gl::VertexAttribPointer(self.tex_coord_location, VERTEX_SIZE, gl::FLOAT, gl::FALSE, VERTEX_STRIDE, tex_pointer);
            check_error("pass texture data to the shader")?;

            gl::EnableVertexAttribArray(self.position_location);
            gl::EnableVertexAttribArray(self.tex_coord_location);

            gl::UniformMatrix4fv(self.modelview_location, 1, GL_FALSE, self.modelview.as_ptr());

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);

            gl::Uniform1i(self.texture_location, 0);

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, SQUARE_INDICES.as_voidptr());
            check_error("draw the actual shape")?;

            Ok(())
        }
    }
}
