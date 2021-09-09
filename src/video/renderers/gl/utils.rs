use crate::video::gl::types::GLfloat;
use cgmath::{Matrix4, SquareMatrix};

pub const VB_WIDTH: i32 = 384;
pub const VB_HEIGHT: i32 = 224;

pub fn color_as_vector(color: (u8, u8, u8)) -> [GLfloat; 4] {
    [
        color.0 as GLfloat / 255.0,
        color.1 as GLfloat / 255.0,
        color.2 as GLfloat / 255.0,
        1.0,
    ]
}

pub fn to_matrix(mat: Matrix4<GLfloat>) -> [GLfloat; 16] {
    let rows: [[GLfloat; 4]; 4] = mat.into();
    // safety: pretty sure this is how arrays work
    unsafe { std::mem::transmute(rows) }
}

pub fn identity_matrix() -> [GLfloat; 16] {
    to_matrix(Matrix4::identity())
}
