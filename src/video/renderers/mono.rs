use super::common::RenderLogic;
use super::gl::utils::{VB_HEIGHT, VB_WIDTH};
use super::gl::{utils, AspectRatio, Program, Textures};
use crate::emulator::video::Eye;
use crate::video::gl::types::{GLfloat, GLint, GLuint};
use anyhow::Result;
use cgmath::{vec3, Matrix4};

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

pub struct MonoRenderLogic {
    program: Program,
    textures: Textures,

    position_location: GLuint,
    tex_coord_location: GLuint,
    modelview_location: GLint,
    texture_location: GLint,
    color_location: GLint,

    eye: Eye,
    texture_color: [GLfloat; 4],
    aspect_ratio: AspectRatio,
    transform: Matrix4<GLfloat>,
}
impl MonoRenderLogic {
    pub fn new(settings: &Settings) -> Self {
        let zoom = settings.screen_zoom;
        let offset = -settings.vertical_offset;
        Self {
            program: Program::new(VERTEX_SHADER, FRAGMENT_SHADER),
            textures: Textures::new(1, (VB_WIDTH, VB_HEIGHT)),

            position_location: 0,
            tex_coord_location: 0,
            modelview_location: -1,
            texture_location: -1,
            color_location: -1,

            eye: settings.eye,
            texture_color: utils::color_as_vector(settings.color),
            aspect_ratio: settings.aspect_ratio,
            transform: Matrix4::from_translation(vec3(0.0, offset, 0.0))
                * Matrix4::from_scale(zoom),
        }
    }
}
impl RenderLogic for MonoRenderLogic {
    fn init(&mut self) -> Result<()> {
        self.program.init()?;
        self.textures.init()?;

        self.position_location = self.program.get_attribute_location("a_Pos");
        self.tex_coord_location = self.program.get_attribute_location("a_TexCoord");
        self.modelview_location = self.program.get_uniform_location("u_MV");
        self.texture_location = self.program.get_uniform_location("u_Texture");
        self.color_location = self.program.get_uniform_location("u_Color");

        // texture and color don't change, set them here
        self.program
            .set_uniform_texture(self.texture_location, self.textures.ids[0]);
        self.program
            .set_uniform_vector(self.color_location, &self.texture_color);

        Ok(())
    }

    fn resize(&mut self, screen_size: (i32, i32)) -> Result<()> {
        self.program.set_viewport(screen_size)?;

        let base_mv = self
            .aspect_ratio
            .compute_mvp_matrix(screen_size, (VB_WIDTH, VB_HEIGHT));
        let model_view = utils::to_matrix(base_mv * self.transform);

        // model view only changes when the surface is resized, set it here
        self.program
            .set_uniform_matrix(self.modelview_location, &model_view);

        Ok(())
    }

    fn update(&mut self, eye: Eye, buffer: &[u8]) -> Result<()> {
        if eye == self.eye {
            self.textures.update(0, buffer)
        } else {
            Ok(())
        }
    }
    fn draw(&self) -> Result<()> {
        self.program.start_render()?;
        self.program
            .draw_square(self.position_location, self.tex_coord_location)
    }
}

pub struct Settings {
    eye: Eye,
    screen_zoom: f32,
    aspect_ratio: AspectRatio,
    vertical_offset: f32,
    color: (u8, u8, u8),
}

#[rustfmt::skip::macros(jni_func)]
pub mod jni {
    use super::{MonoRenderLogic, Settings};
    use crate::emulator::jni::get_emulator;
    use crate::jni_helpers::{JavaBinding, JavaGetResult};
    use crate::video::renderers::common::Renderer;
    use crate::{jni_func, EnvExtensions};
    use anyhow::Result;
    use jni::objects::JObject;
    use jni::sys::jint;
    use jni::JNIEnv;
    use std::convert::TryInto;

    type MonoRenderer = Renderer<MonoRenderLogic>;

    static MONO_BINDING: JavaBinding<MonoRenderer> = JavaBinding::new();

    fn get_settings<'a>(env: &mut JNIEnv<'a>, this: JObject<'a>) -> Result<Settings> {
        let eye = env.get_int(&this, "eye")?.try_into()?;
        let screen_zoom = env.get_percent(&this, "screenZoom")?;
        let aspect_ratio = env.get_int(&this, "aspectRatio")?.try_into()?;
        let vertical_offset = env.get_percent(&this, "verticalOffset")?;
        let color = env.get_color(&this, "color")?;
        Ok(Settings {
            eye,
            aspect_ratio,
            screen_zoom,
            vertical_offset,
            color,
        })
    }

    fn get_renderer<'a>(env: &'a mut JNIEnv, this: JObject<'a>) -> JavaGetResult<'a, MonoRenderer> {
        MONO_BINDING.get_value(env, this)
    }

    jni_func!(MonoRenderer_nativeConstructor, constructor, JObject<'a>, JObject<'a>);
    fn constructor<'a>(
        env: &mut JNIEnv<'a>,
        this: JObject<'a>,
        emulator: JObject<'a>,
        settings: JObject<'a>,
    ) -> Result<()> {
        let settings = get_settings(env, settings)?;
        let renderer = {
            let mut emulator = get_emulator(env, emulator)?;
            Renderer::new(
                emulator.claim_frame_buffer_consumers(),
                MonoRenderLogic::new(&settings),
            )
        };
        MONO_BINDING.init_value(env, this, renderer)
    }

    jni_func!(MonoRenderer_nativeDestructor, destructor);
    fn destructor(env: &mut JNIEnv, this: JObject) -> Result<()> {
        MONO_BINDING.drop_value(env, this)
    }

    jni_func!(MonoRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    jni_func!(MonoRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(
        env: &mut JNIEnv,
        this: JObject,
        width: jint,
        height: jint,
    ) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    jni_func!(MonoRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }
}
