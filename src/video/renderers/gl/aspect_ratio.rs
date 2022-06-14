use super::utils::{VB_HEIGHT, VB_WIDTH};
use crate::video::gl::types::GLfloat;
use cgmath::Matrix4;
use std::convert::TryFrom;

#[derive(Copy, Clone, Debug)]
pub enum AspectRatio {
    Auto,
    Stretch,
}
impl AspectRatio {
    pub fn compute_mvp_matrix(
        &self,
        screen_size: (i32, i32),
        tex_size: (i32, i32),
    ) -> Matrix4<GLfloat> {
        let hsw = screen_size.0 as GLfloat / 2.0;
        let hsh = screen_size.1 as GLfloat / 2.0;
        let htw = tex_size.0 as GLfloat / 2.0;
        let hth = tex_size.1 as GLfloat / 2.0;

        let projection = cgmath::ortho(-hsw, hsw, -hsh, hsh, 100.0, -100.0);

        let max_scale_width = hsw / htw;
        let max_scale_height = hsh / hth;

        let (scale_width, scale_height) = match self {
            AspectRatio::Auto => {
                let scale_to_fit = max_scale_width.min(max_scale_height);
                (scale_to_fit, scale_to_fit)
            }
            AspectRatio::Stretch => (max_scale_width, max_scale_height),
        };

        projection
            * Matrix4::from_nonuniform_scale(
                VB_WIDTH as GLfloat * scale_width,
                VB_HEIGHT as GLfloat * scale_height,
                0.0,
            )
    }
}
impl TryFrom<i32> for AspectRatio {
    type Error = anyhow::Error;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            x if x == AspectRatio::Auto as i32 => Ok(AspectRatio::Auto),
            x if x == AspectRatio::Stretch as i32 => Ok(AspectRatio::Stretch),
            _ => Err(anyhow::anyhow!("Invalid aspect ratio {}", v)),
        }
    }
}
