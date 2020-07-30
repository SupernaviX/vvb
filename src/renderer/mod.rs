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
use std::ffi::{CString};

const GL_TRUE: GLboolean = 0;
const GL_FALSE: GLboolean = 0;

macro_rules! c_string {
    ($string:expr) => {
        CString::new($string).map_err(|_| { "Could not build c string!" })
    };
}

macro_rules! gl_gen {
    ($func:ident, $type:ty, $action:literal) => {
        {
            let mut temp_array: [$type;1] = [0];
            gl::$func(1, temp_array.as_mut_ptr());
            check_error($action)?;
            temp_array[0]
        }
    };
}

macro_rules! gl_voidptr {
    ($structure:expr) => {
        $structure.as_ptr() as *const GLvoid
    };
}

const VERTEX_SHADER: &str = "
layout (location = 0) attribute vec2 a_Pos
uniform mat4 u_Model;
varying vec2 v_TexCoords;

void main() {
    gl_Position = u_Model * vec4(a_Pos, 0.0, 1.0);
    v_TexCoords = i_Pos;
}
";

const FRAGMENT_SHADER: &str = "\
varying vec2 v_TexCoords;
uniform sampler2D u_Texture;
out vec4 o_Color;

void main() {
    o_Color = texture(u_Texture, v_TexCoords);
}
";

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
    texture_id: u32,
    texture_data: Vec<f32>,
    vertex_buffer_id: u32,
    position_location: u32,
    model_location: i32,
    scale: f32,
    offset: (f32, f32)
}

unsafe fn make_shader(type_: GLenum, source: &str) -> Result<GLuint, String> {
    let shader_id = gl::CreateShader(type_);
    let shader_source = [c_string!(source)?.as_ptr()].as_ptr();
    let shader_length = [source.len() as GLint].as_ptr();
    gl::ShaderSource(shader_id, 1, shader_source, shader_length);
    check_error("load a shader's source")?;
    gl::CompileShader(shader_id);
    check_error("compile a shader")?;
    Ok(shader_id)
}

fn check_error(action: &str) -> Result<(), String> {
    let error = unsafe { gl::GetError() };
    match error {
        gl::NO_ERROR => Ok(()),
        _ => Err(format!("OpenGL threw code {} while trying to {}!", error, action))
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
            check_error("get uniform location")?;

            let texture_id = gl_gen!(GenTextures, GLuint, "generate a texture");
            let mut texture_data = vec![0.0; TEXTURE_SIZE];
            // TODO: load texture
            for r in (0..TEXTURE_SIZE).step_by(3) {
                let i = r / 3;
                let checker_x = (i / 16) % 2;
                let checker_y = 1 - ((i / 224) / 16);
                if checker_x != checker_y {
                    texture_data[r] = 1.0;
                }
            }

            gl::BindTexture(gl::TEXTURE_2D, texture_id);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as i32, VB_WIDTH, VB_HEIGHT, 0, gl::RGBA, gl::UNSIGNED_BYTE, gl_voidptr!(texture_data));
            check_error("load a texture")?;

            let vertex_buffer_id = gl_gen!(GenBuffers, GLuint, "generate a buffer");
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

    pub fn on_draw_frame(&self) {
        let model: [GLfloat; 16] = [
            self.scale, 0.0, 0.0, self.offset.0,
            0.0, self.scale, 0.0, self.offset.1,
            0.0, 0.0, self.scale, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UniformMatrix4fv(self.model_location, 1, GL_FALSE, model.as_ptr());

            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_id);
            gl::VertexAttribPointer(self.position_location, 2, gl::FLOAT, GL_FALSE, 0, gl_voidptr!(QUAD_VERTICES));
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }
    }
}
