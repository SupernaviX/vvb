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
    v_TexCoord = vec2(a_TexCoord.x, 1.0 - a_TexCoord.y);
}
";

const FRAGMENT_SHADER: &str = "\
precision mediump float;
varying vec2 v_TexCoord;
uniform sampler2D u_Texture;
uniform vec4 u_Colors[2];
void main() {
    gl_FragColor = texture2D(u_Texture, v_TexCoord);
    gl_FragColor = mix(u_Colors[1], u_Colors[0], gl_FragColor.g);
}
";

pub struct CNSDKRenderLogic {
    program: Program,
    textures: Textures,

    position_location: GLuint,
    tex_coord_location: GLuint,
    modelview_location: GLint,
    texture_location: GLint,
    colors_location: GLint,

    texture_colors: [[GLfloat; 4]; 2],
    aspect_ratio: AspectRatio,
    transforms: [Matrix4<GLfloat>; 2],
    model_views: [[GLfloat; 16]; 2],
}
impl CNSDKRenderLogic {
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
            colors_location: -1,

            texture_colors: [
                utils::color_as_vector(settings.colors[0]),
                utils::color_as_vector(settings.colors[1]),
            ],
            aspect_ratio: settings.aspect_ratio,
            transforms: [
                Matrix4::from_translation(vec3(-0.5, offset, 0.0)) * Matrix4::from_scale(zoom),
                Matrix4::from_translation(vec3(0.5, offset, 0.0)) * Matrix4::from_scale(zoom),
            ],
            model_views: [utils::identity_matrix(), utils::identity_matrix()],
        }
    }
}

impl RenderLogic for CNSDKRenderLogic {
    fn init(&mut self) -> Result<()> {
        self.program.init()?;
        self.textures.init()?;

        self.position_location = self.program.get_attribute_location("a_Pos");
        self.tex_coord_location = self.program.get_attribute_location("a_TexCoord");
        self.modelview_location = self.program.get_uniform_location("u_MV");
        self.texture_location = self.program.get_uniform_location("u_Texture");
        self.colors_location = self.program.get_uniform_location("u_Colors");

        // Set colors here, because they're the same for the entire life of the program
        self.program
            .set_uniform_vector_array(self.colors_location, &self.texture_colors);

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
    pub colors: [(u8, u8, u8); 2],
}

#[rustfmt::skip::macros(jni_func)]
pub mod jni {
    use super::{CNSDKRenderLogic, Settings};
    use crate::emulator::jni::get_emulator;
    use crate::jni_helpers::{JavaBinding, JavaGetResult};
    use crate::video::renderers::common::Renderer;
    use crate::{jni_func, EnvExtensions};
    use anyhow::Result;
    use jni::objects::JObject;
    use jni::sys::jint;
    use jni::JNIEnv;
    use std::convert::TryInto;

    type CNSDKRenderer = Renderer<CNSDKRenderLogic>;

    static CNSDK_BINDING: JavaBinding<CNSDKRenderer> = JavaBinding::new();

    pub fn get_settings<'a>(env: &mut JNIEnv<'a>, this: JObject<'a>) -> Result<Settings> {
        let screen_zoom = env.get_percent(&this, "screenZoom")?;
        let aspect_ratio = env.get_int(&this, "aspectRatio")?.try_into()?;
        let vertical_offset = env.get_percent(&this, "verticalOffset")?;
        let colors = [
            env.get_color(&this, "color")?,
            env.get_color(&this, "colorBG")?,
        ];

        Ok(Settings {
            screen_zoom,
            aspect_ratio,
            vertical_offset,
            colors,
        })
    }

    fn get_renderer<'a>(
        env: &'a mut JNIEnv,
        this: JObject<'a>,
    ) -> JavaGetResult<'a, CNSDKRenderer> {
        CNSDK_BINDING.get_value(env, this)
    }

    jni_func!(CNSDKRenderer_nativeConstructor, constructor, JObject<'a>, JObject<'a>);
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
                CNSDKRenderLogic::new(&settings),
            )
        };
        CNSDK_BINDING.init_value(env, this, renderer)
    }

    jni_func!(CNSDKRenderer_nativeDestructor, destructor);
    fn destructor(env: &mut JNIEnv, this: JObject) -> Result<()> {
        CNSDK_BINDING.drop_value(env, this)
    }

    jni_func!(CNSDKRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    jni_func!(CNSDKRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(
        env: &mut JNIEnv,
        this: JObject,
        width: jint,
        height: jint,
    ) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    jni_func!(CNSDKRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &mut JNIEnv, this: JObject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }
}
