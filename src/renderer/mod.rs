#[cfg(target_os = "android")]
mod gl {
    #[link(name="GLESv2")]
    extern {}
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

#[cfg(target_os = "windows")]
mod gl_bindings;
#[cfg(target_os = "windows")]
mod gl {
    pub use super::gl_bindings::*;
}

use gl::types::{GLboolean,GLuint,GLsizei,GLchar,GLvoid,GLfloat,GLenum};
use std::ffi::{CStr,CString};
use std::ptr::{null_mut, null};
// use log::{debug};

const GL_TRUE: GLboolean = 1;
// const GL_FALSE: GLboolean = 0;

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

const VERTEX_SHADER: &str = "
attribute vec4 a_Pos;
void main() {
    gl_Position = a_Pos;
}
";

const FRAGMENT_SHADER: &str = "\
precision mediump float;
void main() {
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}
";

const SQUARE_VERTICES: [GLfloat; 18] = [
    0.5, 0.5, 0.0,
    0.5, -0.5, 0.0,
    -0.5, 0.5, 0.0,

    0.5, -0.5, 0.0,
    -0.5, -0.5, 0.0,
    -0.5, 0.5, 0.0
];

const VERTEX_SIZE: GLsizei = 3;

const VB_WIDTH: i32 = 384;
const VB_HEIGHT: i32 = 224;
const TEXTURE_SIZE: usize = (VB_WIDTH * VB_HEIGHT * 2 * 3) as usize;

pub struct Renderer {
    program_id: GLuint,
    position_location: GLuint,
}

unsafe fn make_shader(type_: GLenum, source: &str) -> Result<GLuint, String> {
    let shader_id = gl::CreateShader(type_);
    let shader_str = c_string!(source)?;
    let shader_source = [shader_str.as_ptr()].as_ptr();
    gl::ShaderSource(shader_id, 1, shader_source, null());
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
    gl::GetShaderInfoLog(shader_id, length, null_mut(), buf_ptr);
    let cstr = CStr::from_bytes_with_nul(buf.as_slice())
        .map_err(|err| { err.to_string() })?;

    let log = cstr.to_str()
        .map_err(|err| { err.to_string() })?;
    Err(format!("Error compiling shader type {}! <{}>", type_, log))
}

fn check_error(action: &str) -> Result<(), String> {
    let error = unsafe { gl::GetError() };
    match error {
        gl::NO_ERROR => Ok(()),
        _ => Err(format!("OpenGL threw code 0x{:04X} while trying to {}!", error, action))
    }
}

#[allow(dead_code)]
fn load_texture() -> Result<Vec<GLfloat>, String> {
    let mut texture_data = vec![0.0; TEXTURE_SIZE];
    // TODO: load texture
    for r in (0..TEXTURE_SIZE).step_by(3) {
        let i = r / 3;
        let checker_x = (i / 16) % 2;
        let checker_y = 1 - ((i / 224) / 16) % 2;
        if checker_x != checker_y {
            texture_data[r] = 1.0;
        }
    }
    Ok(texture_data)
}

impl Renderer {

    pub fn new() -> Result<Renderer, String> {
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

            let position_location = gl::GetAttribLocation(program_id, c_string!("a_Pos")?.as_ptr()) as u32;

            gl::ClearColor(0.0, 0.0, 1.0, 1.0);
            check_error("set the clear color")?;

            Renderer {
                program_id,
                position_location,
            }
        };

        Ok(state)
    }

    pub fn on_surface_changed(&mut self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    pub fn on_draw_frame(&self) -> Result<(), String> {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            check_error("clear the screen")?;
            gl::UseProgram(self.program_id);
            check_error("use the program")?;

            gl::VertexAttribPointer(self.position_location, VERTEX_SIZE, gl::FLOAT, gl::FALSE, 0, SQUARE_VERTICES.as_ptr() as *const GLvoid);
            check_error("pass position data to the shader")?;
            gl::EnableVertexAttribArray(self.position_location);
            check_error("enable the VAO for position data")?;

            gl::DrawArrays(gl::TRIANGLES, 0, SQUARE_VERTICES.len() as GLsizei / VERTEX_SIZE);
            check_error("draw the actual shape")?;

            Ok(())
        }
    }
}
