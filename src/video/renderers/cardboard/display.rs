use super::super::gl::{
    utils::{self, VB_HEIGHT, VB_WIDTH},
    Program, Textures,
};
use crate::emulator::video::Frame;
use crate::video::gl::types::{GLfloat, GLint, GLuint};

use anyhow::Result;
use cgmath::{self, vec3, Matrix4};

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

pub struct StereoDisplay {
    program: Program,
    textures: Textures,

    position_location: GLuint,
    tex_coord_location: GLuint,
    modelview_location: GLint,
    texture_location: GLint,
    color_location: GLint,

    texture_color: [GLfloat; 4],
    transforms: [Matrix4<GLfloat>; 2],
    model_views: [[GLfloat; 16]; 2],
}
impl StereoDisplay {
    pub fn new(settings: &Settings) -> Self {
        let scale = settings.screen_zoom;
        let offset = -settings.vertical_offset;
        Self {
            program: Program::new(VERTEX_SHADER, FRAGMENT_SHADER),
            textures: Textures::new(2, (VB_WIDTH, VB_HEIGHT)),

            position_location: 0,
            tex_coord_location: 0,
            modelview_location: -1,
            texture_location: -1,
            color_location: -1,

            texture_color: utils::color_as_vector(settings.color),
            transforms: [
                Matrix4::from_translation(vec3(-0.5, offset, 0.0)) * Matrix4::from_scale(scale),
                Matrix4::from_translation(vec3(0.5, offset, 0.0)) * Matrix4::from_scale(scale),
            ],
            model_views: [utils::identity_matrix(), utils::identity_matrix()],
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.program.init()?;
        self.textures.init()?;

        self.position_location = self.program.get_attribute_location("a_Pos");
        self.tex_coord_location = self.program.get_attribute_location("a_TexCoord");
        self.modelview_location = self.program.get_uniform_location("u_MV");
        self.texture_location = self.program.get_uniform_location("u_Texture");
        self.color_location = self.program.get_uniform_location("u_Color");

        // Set color here, because it's the same for the entire life of the program
        self.program
            .set_uniform_vector(self.color_location, &self.texture_color);

        Ok(())
    }

    pub fn resize(&mut self, screen_size: (i32, i32)) -> Result<()> {
        self.program.set_viewport(screen_size)?;

        let base_mv = utils::base_model_view(screen_size, (VB_WIDTH * 2, VB_HEIGHT));
        self.model_views = [
            utils::to_matrix(base_mv * self.transforms[0]),
            utils::to_matrix(base_mv * self.transforms[1]),
        ];

        Ok(())
    }

    pub fn update(&mut self, frame: Frame) -> Result<()> {
        let eye = frame.eye as usize;
        let vb_data = frame.buffer.lock().expect("Buffer lock was poisoned!");
        self.textures.update(eye, &vb_data)
    }

    pub fn render(&self) -> Result<()> {
        self.program.start_render()?;
        for i in 0..self.textures.ids.len() {
            self.render_texture(i)?;
        }
        Ok(())
    }

    fn render_texture(&self, index: usize) -> Result<()> {
        let texture_id = self.textures.ids[index];
        let model_view = &self.model_views[index];

        self.program
            .set_uniform_texture(self.texture_location, texture_id);
        self.program
            .set_uniform_matrix(self.modelview_location, model_view);

        self.program
            .draw_square(self.position_location, self.tex_coord_location)
    }
}

#[derive(Debug)]
pub struct Settings {
    pub screen_zoom: f32,
    pub vertical_offset: f32,
    pub color: (u8, u8, u8),
}
