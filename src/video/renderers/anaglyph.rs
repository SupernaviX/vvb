use crate::emulator::video::FrameChannel;

use super::gl::{utils, Program, Textures};
use crate::video::gl::types::{GLfloat, GLint, GLuint};
use super::gl::utils::{VB_HEIGHT, VB_WIDTH};
use anyhow::Result;
use cgmath::{vec3, Matrix4};
use std::sync::mpsc::TryRecvError;

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
uniform sampler2D u_Textures[2];
uniform vec4 u_Colors[2];
void main() {
    vec4 left = u_Colors[0] * texture2D(u_Textures[0], v_TexCoord).r;
    vec4 right = u_Colors[1] * texture2D(u_Textures[1], v_TexCoord).r;
    gl_FragColor = left + right;
}
";

pub struct AnaglyphRenderer {
    frame_channel: FrameChannel,
    program: Program,
    textures: Textures,

    position_location: GLuint,
    tex_coord_location: GLuint,
    modelview_location: GLint,
    textures_location: GLint,
    colors_location: GLint,

    texture_colors: [[GLfloat; 4]; 2],
    transform: Matrix4<GLfloat>,
}
impl AnaglyphRenderer {
    pub fn new(frame_channel: FrameChannel, settings: &Settings) -> Self {
        let scale = settings.screen_zoom;
        let offset = -settings.vertical_offset;
        AnaglyphRenderer {
            frame_channel,
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
            transform: Matrix4::from_translation(vec3(0.0, offset, 0.0))
                * Matrix4::from_scale(scale),
        }
    }

    pub fn on_surface_created(&mut self) -> Result<()> {
        self.program.init()?;
        self.textures.init()?;

        self.position_location = self.program.get_attribute_location("a_Pos");
        self.tex_coord_location = self.program.get_attribute_location("a_TexCoord");
        self.modelview_location = self.program.get_uniform_location("u_MV");
        self.textures_location = self.program.get_uniform_location("u_Textures");
        self.colors_location = self.program.get_uniform_location("u_Colors");

        // textures and colors don't change, set them here
        self.program
            .set_uniform_texture_array(self.textures_location, &self.textures.ids);
        self.program
            .set_uniform_vector_array(self.colors_location, &self.texture_colors);

        Ok(())
    }

    pub fn on_surface_changed(&mut self, width: i32, height: i32) -> Result<()> {
        self.program.set_viewport((width, height))?;

        let base_mv = utils::base_model_view((width, height), (VB_WIDTH, VB_HEIGHT));
        let model_view = utils::to_matrix(base_mv * self.transform);

        // model view only changes when the surface is resized, set it here
        self.program
            .set_uniform_matrix(self.modelview_location, &model_view);

        Ok(())
    }

    pub fn on_draw_frame(&mut self) -> Result<()> {
        self.update_screen()?;
        self.program.start_render()?;
        self.program
            .draw_square(self.position_location, self.tex_coord_location)
    }

    fn update_screen(&mut self) -> Result<()> {
        match self.frame_channel.try_recv() {
            Ok(frame) => {
                let eye = frame.eye as usize;
                let vb_data = frame.buffer.lock().expect("Buffer lock was poisoned!");
                self.textures.update(eye, &vb_data)
            }
            Err(TryRecvError::Empty) => Ok(()),
            Err(TryRecvError::Disconnected) => Err(anyhow::anyhow!("Emulator has shut down")),
        }
    }
}

pub struct Settings {
    screen_zoom: f32,
    vertical_offset: f32,
    colors: [(u8, u8, u8); 2],
}

#[rustfmt::skip::macros(emulator_func)]
pub mod jni {
    use super::{AnaglyphRenderer, Settings};
    use crate::emulator::Emulator;
    use crate::jni_helpers::EnvExtensions;
    use crate::{emulator_func, jni_helpers};
    use anyhow::Result;
    use jni::sys::{jint, jobject};
    use jni::JNIEnv;

    fn get_settings(env: &JNIEnv, this: jobject) -> Result<Settings> {
        let screen_zoom = env.get_percent(this, "_screenZoom")?;
        let vertical_offset = env.get_percent(this, "_verticalOffset")?;
        let colors = [(0xff, 0x00, 0x00), (0x00, 0xff, 0xff)];
        Ok(Settings {
            screen_zoom,
            vertical_offset,
            colors,
        })
    }

    fn get_renderer<'a>(
        env: &'a JNIEnv,
        this: jobject,
    ) -> jni_helpers::JavaGetResult<'a, AnaglyphRenderer> {
        jni_helpers::java_get(env, this)
    }

    emulator_func!(AnaglyphRenderer_nativeConstructor, constructor, jobject, jobject);
    fn constructor(
        env: &JNIEnv,
        this: jobject,
        emulator: jobject,
        settings: jobject,
    ) -> Result<()> {
        let mut emulator = jni_helpers::java_get::<Emulator>(&env, emulator)?;
        let settings = get_settings(&env, settings)?;
        let renderer = AnaglyphRenderer::new(emulator.get_frame_channel(), &settings);
        jni_helpers::java_init(env, this, renderer)
    }

    emulator_func!(AnaglyphRenderer_nativeDestructor, destructor);
    fn destructor(env: &JNIEnv, this: jobject) -> Result<()> {
        jni_helpers::java_take::<AnaglyphRenderer>(env, this)
    }

    emulator_func!(AnaglyphRenderer_nativeOnSurfaceCreated, on_surface_created);
    fn on_surface_created(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_created()
    }

    emulator_func!(AnaglyphRenderer_nativeOnSurfaceChanged, on_surface_changed, jint, jint);
    fn on_surface_changed(env: &JNIEnv, this: jobject, width: jint, height: jint) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_surface_changed(width, height)
    }

    emulator_func!(AnaglyphRenderer_nativeOnDrawFrame, on_draw_frame);
    fn on_draw_frame(env: &JNIEnv, this: jobject) -> Result<()> {
        let mut this = get_renderer(env, this)?;
        this.on_draw_frame()
    }
}
