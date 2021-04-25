use super::gl::Program;
use crate::emulator::video::Frame;
use crate::video::gl::types::GLfloat;

use anyhow::Result;
use cgmath::{self, vec3, Matrix4};
use jni::sys::jobject;
use jni::JNIEnv;

const VB_WIDTH: i32 = 384;
const VB_HEIGHT: i32 = 224;
const EYE_BUFFER_SIZE: usize = (VB_WIDTH * VB_HEIGHT) as usize * 3;

fn base_model_view(screen_size: (i32, i32), tex_size: (i32, i32)) -> Matrix4<f32> {
    let hsw = screen_size.0 as GLfloat / 2.0;
    let hsh = screen_size.1 as GLfloat / 2.0;
    let htw = tex_size.0 as GLfloat / 2.0;
    let hth = tex_size.1 as GLfloat / 2.0;

    let projection = cgmath::ortho(-hsw, hsw, -hsh, hsh, 100.0, -100.0);

    // Scale required to fill the entire screen
    let scale_to_fit = (hsw / htw).min(hsh / hth);
    projection
        * Matrix4::from_nonuniform_scale(
            VB_WIDTH as GLfloat * scale_to_fit,
            VB_HEIGHT as GLfloat * scale_to_fit,
            0.0,
        )
}

pub struct StereoDisplay {
    program: Program,
    buffers: [Vec<u8>; 2],
    settings: Settings,
}
impl StereoDisplay {
    pub fn new(settings: Settings) -> Self {
        Self {
            program: Program::new(2, (VB_WIDTH, VB_HEIGHT)),
            buffers: [vec![0; EYE_BUFFER_SIZE], vec![0; EYE_BUFFER_SIZE]],
            settings,
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.program.init()
    }

    pub fn resize(&mut self, screen_size: (i32, i32)) -> Result<()> {
        let base_mv = base_model_view(screen_size, (VB_WIDTH * 2, VB_HEIGHT));
        let scale = self.settings.screen_zoom;
        let offset = -self.settings.vertical_offset;
        let model_views = vec![
            base_mv
                * Matrix4::from_translation(vec3(-0.5, offset, 0.0))
                * Matrix4::from_scale(scale),
            base_mv
                * Matrix4::from_translation(vec3(0.5, offset, 0.0))
                * Matrix4::from_scale(scale),
        ];
        self.program.resize(screen_size, model_views)
    }

    pub fn update(&mut self, frame: Frame) -> Result<()> {
        let eye = frame.eye as usize;
        let color = self.settings.color;

        let buffer = &mut self.buffers[eye];
        let vb_data = frame.buffer.lock().expect("Buffer lock was poisoned!");
        for i in 0..vb_data.len() {
            // vb_data just has lightness values, convert them to RGB
            buffer[i * 3] = ((vb_data[i] as u16) * (color.0 as u16) / 256) as u8;
            buffer[i * 3 + 1] = ((vb_data[i] as u16) * (color.1 as u16) / 256) as u8;
            buffer[i * 3 + 2] = ((vb_data[i] as u16) * (color.2 as u16) / 256) as u8;
        }
        drop(vb_data); // free the lock ASAP

        self.program.update(eye, buffer)
    }

    pub fn render(&self) -> Result<()> {
        self.program.render()
    }
}

#[derive(Debug)]
pub struct Settings {
    pub screen_zoom: f32,
    pub vertical_offset: f32,
    pub color: (u8, u8, u8),
}

pub fn get_settings(env: &JNIEnv, this: jobject) -> Result<Settings> {
    let screen_zoom_percent = env.get_field(this, "_screenZoom", "I")?.i()?;
    let vertical_offset = env.get_field(this, "_verticalOffset", "I")?.i()?;
    let color = env.get_field(this, "_color", "I")?.i()?;

    // android passes color as ARGB
    let color = ((color >> 16) as u8, (color >> 8) as u8, color as u8);

    Ok(Settings {
        screen_zoom: (screen_zoom_percent as f32) / 100.0,
        vertical_offset: (vertical_offset as f32) / 100.0,
        color,
    })
}
