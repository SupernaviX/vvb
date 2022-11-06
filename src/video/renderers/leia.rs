use crate::emulator::video::Eye;

use super::common::RenderLogic;
use super::gl::{
    utils::{self, VB_HEIGHT, VB_WIDTH},
    AspectRatio, Program, Textures,
};
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
uniform vec4 u_Colors[2];
uniform sampler2D u_Textures[2];
varying vec2 v_TexCoord;

void main()
{
    //  + alignment_offset
    float view_id = mod(floor(gl_FragCoord.x), 4.0);
    if (view_id < 0.5) { gl_FragColor = texture2D(u_Textures[0], v_TexCoord); }
    else if (view_id < 1.5) { gl_FragColor = texture2D(u_Textures[0], v_TexCoord); }
    else if (view_id < 2.5) { gl_FragColor = texture2D(u_Textures[1], v_TexCoord); }
    else { gl_FragColor = texture2D(u_Textures[1], v_TexCoord); }
    gl_FragColor = mix(u_Colors[1], u_Colors[0], gl_FragColor.g);

    // // scanline
    // vec2 uv = gl_FragCoord.xy / vec2(384.0,224.0);
    // float scanline 	= clamp( 0.95 + 0.05 * cos( 3.14 * ( uv.y + 0.008 ) * 240.0 * 1.0 ), 0.0, 1.0 );
    // float grille 	= 0.85 + 0.15 * clamp( 1.5 * cos( 3.14 * uv.x * 640.0 * 1.0 ), 0.0, 1.0 );
    // gl_FragColor *= scanline * grille * 1.2;
}
";

pub struct LeiaRenderLogic {
    program: Program,
    textures: Textures,

    position_location: GLuint,
    tex_coord_location: GLuint,
    modelview_location: GLint,
    textures_location: GLint,
    colors_location: GLint,

    texture_colors: [[GLfloat; 4]; 2],
    aspect_ratio: AspectRatio,
    transform: Matrix4<GLfloat>,

    enable_3d: bool,
}
impl LeiaRenderLogic {
    pub fn new(settings: &Settings) -> Self {
        let scale = settings.screen_zoom;
        let offset = -settings.vertical_offset;
        Self {
            program: Program::new(VERTEX_SHADER, FRAGMENT_SHADER),
            textures: Textures::new(2, (VB_WIDTH, VB_HEIGHT)),

            position_location: 0,
            tex_coord_location: 0,
            modelview_location: -1,
            textures_location: -1,
            colors_location: -1,

            texture_colors: [
                utils::color_as_vector(settings.colors[0]),
                utils::color_as_vector(settings.colors[1]),
            ],
            aspect_ratio: settings.aspect_ratio,
            transform: Matrix4::from_translation(vec3(0.0, offset, 0.0))
                * Matrix4::from_scale(scale),

            enable_3d: true,
        }
    }
}
impl RenderLogic for LeiaRenderLogic {
    fn init(&mut self) -> Result<()> {
        self.program.init()?;
        self.textures.init()?;

        self.position_location = self.program.get_attribute_location("a_Pos");
        self.tex_coord_location = self.program.get_attribute_location("a_TexCoord");
        self.modelview_location = self.program.get_uniform_location("u_MV");
        self.textures_location = self.program.get_uniform_location("u_Textures");
        self.colors_location = self.program.get_uniform_location("u_Colors");

        // textures and colors don't change, set them here
        self.program
            .set_uniform_vector_array(self.colors_location, &self.texture_colors);

        Ok(())
    }

    fn resize(&mut self, screen_size: (i32, i32)) -> Result<()> {
        self.program.set_viewport(screen_size)?;

        //let base_mv = utils::base_model_view(screen_size, (VB_WIDTH, VB_HEIGHT));
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
        self.textures.update(eye as usize, buffer)
    }
    fn draw(&self) -> Result<()> {
        let texture_ids = [
            self.textures.ids[0],
            self.textures.ids[if self.enable_3d { 1 } else { 0 }],
        ];
        self.program
            .set_uniform_texture_array(self.textures_location, &texture_ids);

        self.program.start_render()?;
        self.program
            .draw_square(self.position_location, self.tex_coord_location)
    }
}

pub struct Settings {
    screen_zoom: f32,
    pub aspect_ratio: AspectRatio,
    vertical_offset: f32,
    colors: [(u8, u8, u8); 2],
}

#[rustfmt::skip::macros(jni_func)]
pub mod jni {
    use super::{LeiaRenderLogic, Settings};
    use crate::emulator::Emulator;
    use crate::jni_helpers::EnvExtensions;
    use crate::video::renderers::common::Renderer;
    use crate::{jni_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::{jboolean, jint, jobject};
    use jni::JNIEnv;

    type LeiaRenderer = Renderer<LeiaRenderLogic>;

    fn get_settings(env: &JNIEnv, this: jobject) -> Result<Settings> {
        let screen_zoom = env.get_percent(this, "screenZoom")?;
        let aspect_ratio = env.get_int(this, "aspectRatio")?.try_into()?;
        let vertical_offset = env.get_percent(this, "verticalOffset")?;
        let colors = [
            env.get_color(this, "color")?,
            env.get_color(this, "colorBG")?,
        ];
        Ok(Settings {
            screen_zoom,
            aspect_ratio,
            vertical_offset,
            colors,
        })
    }

    fn get_renderer<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, LeiaRenderer> {
        jni_helpers::java_get(env, this)
    }

    jni_func!(LeiaRenderer_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(env, emulator)?;
        let settings = get_settings(env, settings)?;
        let renderer = Renderer::new(
            emulator.claim_frame_buffer_consumers(),
            LeiaRenderLogic::new(&settings),
        );
        jni_helpers::java_init(env, this, renderer)
    }

    jni_func!(LeiaRenderer_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<LeiaRenderer>(env, this)
    }

    jni_func!(LeiaRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    jni_func!(LeiaRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(env: &JNIEnv, this: jobject, width: jint, height: jint) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    jni_func!(LeiaRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }

    jni_func!(LeiaRenderer_nativeOnModeChanged, on_mode_changed, jboolean);
    fn on_mode_changed(env: &JNIEnv, this: jobject, enable_3d: jboolean) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.logic.enable_3d = enable_3d != 0;
        Ok(())
    }
}
