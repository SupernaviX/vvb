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

use gl::types::{GLboolean,GLint,GLuint,GLchar,GLsizei,GLsizeiptr,GLvoid,GLfloat,GLenum};
use std::ffi::{CStr,CString};
use std::ptr::{null_mut, null};

const GL_TRUE: GLboolean = 1;
const GL_FALSE: GLboolean = 0;

macro_rules! c_string {
    ($string:expr) => {
        CString::new($string).map_err(|_| { "Could not build c string!" })
    };
}

macro_rules! gl_voidptr {
    ($structure:expr) => {
        $structure.as_ptr() as *const GLvoid
    };
}

fn gl_temp_array<T: Copy + Default, F: FnOnce(*mut T) -> ()>(cb: F) -> T {
    let mut tmp_array: [T;1] = Default::default();
    cb(tmp_array.as_mut_ptr());
    tmp_array[0]
}

const VERTEX_SHADER: &str = "
attribute vec2 a_Pos;
uniform mat4 u_Model;
varying mediump vec2 v_TexCoords;

void main() {
    gl_Position = u_Model * vec4(a_Pos, 0.0, 1.0);
    v_TexCoords = a_Pos;
}
";

const FRAGMENT_SHADER: &str = "\
varying mediump vec2 v_TexCoords;
uniform sampler2D u_Texture;

void main() {
    gl_FragColor = texture2D(u_Texture, v_TexCoords);
}
";

const SQUARE_VERTICES: [GLfloat; 8] = [
    -1.0, -1.0,
    1.0, -1.0,
    -1.0, 1.0,
    1.0, 1.0
];

const TEXTURE_VERTICES: [GLfloat; 8] = [
    1.0, 1.0,
    1.0, 0.0,
    0.0, 1.0,
    0.0, 0.0
];

const QUAD_VERTICES: [f32; 12] = [
    0.0, 1.0,
    1.0, 0.0,
    0.0, 0.0,
    0.0, 1.0,
    1.0, 1.0,
    1.0, 0.0
];

const VB_WIDTH: i32 = 384;
const VB_HEIGHT: i32 = 224;
const TEXTURE_SIZE: usize = (VB_WIDTH * VB_HEIGHT * 2 * 3) as usize;

pub struct Renderer {
    texture_id: GLuint,
    texture_data: Vec<GLfloat>,
    vertex_buffer_id: GLuint,
    position_location: GLuint,
    model_location: GLint,
    texture_location: GLint,
    scale: f32,
    offset: (f32, f32)
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

impl Renderer {

    pub fn new() -> Result<Renderer, String> {
        let state = unsafe {
            let program = gl::CreateProgram();
            check_error("create a program")?;

            let vertex_shader = make_shader(gl::VERTEX_SHADER, VERTEX_SHADER)?;
            gl::AttachShader(program, vertex_shader);
            check_error("attach the vertex shader")?;

            let fragment_shader = make_shader(gl::FRAGMENT_SHADER, FRAGMENT_SHADER)?;
            gl::AttachShader(program, fragment_shader);
            check_error("attach the fragment shader")?;

            gl::LinkProgram(program);
            check_error("link a program")?;
            gl::UseProgram(program);
            check_error("build a program")?;

            let position_location = gl::GetAttribLocation(program, c_string!("a_Pos")?.as_ptr()) as u32;
            let model_location = gl::GetUniformLocation(program, c_string!("u_Model")?.as_ptr());
            let texture_location = gl::GetUniformLocation(program, c_string!("u_Texture")?.as_ptr());
            check_error("get uniform location")?;

            let texture_id = gl_temp_array(|ptr| { gl::GenTextures(1, ptr) });
            check_error("generate a texture")?;
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

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, VB_WIDTH, VB_HEIGHT, 0, gl::RGB, gl::UNSIGNED_BYTE, gl_voidptr!(texture_data));
            check_error("load a texture")?;

            let vertex_buffer_id = gl_temp_array(|ptr| { gl::GenBuffers(1, ptr) });
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer_id);
            gl::BufferData(gl::ARRAY_BUFFER, QUAD_VERTICES.len() as GLsizeiptr, gl_voidptr!(QUAD_VERTICES), gl::STATIC_DRAW);
            check_error("load a buffer")?;

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            check_error("set the clear color")?;

            Renderer {
                texture_id,
                texture_data,
                vertex_buffer_id,
                position_location,
                model_location,
                texture_location,
                scale: VB_WIDTH as f32,
                offset: (0.0, 0.0),    
            }
        };

        Ok(state)
    }

    pub fn on_surface_changed(&mut self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }

        let x_scale = width as f32;
        let y_scale = height as f32;

        if (x_scale / VB_WIDTH as f32) < (y_scale / VB_HEIGHT as f32) {
            self.scale = x_scale;
            let y_border = height - (width * VB_HEIGHT);
            self.offset = (0.0, (y_border / 2) as f32);
        } else {
            self.scale = y_scale;
            let x_border = width - (height * VB_WIDTH);
            self.offset = ((x_border / 2) as f32, 0.0);
        }
    }

    pub fn on_draw_frame(&self) -> Result<(), String> {
        let model: [GLfloat; 16] = [
            self.scale, 0.0, 0.0, self.offset.0,
            0.0, self.scale, 0.0, self.offset.1,
            0.0, 0.0, self.scale, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            check_error("clear the screen")?;

            gl::UniformMatrix4fv(self.model_location, 1, GL_FALSE, model.as_ptr());
            check_error("bind the viewmodel")?;

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
            gl::Uniform1i(self.texture_location, 0);
            check_error("bind the texture")?;

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_id);
            gl::VertexAttribPointer(self.position_location, 2, gl::FLOAT, GL_FALSE, 0, gl_voidptr!(QUAD_VERTICES));
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            check_error("draw things")?;
            Ok(())
        }
    }
}
