use super::common::RenderLogic;
use super::gl::{
    utils::{self, VB_HEIGHT, VB_WIDTH},
    AspectRatio, Program, Textures,
};
use crate::emulator::video::Eye;
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

pub struct StereoRenderLogic {
    program: Program,
    textures: Textures,

    position_location: GLuint,
    tex_coord_location: GLuint,
    modelview_location: GLint,
    texture_location: GLint,
    color_location: GLint,

    texture_color: [GLfloat; 4],
    aspect_ratio: AspectRatio,
    transforms: [Matrix4<GLfloat>; 2],
    model_views: [[GLfloat; 16]; 2],
}
impl StereoRenderLogic {
    pub fn new(settings: &Settings) -> Self {
        let zoom = settings.screen_zoom;
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
            aspect_ratio: settings.aspect_ratio,
            transforms: [
                Matrix4::from_translation(vec3(-0.5, offset, 0.0)) * Matrix4::from_scale(zoom),
                Matrix4::from_translation(vec3(0.5, offset, 0.0)) * Matrix4::from_scale(zoom),
            ],
            model_views: [utils::identity_matrix(), utils::identity_matrix()],
        }
    }
}

impl RenderLogic for StereoRenderLogic {
    fn init(&mut self) -> Result<()> {
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

    fn resize(&mut self, screen_size: (i32, i32)) -> Result<()> {
        self.program.set_viewport(screen_size)?;

        let base_mv = self
            .aspect_ratio
            .compute_mvp_matrix(screen_size, (VB_WIDTH * 2, VB_HEIGHT));
        self.model_views = [
            utils::to_matrix(base_mv * self.transforms[0]),
            utils::to_matrix(base_mv * self.transforms[1]),
        ];

        Ok(())
    }

    fn update(&mut self, eye: Eye, buffer: &[u8]) -> Result<()> {
        self.textures.update(eye as usize, buffer)
    }

    fn draw(&self) -> Result<()> {
        self.program.start_render()?;
        for i in 0..self.textures.ids.len() {
            let texture_id = self.textures.ids[i];
            let model_view = &self.model_views[i];

            self.program
                .set_uniform_texture(self.texture_location, texture_id);
            self.program
                .set_uniform_matrix(self.modelview_location, model_view);

            self.program
                .draw_square(self.position_location, self.tex_coord_location)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Settings {
    pub screen_zoom: f32,
    pub aspect_ratio: AspectRatio,
    pub vertical_offset: f32,
    pub color: (u8, u8, u8),
}

#[rustfmt::skip::macros(jni_func)]
pub mod jni {
    use super::{Settings, StereoRenderLogic};
    use crate::emulator::Emulator;
    use crate::jni_helpers::EnvExtensions;
    use crate::video::renderers::common::Renderer;
    use crate::{jni_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;
    use std::convert::TryInto;

    type StereoRenderer = Renderer<StereoRenderLogic>;

    pub fn get_settings(env: &JNIEnv, this: jobject) -> Result<Settings> {
        let screen_zoom = env.get_percent(this, "screenZoom")?;
        let aspect_ratio = env.get_int(this, "aspectRatio")?.try_into()?;
        let vertical_offset = env.get_percent(this, "verticalOffset")?;
        let color = env.get_color(this, "color")?;

        Ok(Settings {
            screen_zoom,
            aspect_ratio,
            vertical_offset,
            color,
        })
    }

    fn get_renderer<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, StereoRenderer> {
        jni_helpers::java_get(env, this)
    }

    jni_func!(StereoRenderer_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(env, emulator)?;
        let settings = get_settings(env, settings)?;
        let renderer = Renderer::new(
            emulator.get_frame_channel(),
            StereoRenderLogic::new(&settings),
        );
        jni_helpers::java_init(env, this, renderer)
    }

    jni_func!(StereoRenderer_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<StereoRenderer>(env, this)
    }

    jni_func!(StereoRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    jni_func!(StereoRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(env: &JNIEnv, this: jobject, width: jint, height: jint) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    jni_func!(StereoRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }
}
