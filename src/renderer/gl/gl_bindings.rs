// This is the output of build.rs, but with Generator::Global instead of Generator::Static.
// It's copied here so windows builds and autocomplete can work.

#![cfg(target_os = "windows")]

mod __gl_imports {
    pub use std::mem;
    pub use std::os::raw;
}

#[inline(never)]
fn metaloadfn(
    loadfn: &mut dyn FnMut(&'static str) -> *const __gl_imports::raw::c_void,
    symbol: &'static str,
    fallbacks: &[&'static str],
) -> *const __gl_imports::raw::c_void {
    let mut ptr = loadfn(symbol);
    if ptr.is_null() {
        for &sym in fallbacks {
            ptr = loadfn(sym);
            if !ptr.is_null() {
                break;
            }
        }
    }
    ptr
}

pub mod types {
    #![allow(
        non_camel_case_types,
        non_snake_case,
        dead_code,
        missing_copy_implementations
    )]

    // Common types from OpenGL 1.1
    pub type GLenum = super::__gl_imports::raw::c_uint;
    pub type GLboolean = super::__gl_imports::raw::c_uchar;
    pub type GLbitfield = super::__gl_imports::raw::c_uint;
    pub type GLvoid = super::__gl_imports::raw::c_void;
    pub type GLbyte = super::__gl_imports::raw::c_char;
    pub type GLshort = super::__gl_imports::raw::c_short;
    pub type GLint = super::__gl_imports::raw::c_int;
    pub type GLclampx = super::__gl_imports::raw::c_int;
    pub type GLubyte = super::__gl_imports::raw::c_uchar;
    pub type GLushort = super::__gl_imports::raw::c_ushort;
    pub type GLuint = super::__gl_imports::raw::c_uint;
    pub type GLsizei = super::__gl_imports::raw::c_int;
    pub type GLfloat = super::__gl_imports::raw::c_float;
    pub type GLclampf = super::__gl_imports::raw::c_float;
    pub type GLdouble = super::__gl_imports::raw::c_double;
    pub type GLclampd = super::__gl_imports::raw::c_double;
    pub type GLeglImageOES = *const super::__gl_imports::raw::c_void;
    pub type GLchar = super::__gl_imports::raw::c_char;
    pub type GLcharARB = super::__gl_imports::raw::c_char;

    #[cfg(target_os = "macos")]
    pub type GLhandleARB = *const super::__gl_imports::raw::c_void;
    #[cfg(not(target_os = "macos"))]
    pub type GLhandleARB = super::__gl_imports::raw::c_uint;

    pub type GLhalfARB = super::__gl_imports::raw::c_ushort;
    pub type GLhalf = super::__gl_imports::raw::c_ushort;

    // Must be 32 bits
    pub type GLfixed = GLint;

    pub type GLintptr = isize;
    pub type GLsizeiptr = isize;
    pub type GLint64 = i64;
    pub type GLuint64 = u64;
    pub type GLintptrARB = isize;
    pub type GLsizeiptrARB = isize;
    pub type GLint64EXT = i64;
    pub type GLuint64EXT = u64;

    pub enum __GLsync {}
    pub type GLsync = *const __GLsync;

    // compatible with OpenCL cl_context
    pub enum _cl_context {}
    pub enum _cl_event {}

    pub type GLDEBUGPROC = Option<
        extern "system" fn(
            source: GLenum,
            gltype: GLenum,
            id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            userParam: *mut super::__gl_imports::raw::c_void,
        ),
    >;
    pub type GLDEBUGPROCARB = Option<
        extern "system" fn(
            source: GLenum,
            gltype: GLenum,
            id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            userParam: *mut super::__gl_imports::raw::c_void,
        ),
    >;
    pub type GLDEBUGPROCKHR = Option<
        extern "system" fn(
            source: GLenum,
            gltype: GLenum,
            id: GLuint,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            userParam: *mut super::__gl_imports::raw::c_void,
        ),
    >;

    // GLES 1 types
    // "pub type GLclampx = i32;",

    // GLES 1/2 types (tagged for GLES 1)
    // "pub type GLbyte = i8;",
    // "pub type GLubyte = u8;",
    // "pub type GLfloat = GLfloat;",
    // "pub type GLclampf = GLfloat;",
    // "pub type GLfixed = i32;",
    // "pub type GLint64 = i64;",
    // "pub type GLuint64 = u64;",
    // "pub type GLintptr = intptr_t;",
    // "pub type GLsizeiptr = ssize_t;",

    // GLES 1/2 types (tagged for GLES 2 - attribute syntax is limited)
    // "pub type GLbyte = i8;",
    // "pub type GLubyte = u8;",
    // "pub type GLfloat = GLfloat;",
    // "pub type GLclampf = GLfloat;",
    // "pub type GLfixed = i32;",
    // "pub type GLint64 = i64;",
    // "pub type GLuint64 = u64;",
    // "pub type GLint64EXT = i64;",
    // "pub type GLuint64EXT = u64;",
    // "pub type GLintptr = intptr_t;",
    // "pub type GLsizeiptr = ssize_t;",

    // GLES 2 types (none currently)

    // Vendor extension types
    pub type GLDEBUGPROCAMD = Option<
        extern "system" fn(
            id: GLuint,
            category: GLenum,
            severity: GLenum,
            length: GLsizei,
            message: *const GLchar,
            userParam: *mut super::__gl_imports::raw::c_void,
        ),
    >;
    pub type GLhalfNV = super::__gl_imports::raw::c_ushort;
    pub type GLvdpauSurfaceNV = GLintptr;
}

#[allow(dead_code, non_upper_case_globals)]
pub const ACTIVE_ATTRIBUTES: types::GLenum = 0x8B89;
#[allow(dead_code, non_upper_case_globals)]
pub const ACTIVE_ATTRIBUTE_MAX_LENGTH: types::GLenum = 0x8B8A;
#[allow(dead_code, non_upper_case_globals)]
pub const ACTIVE_TEXTURE: types::GLenum = 0x84E0;
#[allow(dead_code, non_upper_case_globals)]
pub const ACTIVE_UNIFORMS: types::GLenum = 0x8B86;
#[allow(dead_code, non_upper_case_globals)]
pub const ACTIVE_UNIFORM_MAX_LENGTH: types::GLenum = 0x8B87;
#[allow(dead_code, non_upper_case_globals)]
pub const ALIASED_LINE_WIDTH_RANGE: types::GLenum = 0x846E;
#[allow(dead_code, non_upper_case_globals)]
pub const ALIASED_POINT_SIZE_RANGE: types::GLenum = 0x846D;
#[allow(dead_code, non_upper_case_globals)]
pub const ALPHA: types::GLenum = 0x1906;
#[allow(dead_code, non_upper_case_globals)]
pub const ALPHA_BITS: types::GLenum = 0x0D55;
#[allow(dead_code, non_upper_case_globals)]
pub const ALWAYS: types::GLenum = 0x0207;
#[allow(dead_code, non_upper_case_globals)]
pub const ARRAY_BUFFER: types::GLenum = 0x8892;
#[allow(dead_code, non_upper_case_globals)]
pub const ARRAY_BUFFER_BINDING: types::GLenum = 0x8894;
#[allow(dead_code, non_upper_case_globals)]
pub const ATTACHED_SHADERS: types::GLenum = 0x8B85;
#[allow(dead_code, non_upper_case_globals)]
pub const BACK: types::GLenum = 0x0405;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND: types::GLenum = 0x0BE2;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND_COLOR: types::GLenum = 0x8005;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND_DST_ALPHA: types::GLenum = 0x80CA;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND_DST_RGB: types::GLenum = 0x80C8;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND_EQUATION: types::GLenum = 0x8009;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND_EQUATION_ALPHA: types::GLenum = 0x883D;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND_EQUATION_RGB: types::GLenum = 0x8009;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND_SRC_ALPHA: types::GLenum = 0x80CB;
#[allow(dead_code, non_upper_case_globals)]
pub const BLEND_SRC_RGB: types::GLenum = 0x80C9;
#[allow(dead_code, non_upper_case_globals)]
pub const BLUE_BITS: types::GLenum = 0x0D54;
#[allow(dead_code, non_upper_case_globals)]
pub const BOOL: types::GLenum = 0x8B56;
#[allow(dead_code, non_upper_case_globals)]
pub const BOOL_VEC2: types::GLenum = 0x8B57;
#[allow(dead_code, non_upper_case_globals)]
pub const BOOL_VEC3: types::GLenum = 0x8B58;
#[allow(dead_code, non_upper_case_globals)]
pub const BOOL_VEC4: types::GLenum = 0x8B59;
#[allow(dead_code, non_upper_case_globals)]
pub const BUFFER_SIZE: types::GLenum = 0x8764;
#[allow(dead_code, non_upper_case_globals)]
pub const BUFFER_USAGE: types::GLenum = 0x8765;
#[allow(dead_code, non_upper_case_globals)]
pub const BYTE: types::GLenum = 0x1400;
#[allow(dead_code, non_upper_case_globals)]
pub const CCW: types::GLenum = 0x0901;
#[allow(dead_code, non_upper_case_globals)]
pub const CLAMP_TO_EDGE: types::GLenum = 0x812F;
#[allow(dead_code, non_upper_case_globals)]
pub const COLOR_ATTACHMENT0: types::GLenum = 0x8CE0;
#[allow(dead_code, non_upper_case_globals)]
pub const COLOR_BUFFER_BIT: types::GLenum = 0x00004000;
#[allow(dead_code, non_upper_case_globals)]
pub const COLOR_CLEAR_VALUE: types::GLenum = 0x0C22;
#[allow(dead_code, non_upper_case_globals)]
pub const COLOR_WRITEMASK: types::GLenum = 0x0C23;
#[allow(dead_code, non_upper_case_globals)]
pub const COMPILE_STATUS: types::GLenum = 0x8B81;
#[allow(dead_code, non_upper_case_globals)]
pub const COMPRESSED_TEXTURE_FORMATS: types::GLenum = 0x86A3;
#[allow(dead_code, non_upper_case_globals)]
pub const CONSTANT_ALPHA: types::GLenum = 0x8003;
#[allow(dead_code, non_upper_case_globals)]
pub const CONSTANT_COLOR: types::GLenum = 0x8001;
#[allow(dead_code, non_upper_case_globals)]
pub const CULL_FACE: types::GLenum = 0x0B44;
#[allow(dead_code, non_upper_case_globals)]
pub const CULL_FACE_MODE: types::GLenum = 0x0B45;
#[allow(dead_code, non_upper_case_globals)]
pub const CURRENT_PROGRAM: types::GLenum = 0x8B8D;
#[allow(dead_code, non_upper_case_globals)]
pub const CURRENT_VERTEX_ATTRIB: types::GLenum = 0x8626;
#[allow(dead_code, non_upper_case_globals)]
pub const CW: types::GLenum = 0x0900;
#[allow(dead_code, non_upper_case_globals)]
pub const DECR: types::GLenum = 0x1E03;
#[allow(dead_code, non_upper_case_globals)]
pub const DECR_WRAP: types::GLenum = 0x8508;
#[allow(dead_code, non_upper_case_globals)]
pub const DELETE_STATUS: types::GLenum = 0x8B80;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_ATTACHMENT: types::GLenum = 0x8D00;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_BITS: types::GLenum = 0x0D56;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_BUFFER_BIT: types::GLenum = 0x00000100;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_CLEAR_VALUE: types::GLenum = 0x0B73;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_COMPONENT: types::GLenum = 0x1902;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_COMPONENT16: types::GLenum = 0x81A5;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_FUNC: types::GLenum = 0x0B74;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_RANGE: types::GLenum = 0x0B70;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_TEST: types::GLenum = 0x0B71;
#[allow(dead_code, non_upper_case_globals)]
pub const DEPTH_WRITEMASK: types::GLenum = 0x0B72;
#[allow(dead_code, non_upper_case_globals)]
pub const DITHER: types::GLenum = 0x0BD0;
#[allow(dead_code, non_upper_case_globals)]
pub const DONT_CARE: types::GLenum = 0x1100;
#[allow(dead_code, non_upper_case_globals)]
pub const DST_ALPHA: types::GLenum = 0x0304;
#[allow(dead_code, non_upper_case_globals)]
pub const DST_COLOR: types::GLenum = 0x0306;
#[allow(dead_code, non_upper_case_globals)]
pub const DYNAMIC_DRAW: types::GLenum = 0x88E8;
#[allow(dead_code, non_upper_case_globals)]
pub const ELEMENT_ARRAY_BUFFER: types::GLenum = 0x8893;
#[allow(dead_code, non_upper_case_globals)]
pub const ELEMENT_ARRAY_BUFFER_BINDING: types::GLenum = 0x8895;
#[allow(dead_code, non_upper_case_globals)]
pub const EQUAL: types::GLenum = 0x0202;
#[allow(dead_code, non_upper_case_globals)]
pub const EXTENSIONS: types::GLenum = 0x1F03;
#[allow(dead_code, non_upper_case_globals)]
pub const FALSE: types::GLboolean = 0;
#[allow(dead_code, non_upper_case_globals)]
pub const FASTEST: types::GLenum = 0x1101;
#[allow(dead_code, non_upper_case_globals)]
pub const FIXED: types::GLenum = 0x140C;
#[allow(dead_code, non_upper_case_globals)]
pub const FLOAT: types::GLenum = 0x1406;
#[allow(dead_code, non_upper_case_globals)]
pub const FLOAT_MAT2: types::GLenum = 0x8B5A;
#[allow(dead_code, non_upper_case_globals)]
pub const FLOAT_MAT3: types::GLenum = 0x8B5B;
#[allow(dead_code, non_upper_case_globals)]
pub const FLOAT_MAT4: types::GLenum = 0x8B5C;
#[allow(dead_code, non_upper_case_globals)]
pub const FLOAT_VEC2: types::GLenum = 0x8B50;
#[allow(dead_code, non_upper_case_globals)]
pub const FLOAT_VEC3: types::GLenum = 0x8B51;
#[allow(dead_code, non_upper_case_globals)]
pub const FLOAT_VEC4: types::GLenum = 0x8B52;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAGMENT_SHADER: types::GLenum = 0x8B30;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER: types::GLenum = 0x8D40;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_ATTACHMENT_OBJECT_NAME: types::GLenum = 0x8CD1;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE: types::GLenum = 0x8CD0;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE: types::GLenum = 0x8CD3;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL: types::GLenum = 0x8CD2;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_BINDING: types::GLenum = 0x8CA6;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_COMPLETE: types::GLenum = 0x8CD5;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_INCOMPLETE_ATTACHMENT: types::GLenum = 0x8CD6;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_INCOMPLETE_DIMENSIONS: types::GLenum = 0x8CD9;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: types::GLenum = 0x8CD7;
#[allow(dead_code, non_upper_case_globals)]
pub const FRAMEBUFFER_UNSUPPORTED: types::GLenum = 0x8CDD;
#[allow(dead_code, non_upper_case_globals)]
pub const FRONT: types::GLenum = 0x0404;
#[allow(dead_code, non_upper_case_globals)]
pub const FRONT_AND_BACK: types::GLenum = 0x0408;
#[allow(dead_code, non_upper_case_globals)]
pub const FRONT_FACE: types::GLenum = 0x0B46;
#[allow(dead_code, non_upper_case_globals)]
pub const FUNC_ADD: types::GLenum = 0x8006;
#[allow(dead_code, non_upper_case_globals)]
pub const FUNC_REVERSE_SUBTRACT: types::GLenum = 0x800B;
#[allow(dead_code, non_upper_case_globals)]
pub const FUNC_SUBTRACT: types::GLenum = 0x800A;
#[allow(dead_code, non_upper_case_globals)]
pub const GENERATE_MIPMAP_HINT: types::GLenum = 0x8192;
#[allow(dead_code, non_upper_case_globals)]
pub const GEQUAL: types::GLenum = 0x0206;
#[allow(dead_code, non_upper_case_globals)]
pub const GREATER: types::GLenum = 0x0204;
#[allow(dead_code, non_upper_case_globals)]
pub const GREEN_BITS: types::GLenum = 0x0D53;
#[allow(dead_code, non_upper_case_globals)]
pub const HIGH_FLOAT: types::GLenum = 0x8DF2;
#[allow(dead_code, non_upper_case_globals)]
pub const HIGH_INT: types::GLenum = 0x8DF5;
#[allow(dead_code, non_upper_case_globals)]
pub const IMPLEMENTATION_COLOR_READ_FORMAT: types::GLenum = 0x8B9B;
#[allow(dead_code, non_upper_case_globals)]
pub const IMPLEMENTATION_COLOR_READ_TYPE: types::GLenum = 0x8B9A;
#[allow(dead_code, non_upper_case_globals)]
pub const INCR: types::GLenum = 0x1E02;
#[allow(dead_code, non_upper_case_globals)]
pub const INCR_WRAP: types::GLenum = 0x8507;
#[allow(dead_code, non_upper_case_globals)]
pub const INFO_LOG_LENGTH: types::GLenum = 0x8B84;
#[allow(dead_code, non_upper_case_globals)]
pub const INT: types::GLenum = 0x1404;
#[allow(dead_code, non_upper_case_globals)]
pub const INT_VEC2: types::GLenum = 0x8B53;
#[allow(dead_code, non_upper_case_globals)]
pub const INT_VEC3: types::GLenum = 0x8B54;
#[allow(dead_code, non_upper_case_globals)]
pub const INT_VEC4: types::GLenum = 0x8B55;
#[allow(dead_code, non_upper_case_globals)]
pub const INVALID_ENUM: types::GLenum = 0x0500;
#[allow(dead_code, non_upper_case_globals)]
pub const INVALID_FRAMEBUFFER_OPERATION: types::GLenum = 0x0506;
#[allow(dead_code, non_upper_case_globals)]
pub const INVALID_OPERATION: types::GLenum = 0x0502;
#[allow(dead_code, non_upper_case_globals)]
pub const INVALID_VALUE: types::GLenum = 0x0501;
#[allow(dead_code, non_upper_case_globals)]
pub const INVERT: types::GLenum = 0x150A;
#[allow(dead_code, non_upper_case_globals)]
pub const KEEP: types::GLenum = 0x1E00;
#[allow(dead_code, non_upper_case_globals)]
pub const LEQUAL: types::GLenum = 0x0203;
#[allow(dead_code, non_upper_case_globals)]
pub const LESS: types::GLenum = 0x0201;
#[allow(dead_code, non_upper_case_globals)]
pub const LINEAR: types::GLenum = 0x2601;
#[allow(dead_code, non_upper_case_globals)]
pub const LINEAR_MIPMAP_LINEAR: types::GLenum = 0x2703;
#[allow(dead_code, non_upper_case_globals)]
pub const LINEAR_MIPMAP_NEAREST: types::GLenum = 0x2701;
#[allow(dead_code, non_upper_case_globals)]
pub const LINES: types::GLenum = 0x0001;
#[allow(dead_code, non_upper_case_globals)]
pub const LINE_LOOP: types::GLenum = 0x0002;
#[allow(dead_code, non_upper_case_globals)]
pub const LINE_STRIP: types::GLenum = 0x0003;
#[allow(dead_code, non_upper_case_globals)]
pub const LINE_WIDTH: types::GLenum = 0x0B21;
#[allow(dead_code, non_upper_case_globals)]
pub const LINK_STATUS: types::GLenum = 0x8B82;
#[allow(dead_code, non_upper_case_globals)]
pub const LOW_FLOAT: types::GLenum = 0x8DF0;
#[allow(dead_code, non_upper_case_globals)]
pub const LOW_INT: types::GLenum = 0x8DF3;
#[allow(dead_code, non_upper_case_globals)]
pub const LUMINANCE: types::GLenum = 0x1909;
#[allow(dead_code, non_upper_case_globals)]
pub const LUMINANCE_ALPHA: types::GLenum = 0x190A;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_COMBINED_TEXTURE_IMAGE_UNITS: types::GLenum = 0x8B4D;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_CUBE_MAP_TEXTURE_SIZE: types::GLenum = 0x851C;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_FRAGMENT_UNIFORM_VECTORS: types::GLenum = 0x8DFD;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_RENDERBUFFER_SIZE: types::GLenum = 0x84E8;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_TEXTURE_IMAGE_UNITS: types::GLenum = 0x8872;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_TEXTURE_SIZE: types::GLenum = 0x0D33;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_VARYING_VECTORS: types::GLenum = 0x8DFC;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_VERTEX_ATTRIBS: types::GLenum = 0x8869;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_VERTEX_TEXTURE_IMAGE_UNITS: types::GLenum = 0x8B4C;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_VERTEX_UNIFORM_VECTORS: types::GLenum = 0x8DFB;
#[allow(dead_code, non_upper_case_globals)]
pub const MAX_VIEWPORT_DIMS: types::GLenum = 0x0D3A;
#[allow(dead_code, non_upper_case_globals)]
pub const MEDIUM_FLOAT: types::GLenum = 0x8DF1;
#[allow(dead_code, non_upper_case_globals)]
pub const MEDIUM_INT: types::GLenum = 0x8DF4;
#[allow(dead_code, non_upper_case_globals)]
pub const MIRRORED_REPEAT: types::GLenum = 0x8370;
#[allow(dead_code, non_upper_case_globals)]
pub const NEAREST: types::GLenum = 0x2600;
#[allow(dead_code, non_upper_case_globals)]
pub const NEAREST_MIPMAP_LINEAR: types::GLenum = 0x2702;
#[allow(dead_code, non_upper_case_globals)]
pub const NEAREST_MIPMAP_NEAREST: types::GLenum = 0x2700;
#[allow(dead_code, non_upper_case_globals)]
pub const NEVER: types::GLenum = 0x0200;
#[allow(dead_code, non_upper_case_globals)]
pub const NICEST: types::GLenum = 0x1102;
#[allow(dead_code, non_upper_case_globals)]
pub const NONE: types::GLenum = 0;
#[allow(dead_code, non_upper_case_globals)]
pub const NOTEQUAL: types::GLenum = 0x0205;
#[allow(dead_code, non_upper_case_globals)]
pub const NO_ERROR: types::GLenum = 0;
#[allow(dead_code, non_upper_case_globals)]
pub const NUM_COMPRESSED_TEXTURE_FORMATS: types::GLenum = 0x86A2;
#[allow(dead_code, non_upper_case_globals)]
pub const NUM_SHADER_BINARY_FORMATS: types::GLenum = 0x8DF9;
#[allow(dead_code, non_upper_case_globals)]
pub const ONE: types::GLenum = 1;
#[allow(dead_code, non_upper_case_globals)]
pub const ONE_MINUS_CONSTANT_ALPHA: types::GLenum = 0x8004;
#[allow(dead_code, non_upper_case_globals)]
pub const ONE_MINUS_CONSTANT_COLOR: types::GLenum = 0x8002;
#[allow(dead_code, non_upper_case_globals)]
pub const ONE_MINUS_DST_ALPHA: types::GLenum = 0x0305;
#[allow(dead_code, non_upper_case_globals)]
pub const ONE_MINUS_DST_COLOR: types::GLenum = 0x0307;
#[allow(dead_code, non_upper_case_globals)]
pub const ONE_MINUS_SRC_ALPHA: types::GLenum = 0x0303;
#[allow(dead_code, non_upper_case_globals)]
pub const ONE_MINUS_SRC_COLOR: types::GLenum = 0x0301;
#[allow(dead_code, non_upper_case_globals)]
pub const OUT_OF_MEMORY: types::GLenum = 0x0505;
#[allow(dead_code, non_upper_case_globals)]
pub const PACK_ALIGNMENT: types::GLenum = 0x0D05;
#[allow(dead_code, non_upper_case_globals)]
pub const POINTS: types::GLenum = 0x0000;
#[allow(dead_code, non_upper_case_globals)]
pub const POLYGON_OFFSET_FACTOR: types::GLenum = 0x8038;
#[allow(dead_code, non_upper_case_globals)]
pub const POLYGON_OFFSET_FILL: types::GLenum = 0x8037;
#[allow(dead_code, non_upper_case_globals)]
pub const POLYGON_OFFSET_UNITS: types::GLenum = 0x2A00;
#[allow(dead_code, non_upper_case_globals)]
pub const RED_BITS: types::GLenum = 0x0D52;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER: types::GLenum = 0x8D41;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_ALPHA_SIZE: types::GLenum = 0x8D53;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_BINDING: types::GLenum = 0x8CA7;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_BLUE_SIZE: types::GLenum = 0x8D52;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_DEPTH_SIZE: types::GLenum = 0x8D54;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_GREEN_SIZE: types::GLenum = 0x8D51;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_HEIGHT: types::GLenum = 0x8D43;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_INTERNAL_FORMAT: types::GLenum = 0x8D44;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_RED_SIZE: types::GLenum = 0x8D50;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_STENCIL_SIZE: types::GLenum = 0x8D55;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERBUFFER_WIDTH: types::GLenum = 0x8D42;
#[allow(dead_code, non_upper_case_globals)]
pub const RENDERER: types::GLenum = 0x1F01;
#[allow(dead_code, non_upper_case_globals)]
pub const REPEAT: types::GLenum = 0x2901;
#[allow(dead_code, non_upper_case_globals)]
pub const REPLACE: types::GLenum = 0x1E01;
#[allow(dead_code, non_upper_case_globals)]
pub const RGB: types::GLenum = 0x1907;
#[allow(dead_code, non_upper_case_globals)]
pub const RGB565: types::GLenum = 0x8D62;
#[allow(dead_code, non_upper_case_globals)]
pub const RGB5_A1: types::GLenum = 0x8057;
#[allow(dead_code, non_upper_case_globals)]
pub const RGBA: types::GLenum = 0x1908;
#[allow(dead_code, non_upper_case_globals)]
pub const RGBA4: types::GLenum = 0x8056;
#[allow(dead_code, non_upper_case_globals)]
pub const SAMPLER_2D: types::GLenum = 0x8B5E;
#[allow(dead_code, non_upper_case_globals)]
pub const SAMPLER_CUBE: types::GLenum = 0x8B60;
#[allow(dead_code, non_upper_case_globals)]
pub const SAMPLES: types::GLenum = 0x80A9;
#[allow(dead_code, non_upper_case_globals)]
pub const SAMPLE_ALPHA_TO_COVERAGE: types::GLenum = 0x809E;
#[allow(dead_code, non_upper_case_globals)]
pub const SAMPLE_BUFFERS: types::GLenum = 0x80A8;
#[allow(dead_code, non_upper_case_globals)]
pub const SAMPLE_COVERAGE: types::GLenum = 0x80A0;
#[allow(dead_code, non_upper_case_globals)]
pub const SAMPLE_COVERAGE_INVERT: types::GLenum = 0x80AB;
#[allow(dead_code, non_upper_case_globals)]
pub const SAMPLE_COVERAGE_VALUE: types::GLenum = 0x80AA;
#[allow(dead_code, non_upper_case_globals)]
pub const SCISSOR_BOX: types::GLenum = 0x0C10;
#[allow(dead_code, non_upper_case_globals)]
pub const SCISSOR_TEST: types::GLenum = 0x0C11;
#[allow(dead_code, non_upper_case_globals)]
pub const SHADER_BINARY_FORMATS: types::GLenum = 0x8DF8;
#[allow(dead_code, non_upper_case_globals)]
pub const SHADER_COMPILER: types::GLenum = 0x8DFA;
#[allow(dead_code, non_upper_case_globals)]
pub const SHADER_SOURCE_LENGTH: types::GLenum = 0x8B88;
#[allow(dead_code, non_upper_case_globals)]
pub const SHADER_TYPE: types::GLenum = 0x8B4F;
#[allow(dead_code, non_upper_case_globals)]
pub const SHADING_LANGUAGE_VERSION: types::GLenum = 0x8B8C;
#[allow(dead_code, non_upper_case_globals)]
pub const SHORT: types::GLenum = 0x1402;
#[allow(dead_code, non_upper_case_globals)]
pub const SRC_ALPHA: types::GLenum = 0x0302;
#[allow(dead_code, non_upper_case_globals)]
pub const SRC_ALPHA_SATURATE: types::GLenum = 0x0308;
#[allow(dead_code, non_upper_case_globals)]
pub const SRC_COLOR: types::GLenum = 0x0300;
#[allow(dead_code, non_upper_case_globals)]
pub const STATIC_DRAW: types::GLenum = 0x88E4;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_ATTACHMENT: types::GLenum = 0x8D20;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BACK_FAIL: types::GLenum = 0x8801;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BACK_FUNC: types::GLenum = 0x8800;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BACK_PASS_DEPTH_FAIL: types::GLenum = 0x8802;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BACK_PASS_DEPTH_PASS: types::GLenum = 0x8803;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BACK_REF: types::GLenum = 0x8CA3;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BACK_VALUE_MASK: types::GLenum = 0x8CA4;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BACK_WRITEMASK: types::GLenum = 0x8CA5;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BITS: types::GLenum = 0x0D57;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_BUFFER_BIT: types::GLenum = 0x00000400;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_CLEAR_VALUE: types::GLenum = 0x0B91;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_FAIL: types::GLenum = 0x0B94;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_FUNC: types::GLenum = 0x0B92;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_INDEX8: types::GLenum = 0x8D48;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_PASS_DEPTH_FAIL: types::GLenum = 0x0B95;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_PASS_DEPTH_PASS: types::GLenum = 0x0B96;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_REF: types::GLenum = 0x0B97;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_TEST: types::GLenum = 0x0B90;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_VALUE_MASK: types::GLenum = 0x0B93;
#[allow(dead_code, non_upper_case_globals)]
pub const STENCIL_WRITEMASK: types::GLenum = 0x0B98;
#[allow(dead_code, non_upper_case_globals)]
pub const STREAM_DRAW: types::GLenum = 0x88E0;
#[allow(dead_code, non_upper_case_globals)]
pub const SUBPIXEL_BITS: types::GLenum = 0x0D50;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE: types::GLenum = 0x1702;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE0: types::GLenum = 0x84C0;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE1: types::GLenum = 0x84C1;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE10: types::GLenum = 0x84CA;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE11: types::GLenum = 0x84CB;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE12: types::GLenum = 0x84CC;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE13: types::GLenum = 0x84CD;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE14: types::GLenum = 0x84CE;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE15: types::GLenum = 0x84CF;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE16: types::GLenum = 0x84D0;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE17: types::GLenum = 0x84D1;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE18: types::GLenum = 0x84D2;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE19: types::GLenum = 0x84D3;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE2: types::GLenum = 0x84C2;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE20: types::GLenum = 0x84D4;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE21: types::GLenum = 0x84D5;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE22: types::GLenum = 0x84D6;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE23: types::GLenum = 0x84D7;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE24: types::GLenum = 0x84D8;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE25: types::GLenum = 0x84D9;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE26: types::GLenum = 0x84DA;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE27: types::GLenum = 0x84DB;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE28: types::GLenum = 0x84DC;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE29: types::GLenum = 0x84DD;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE3: types::GLenum = 0x84C3;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE30: types::GLenum = 0x84DE;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE31: types::GLenum = 0x84DF;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE4: types::GLenum = 0x84C4;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE5: types::GLenum = 0x84C5;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE6: types::GLenum = 0x84C6;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE7: types::GLenum = 0x84C7;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE8: types::GLenum = 0x84C8;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE9: types::GLenum = 0x84C9;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_2D: types::GLenum = 0x0DE1;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_BINDING_2D: types::GLenum = 0x8069;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_BINDING_CUBE_MAP: types::GLenum = 0x8514;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_CUBE_MAP: types::GLenum = 0x8513;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_CUBE_MAP_NEGATIVE_X: types::GLenum = 0x8516;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_CUBE_MAP_NEGATIVE_Y: types::GLenum = 0x8518;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_CUBE_MAP_NEGATIVE_Z: types::GLenum = 0x851A;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_CUBE_MAP_POSITIVE_X: types::GLenum = 0x8515;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_CUBE_MAP_POSITIVE_Y: types::GLenum = 0x8517;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_CUBE_MAP_POSITIVE_Z: types::GLenum = 0x8519;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_MAG_FILTER: types::GLenum = 0x2800;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_MIN_FILTER: types::GLenum = 0x2801;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_WRAP_S: types::GLenum = 0x2802;
#[allow(dead_code, non_upper_case_globals)]
pub const TEXTURE_WRAP_T: types::GLenum = 0x2803;
#[allow(dead_code, non_upper_case_globals)]
pub const TRIANGLES: types::GLenum = 0x0004;
#[allow(dead_code, non_upper_case_globals)]
pub const TRIANGLE_FAN: types::GLenum = 0x0006;
#[allow(dead_code, non_upper_case_globals)]
pub const TRIANGLE_STRIP: types::GLenum = 0x0005;
#[allow(dead_code, non_upper_case_globals)]
pub const TRUE: types::GLboolean = 1;
#[allow(dead_code, non_upper_case_globals)]
pub const UNPACK_ALIGNMENT: types::GLenum = 0x0CF5;
#[allow(dead_code, non_upper_case_globals)]
pub const UNSIGNED_BYTE: types::GLenum = 0x1401;
#[allow(dead_code, non_upper_case_globals)]
pub const UNSIGNED_INT: types::GLenum = 0x1405;
#[allow(dead_code, non_upper_case_globals)]
pub const UNSIGNED_SHORT: types::GLenum = 0x1403;
#[allow(dead_code, non_upper_case_globals)]
pub const UNSIGNED_SHORT_4_4_4_4: types::GLenum = 0x8033;
#[allow(dead_code, non_upper_case_globals)]
pub const UNSIGNED_SHORT_5_5_5_1: types::GLenum = 0x8034;
#[allow(dead_code, non_upper_case_globals)]
pub const UNSIGNED_SHORT_5_6_5: types::GLenum = 0x8363;
#[allow(dead_code, non_upper_case_globals)]
pub const VALIDATE_STATUS: types::GLenum = 0x8B83;
#[allow(dead_code, non_upper_case_globals)]
pub const VENDOR: types::GLenum = 0x1F00;
#[allow(dead_code, non_upper_case_globals)]
pub const VERSION: types::GLenum = 0x1F02;
#[allow(dead_code, non_upper_case_globals)]
pub const VERTEX_ATTRIB_ARRAY_BUFFER_BINDING: types::GLenum = 0x889F;
#[allow(dead_code, non_upper_case_globals)]
pub const VERTEX_ATTRIB_ARRAY_ENABLED: types::GLenum = 0x8622;
#[allow(dead_code, non_upper_case_globals)]
pub const VERTEX_ATTRIB_ARRAY_NORMALIZED: types::GLenum = 0x886A;
#[allow(dead_code, non_upper_case_globals)]
pub const VERTEX_ATTRIB_ARRAY_POINTER: types::GLenum = 0x8645;
#[allow(dead_code, non_upper_case_globals)]
pub const VERTEX_ATTRIB_ARRAY_SIZE: types::GLenum = 0x8623;
#[allow(dead_code, non_upper_case_globals)]
pub const VERTEX_ATTRIB_ARRAY_STRIDE: types::GLenum = 0x8624;
#[allow(dead_code, non_upper_case_globals)]
pub const VERTEX_ATTRIB_ARRAY_TYPE: types::GLenum = 0x8625;
#[allow(dead_code, non_upper_case_globals)]
pub const VERTEX_SHADER: types::GLenum = 0x8B31;
#[allow(dead_code, non_upper_case_globals)]
pub const VIEWPORT: types::GLenum = 0x0BA2;
#[allow(dead_code, non_upper_case_globals)]
pub const ZERO: types::GLenum = 0;
/// Fallbacks: ActiveTextureARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ActiveTexture(texture: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> ()>(
        storage::ActiveTexture.f,
    )(texture)
}
/// Fallbacks: AttachObjectARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn AttachShader(program: types::GLuint, shader: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint, types::GLuint) -> ()>(
        storage::AttachShader.f,
    )(program, shader)
}
/// Fallbacks: BindAttribLocationARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BindAttribLocation(
    program: types::GLuint,
    index: types::GLuint,
    name: *const types::GLchar,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLuint, *const types::GLchar) -> (),
    >(storage::BindAttribLocation.f)(program, index, name)
}
/// Fallbacks: BindBufferARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BindBuffer(target: types::GLenum, buffer: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLuint) -> ()>(
        storage::BindBuffer.f,
    )(target, buffer)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BindFramebuffer(target: types::GLenum, framebuffer: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLuint) -> ()>(
        storage::BindFramebuffer.f,
    )(target, framebuffer)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BindRenderbuffer(target: types::GLenum, renderbuffer: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLuint) -> ()>(
        storage::BindRenderbuffer.f,
    )(target, renderbuffer)
}
/// Fallbacks: BindTextureEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BindTexture(target: types::GLenum, texture: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLuint) -> ()>(
        storage::BindTexture.f,
    )(target, texture)
}
/// Fallbacks: BlendColorEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BlendColor(
    red: types::GLfloat,
    green: types::GLfloat,
    blue: types::GLfloat,
    alpha: types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLfloat, types::GLfloat, types::GLfloat, types::GLfloat) -> (),
    >(storage::BlendColor.f)(red, green, blue, alpha)
}
/// Fallbacks: BlendEquationEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BlendEquation(mode: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> ()>(
        storage::BlendEquation.f,
    )(mode)
}
/// Fallbacks: BlendEquationSeparateEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BlendEquationSeparate(modeRGB: types::GLenum, modeAlpha: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLenum) -> ()>(
        storage::BlendEquationSeparate.f,
    )(modeRGB, modeAlpha)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BlendFunc(sfactor: types::GLenum, dfactor: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLenum) -> ()>(
        storage::BlendFunc.f,
    )(sfactor, dfactor)
}
/// Fallbacks: BlendFuncSeparateEXT, BlendFuncSeparateINGR
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BlendFuncSeparate(
    sfactorRGB: types::GLenum,
    dfactorRGB: types::GLenum,
    sfactorAlpha: types::GLenum,
    dfactorAlpha: types::GLenum,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLenum, types::GLenum) -> (),
    >(storage::BlendFuncSeparate.f)(sfactorRGB, dfactorRGB, sfactorAlpha, dfactorAlpha)
}
/// Fallbacks: BufferDataARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BufferData(
    target: types::GLenum,
    size: types::GLsizeiptr,
    data: *const __gl_imports::raw::c_void,
    usage: types::GLenum,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLsizeiptr,
            *const __gl_imports::raw::c_void,
            types::GLenum,
        ) -> (),
    >(storage::BufferData.f)(target, size, data, usage)
}
/// Fallbacks: BufferSubDataARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn BufferSubData(
    target: types::GLenum,
    offset: types::GLintptr,
    size: types::GLsizeiptr,
    data: *const __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLintptr,
            types::GLsizeiptr,
            *const __gl_imports::raw::c_void,
        ) -> (),
    >(storage::BufferSubData.f)(target, offset, size, data)
}
/// Fallbacks: CheckFramebufferStatusEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CheckFramebufferStatus(target: types::GLenum) -> types::GLenum {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> types::GLenum>(
        storage::CheckFramebufferStatus.f,
    )(target)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Clear(mask: types::GLbitfield) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLbitfield) -> ()>(storage::Clear.f)(
        mask,
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ClearColor(
    red: types::GLfloat,
    green: types::GLfloat,
    blue: types::GLfloat,
    alpha: types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLfloat, types::GLfloat, types::GLfloat, types::GLfloat) -> (),
    >(storage::ClearColor.f)(red, green, blue, alpha)
}
/// Fallbacks: ClearDepthfOES
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ClearDepthf(d: types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLfloat) -> ()>(
        storage::ClearDepthf.f,
    )(d)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ClearStencil(s: types::GLint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLint) -> ()>(
        storage::ClearStencil.f,
    )(s)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ColorMask(
    red: types::GLboolean,
    green: types::GLboolean,
    blue: types::GLboolean,
    alpha: types::GLboolean,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLboolean,
            types::GLboolean,
            types::GLboolean,
            types::GLboolean,
        ) -> (),
    >(storage::ColorMask.f)(red, green, blue, alpha)
}
/// Fallbacks: CompileShaderARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CompileShader(shader: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::CompileShader.f,
    )(shader)
}
/// Fallbacks: CompressedTexImage2DARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CompressedTexImage2D(
    target: types::GLenum,
    level: types::GLint,
    internalformat: types::GLenum,
    width: types::GLsizei,
    height: types::GLsizei,
    border: types::GLint,
    imageSize: types::GLsizei,
    data: *const __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLint,
            types::GLenum,
            types::GLsizei,
            types::GLsizei,
            types::GLint,
            types::GLsizei,
            *const __gl_imports::raw::c_void,
        ) -> (),
    >(storage::CompressedTexImage2D.f)(
        target,
        level,
        internalformat,
        width,
        height,
        border,
        imageSize,
        data,
    )
}
/// Fallbacks: CompressedTexSubImage2DARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CompressedTexSubImage2D(
    target: types::GLenum,
    level: types::GLint,
    xoffset: types::GLint,
    yoffset: types::GLint,
    width: types::GLsizei,
    height: types::GLsizei,
    format: types::GLenum,
    imageSize: types::GLsizei,
    data: *const __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLint,
            types::GLint,
            types::GLint,
            types::GLsizei,
            types::GLsizei,
            types::GLenum,
            types::GLsizei,
            *const __gl_imports::raw::c_void,
        ) -> (),
    >(storage::CompressedTexSubImage2D.f)(
        target, level, xoffset, yoffset, width, height, format, imageSize, data,
    )
}
/// Fallbacks: CopyTexImage2DEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CopyTexImage2D(
    target: types::GLenum,
    level: types::GLint,
    internalformat: types::GLenum,
    x: types::GLint,
    y: types::GLint,
    width: types::GLsizei,
    height: types::GLsizei,
    border: types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLint,
            types::GLenum,
            types::GLint,
            types::GLint,
            types::GLsizei,
            types::GLsizei,
            types::GLint,
        ) -> (),
    >(storage::CopyTexImage2D.f)(target, level, internalformat, x, y, width, height, border)
}
/// Fallbacks: CopyTexSubImage2DEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CopyTexSubImage2D(
    target: types::GLenum,
    level: types::GLint,
    xoffset: types::GLint,
    yoffset: types::GLint,
    x: types::GLint,
    y: types::GLint,
    width: types::GLsizei,
    height: types::GLsizei,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLint,
            types::GLint,
            types::GLint,
            types::GLint,
            types::GLint,
            types::GLsizei,
            types::GLsizei,
        ) -> (),
    >(storage::CopyTexSubImage2D.f)(target, level, xoffset, yoffset, x, y, width, height)
}
/// Fallbacks: CreateProgramObjectARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CreateProgram() -> types::GLuint {
    __gl_imports::mem::transmute::<_, extern "system" fn() -> types::GLuint>(
        storage::CreateProgram.f,
    )()
}
/// Fallbacks: CreateShaderObjectARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CreateShader(type_: types::GLenum) -> types::GLuint {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> types::GLuint>(
        storage::CreateShader.f,
    )(type_)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn CullFace(mode: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> ()>(storage::CullFace.f)(
        mode,
    )
}
/// Fallbacks: DeleteBuffersARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DeleteBuffers(n: types::GLsizei, buffers: *const types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLsizei, *const types::GLuint) -> ()>(
        storage::DeleteBuffers.f,
    )(n, buffers)
}
/// Fallbacks: DeleteFramebuffersEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DeleteFramebuffers(n: types::GLsizei, framebuffers: *const types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLsizei, *const types::GLuint) -> ()>(
        storage::DeleteFramebuffers.f,
    )(n, framebuffers)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DeleteProgram(program: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::DeleteProgram.f,
    )(program)
}
/// Fallbacks: DeleteRenderbuffersEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DeleteRenderbuffers(n: types::GLsizei, renderbuffers: *const types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLsizei, *const types::GLuint) -> ()>(
        storage::DeleteRenderbuffers.f,
    )(n, renderbuffers)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DeleteShader(shader: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::DeleteShader.f,
    )(shader)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DeleteTextures(n: types::GLsizei, textures: *const types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLsizei, *const types::GLuint) -> ()>(
        storage::DeleteTextures.f,
    )(n, textures)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DepthFunc(func: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> ()>(storage::DepthFunc.f)(
        func,
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DepthMask(flag: types::GLboolean) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLboolean) -> ()>(
        storage::DepthMask.f,
    )(flag)
}
/// Fallbacks: DepthRangefOES
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DepthRangef(n: types::GLfloat, f: types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLfloat, types::GLfloat) -> ()>(
        storage::DepthRangef.f,
    )(n, f)
}
/// Fallbacks: DetachObjectARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DetachShader(program: types::GLuint, shader: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint, types::GLuint) -> ()>(
        storage::DetachShader.f,
    )(program, shader)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Disable(cap: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> ()>(storage::Disable.f)(
        cap,
    )
}
/// Fallbacks: DisableVertexAttribArrayARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DisableVertexAttribArray(index: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::DisableVertexAttribArray.f,
    )(index)
}
/// Fallbacks: DrawArraysEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DrawArrays(mode: types::GLenum, first: types::GLint, count: types::GLsizei) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLint, types::GLsizei) -> (),
    >(storage::DrawArrays.f)(mode, first, count)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn DrawElements(
    mode: types::GLenum,
    count: types::GLsizei,
    type_: types::GLenum,
    indices: *const __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLsizei,
            types::GLenum,
            *const __gl_imports::raw::c_void,
        ) -> (),
    >(storage::DrawElements.f)(mode, count, type_, indices)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Enable(cap: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> ()>(storage::Enable.f)(
        cap,
    )
}
/// Fallbacks: EnableVertexAttribArrayARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn EnableVertexAttribArray(index: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::EnableVertexAttribArray.f,
    )(index)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Finish() -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn() -> ()>(storage::Finish.f)()
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Flush() -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn() -> ()>(storage::Flush.f)()
}
/// Fallbacks: FramebufferRenderbufferEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn FramebufferRenderbuffer(
    target: types::GLenum,
    attachment: types::GLenum,
    renderbuffertarget: types::GLenum,
    renderbuffer: types::GLuint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLenum, types::GLuint) -> (),
    >(storage::FramebufferRenderbuffer.f)(target, attachment, renderbuffertarget, renderbuffer)
}
/// Fallbacks: FramebufferTexture2DEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn FramebufferTexture2D(
    target: types::GLenum,
    attachment: types::GLenum,
    textarget: types::GLenum,
    texture: types::GLuint,
    level: types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLenum,
            types::GLenum,
            types::GLuint,
            types::GLint,
        ) -> (),
    >(storage::FramebufferTexture2D.f)(target, attachment, textarget, texture, level)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn FrontFace(mode: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> ()>(storage::FrontFace.f)(
        mode,
    )
}
/// Fallbacks: GenBuffersARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GenBuffers(n: types::GLsizei, buffers: *mut types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLsizei, *mut types::GLuint) -> ()>(
        storage::GenBuffers.f,
    )(n, buffers)
}
/// Fallbacks: GenFramebuffersEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GenFramebuffers(n: types::GLsizei, framebuffers: *mut types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLsizei, *mut types::GLuint) -> ()>(
        storage::GenFramebuffers.f,
    )(n, framebuffers)
}
/// Fallbacks: GenRenderbuffersEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GenRenderbuffers(n: types::GLsizei, renderbuffers: *mut types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLsizei, *mut types::GLuint) -> ()>(
        storage::GenRenderbuffers.f,
    )(n, renderbuffers)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GenTextures(n: types::GLsizei, textures: *mut types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLsizei, *mut types::GLuint) -> ()>(
        storage::GenTextures.f,
    )(n, textures)
}
/// Fallbacks: GenerateMipmapEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GenerateMipmap(target: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> ()>(
        storage::GenerateMipmap.f,
    )(target)
}
/// Fallbacks: GetActiveAttribARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetActiveAttrib(
    program: types::GLuint,
    index: types::GLuint,
    bufSize: types::GLsizei,
    length: *mut types::GLsizei,
    size: *mut types::GLint,
    type_: *mut types::GLenum,
    name: *mut types::GLchar,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLuint,
            types::GLsizei,
            *mut types::GLsizei,
            *mut types::GLint,
            *mut types::GLenum,
            *mut types::GLchar,
        ) -> (),
    >(storage::GetActiveAttrib.f)(program, index, bufSize, length, size, type_, name)
}
/// Fallbacks: GetActiveUniformARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetActiveUniform(
    program: types::GLuint,
    index: types::GLuint,
    bufSize: types::GLsizei,
    length: *mut types::GLsizei,
    size: *mut types::GLint,
    type_: *mut types::GLenum,
    name: *mut types::GLchar,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLuint,
            types::GLsizei,
            *mut types::GLsizei,
            *mut types::GLint,
            *mut types::GLenum,
            *mut types::GLchar,
        ) -> (),
    >(storage::GetActiveUniform.f)(program, index, bufSize, length, size, type_, name)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetAttachedShaders(
    program: types::GLuint,
    maxCount: types::GLsizei,
    count: *mut types::GLsizei,
    shaders: *mut types::GLuint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLsizei,
            *mut types::GLsizei,
            *mut types::GLuint,
        ) -> (),
    >(storage::GetAttachedShaders.f)(program, maxCount, count, shaders)
}
/// Fallbacks: GetAttribLocationARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetAttribLocation(
    program: types::GLuint,
    name: *const types::GLchar,
) -> types::GLint {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, *const types::GLchar) -> types::GLint,
    >(storage::GetAttribLocation.f)(program, name)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetBooleanv(pname: types::GLenum, data: *mut types::GLboolean) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, *mut types::GLboolean) -> ()>(
        storage::GetBooleanv.f,
    )(pname, data)
}
/// Fallbacks: GetBufferParameterivARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetBufferParameteriv(
    target: types::GLenum,
    pname: types::GLenum,
    params: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, *mut types::GLint) -> (),
    >(storage::GetBufferParameteriv.f)(target, pname, params)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetError() -> types::GLenum {
    __gl_imports::mem::transmute::<_, extern "system" fn() -> types::GLenum>(storage::GetError.f)()
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetFloatv(pname: types::GLenum, data: *mut types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, *mut types::GLfloat) -> ()>(
        storage::GetFloatv.f,
    )(pname, data)
}
/// Fallbacks: GetFramebufferAttachmentParameterivEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetFramebufferAttachmentParameteriv(
    target: types::GLenum,
    attachment: types::GLenum,
    pname: types::GLenum,
    params: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLenum, *mut types::GLint) -> (),
    >(storage::GetFramebufferAttachmentParameteriv.f)(target, attachment, pname, params)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetIntegerv(pname: types::GLenum, data: *mut types::GLint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, *mut types::GLint) -> ()>(
        storage::GetIntegerv.f,
    )(pname, data)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetProgramInfoLog(
    program: types::GLuint,
    bufSize: types::GLsizei,
    length: *mut types::GLsizei,
    infoLog: *mut types::GLchar,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLsizei,
            *mut types::GLsizei,
            *mut types::GLchar,
        ) -> (),
    >(storage::GetProgramInfoLog.f)(program, bufSize, length, infoLog)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetProgramiv(
    program: types::GLuint,
    pname: types::GLenum,
    params: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLenum, *mut types::GLint) -> (),
    >(storage::GetProgramiv.f)(program, pname, params)
}
/// Fallbacks: GetRenderbufferParameterivEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetRenderbufferParameteriv(
    target: types::GLenum,
    pname: types::GLenum,
    params: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, *mut types::GLint) -> (),
    >(storage::GetRenderbufferParameteriv.f)(target, pname, params)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetShaderInfoLog(
    shader: types::GLuint,
    bufSize: types::GLsizei,
    length: *mut types::GLsizei,
    infoLog: *mut types::GLchar,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLsizei,
            *mut types::GLsizei,
            *mut types::GLchar,
        ) -> (),
    >(storage::GetShaderInfoLog.f)(shader, bufSize, length, infoLog)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetShaderPrecisionFormat(
    shadertype: types::GLenum,
    precisiontype: types::GLenum,
    range: *mut types::GLint,
    precision: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLenum,
            *mut types::GLint,
            *mut types::GLint,
        ) -> (),
    >(storage::GetShaderPrecisionFormat.f)(shadertype, precisiontype, range, precision)
}
/// Fallbacks: GetShaderSourceARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetShaderSource(
    shader: types::GLuint,
    bufSize: types::GLsizei,
    length: *mut types::GLsizei,
    source: *mut types::GLchar,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLsizei,
            *mut types::GLsizei,
            *mut types::GLchar,
        ) -> (),
    >(storage::GetShaderSource.f)(shader, bufSize, length, source)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetShaderiv(
    shader: types::GLuint,
    pname: types::GLenum,
    params: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLenum, *mut types::GLint) -> (),
    >(storage::GetShaderiv.f)(shader, pname, params)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetString(name: types::GLenum) -> *const types::GLubyte {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> *const types::GLubyte>(
        storage::GetString.f,
    )(name)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetTexParameterfv(
    target: types::GLenum,
    pname: types::GLenum,
    params: *mut types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, *mut types::GLfloat) -> (),
    >(storage::GetTexParameterfv.f)(target, pname, params)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetTexParameteriv(
    target: types::GLenum,
    pname: types::GLenum,
    params: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, *mut types::GLint) -> (),
    >(storage::GetTexParameteriv.f)(target, pname, params)
}
/// Fallbacks: GetUniformLocationARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetUniformLocation(
    program: types::GLuint,
    name: *const types::GLchar,
) -> types::GLint {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, *const types::GLchar) -> types::GLint,
    >(storage::GetUniformLocation.f)(program, name)
}
/// Fallbacks: GetUniformfvARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetUniformfv(
    program: types::GLuint,
    location: types::GLint,
    params: *mut types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLint, *mut types::GLfloat) -> (),
    >(storage::GetUniformfv.f)(program, location, params)
}
/// Fallbacks: GetUniformivARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetUniformiv(
    program: types::GLuint,
    location: types::GLint,
    params: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLint, *mut types::GLint) -> (),
    >(storage::GetUniformiv.f)(program, location, params)
}
/// Fallbacks: GetVertexAttribPointervARB, GetVertexAttribPointervNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetVertexAttribPointerv(
    index: types::GLuint,
    pname: types::GLenum,
    pointer: *const *mut __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLenum,
            *const *mut __gl_imports::raw::c_void,
        ) -> (),
    >(storage::GetVertexAttribPointerv.f)(index, pname, pointer)
}
/// Fallbacks: GetVertexAttribfvARB, GetVertexAttribfvNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetVertexAttribfv(
    index: types::GLuint,
    pname: types::GLenum,
    params: *mut types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLenum, *mut types::GLfloat) -> (),
    >(storage::GetVertexAttribfv.f)(index, pname, params)
}
/// Fallbacks: GetVertexAttribivARB, GetVertexAttribivNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn GetVertexAttribiv(
    index: types::GLuint,
    pname: types::GLenum,
    params: *mut types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLenum, *mut types::GLint) -> (),
    >(storage::GetVertexAttribiv.f)(index, pname, params)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Hint(target: types::GLenum, mode: types::GLenum) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLenum) -> ()>(
        storage::Hint.f,
    )(target, mode)
}
/// Fallbacks: IsBufferARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn IsBuffer(buffer: types::GLuint) -> types::GLboolean {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> types::GLboolean>(
        storage::IsBuffer.f,
    )(buffer)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn IsEnabled(cap: types::GLenum) -> types::GLboolean {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum) -> types::GLboolean>(
        storage::IsEnabled.f,
    )(cap)
}
/// Fallbacks: IsFramebufferEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn IsFramebuffer(framebuffer: types::GLuint) -> types::GLboolean {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> types::GLboolean>(
        storage::IsFramebuffer.f,
    )(framebuffer)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn IsProgram(program: types::GLuint) -> types::GLboolean {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> types::GLboolean>(
        storage::IsProgram.f,
    )(program)
}
/// Fallbacks: IsRenderbufferEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn IsRenderbuffer(renderbuffer: types::GLuint) -> types::GLboolean {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> types::GLboolean>(
        storage::IsRenderbuffer.f,
    )(renderbuffer)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn IsShader(shader: types::GLuint) -> types::GLboolean {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> types::GLboolean>(
        storage::IsShader.f,
    )(shader)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn IsTexture(texture: types::GLuint) -> types::GLboolean {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> types::GLboolean>(
        storage::IsTexture.f,
    )(texture)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn LineWidth(width: types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLfloat) -> ()>(
        storage::LineWidth.f,
    )(width)
}
/// Fallbacks: LinkProgramARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn LinkProgram(program: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::LinkProgram.f,
    )(program)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn PixelStorei(pname: types::GLenum, param: types::GLint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLint) -> ()>(
        storage::PixelStorei.f,
    )(pname, param)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn PolygonOffset(factor: types::GLfloat, units: types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLfloat, types::GLfloat) -> ()>(
        storage::PolygonOffset.f,
    )(factor, units)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ReadPixels(
    x: types::GLint,
    y: types::GLint,
    width: types::GLsizei,
    height: types::GLsizei,
    format: types::GLenum,
    type_: types::GLenum,
    pixels: *mut __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLint,
            types::GLint,
            types::GLsizei,
            types::GLsizei,
            types::GLenum,
            types::GLenum,
            *mut __gl_imports::raw::c_void,
        ) -> (),
    >(storage::ReadPixels.f)(x, y, width, height, format, type_, pixels)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ReleaseShaderCompiler() -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn() -> ()>(storage::ReleaseShaderCompiler.f)(
    )
}
/// Fallbacks: RenderbufferStorageEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn RenderbufferStorage(
    target: types::GLenum,
    internalformat: types::GLenum,
    width: types::GLsizei,
    height: types::GLsizei,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLsizei, types::GLsizei) -> (),
    >(storage::RenderbufferStorage.f)(target, internalformat, width, height)
}
/// Fallbacks: SampleCoverageARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn SampleCoverage(value: types::GLfloat, invert: types::GLboolean) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLfloat, types::GLboolean) -> ()>(
        storage::SampleCoverage.f,
    )(value, invert)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Scissor(
    x: types::GLint,
    y: types::GLint,
    width: types::GLsizei,
    height: types::GLsizei,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLint, types::GLsizei, types::GLsizei) -> (),
    >(storage::Scissor.f)(x, y, width, height)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ShaderBinary(
    count: types::GLsizei,
    shaders: *const types::GLuint,
    binaryformat: types::GLenum,
    binary: *const __gl_imports::raw::c_void,
    length: types::GLsizei,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLsizei,
            *const types::GLuint,
            types::GLenum,
            *const __gl_imports::raw::c_void,
            types::GLsizei,
        ) -> (),
    >(storage::ShaderBinary.f)(count, shaders, binaryformat, binary, length)
}
/// Fallbacks: ShaderSourceARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ShaderSource(
    shader: types::GLuint,
    count: types::GLsizei,
    string: *const *const types::GLchar,
    length: *const types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLsizei,
            *const *const types::GLchar,
            *const types::GLint,
        ) -> (),
    >(storage::ShaderSource.f)(shader, count, string, length)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StencilFunc(func: types::GLenum, ref_: types::GLint, mask: types::GLuint) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLint, types::GLuint) -> (),
    >(storage::StencilFunc.f)(func, ref_, mask)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StencilFuncSeparate(
    face: types::GLenum,
    func: types::GLenum,
    ref_: types::GLint,
    mask: types::GLuint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLint, types::GLuint) -> (),
    >(storage::StencilFuncSeparate.f)(face, func, ref_, mask)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StencilMask(mask: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::StencilMask.f,
    )(mask)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StencilMaskSeparate(face: types::GLenum, mask: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLenum, types::GLuint) -> ()>(
        storage::StencilMaskSeparate.f,
    )(face, mask)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StencilOp(fail: types::GLenum, zfail: types::GLenum, zpass: types::GLenum) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLenum) -> (),
    >(storage::StencilOp.f)(fail, zfail, zpass)
}
/// Fallbacks: StencilOpSeparateATI
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn StencilOpSeparate(
    face: types::GLenum,
    sfail: types::GLenum,
    dpfail: types::GLenum,
    dppass: types::GLenum,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLenum, types::GLenum) -> (),
    >(storage::StencilOpSeparate.f)(face, sfail, dpfail, dppass)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn TexImage2D(
    target: types::GLenum,
    level: types::GLint,
    internalformat: types::GLint,
    width: types::GLsizei,
    height: types::GLsizei,
    border: types::GLint,
    format: types::GLenum,
    type_: types::GLenum,
    pixels: *const __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLint,
            types::GLint,
            types::GLsizei,
            types::GLsizei,
            types::GLint,
            types::GLenum,
            types::GLenum,
            *const __gl_imports::raw::c_void,
        ) -> (),
    >(storage::TexImage2D.f)(
        target,
        level,
        internalformat,
        width,
        height,
        border,
        format,
        type_,
        pixels,
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn TexParameterf(
    target: types::GLenum,
    pname: types::GLenum,
    param: types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLfloat) -> (),
    >(storage::TexParameterf.f)(target, pname, param)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn TexParameterfv(
    target: types::GLenum,
    pname: types::GLenum,
    params: *const types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, *const types::GLfloat) -> (),
    >(storage::TexParameterfv.f)(target, pname, params)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn TexParameteri(
    target: types::GLenum,
    pname: types::GLenum,
    param: types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, types::GLint) -> (),
    >(storage::TexParameteri.f)(target, pname, param)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn TexParameteriv(
    target: types::GLenum,
    pname: types::GLenum,
    params: *const types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLenum, types::GLenum, *const types::GLint) -> (),
    >(storage::TexParameteriv.f)(target, pname, params)
}
/// Fallbacks: TexSubImage2DEXT
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn TexSubImage2D(
    target: types::GLenum,
    level: types::GLint,
    xoffset: types::GLint,
    yoffset: types::GLint,
    width: types::GLsizei,
    height: types::GLsizei,
    format: types::GLenum,
    type_: types::GLenum,
    pixels: *const __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLenum,
            types::GLint,
            types::GLint,
            types::GLint,
            types::GLsizei,
            types::GLsizei,
            types::GLenum,
            types::GLenum,
            *const __gl_imports::raw::c_void,
        ) -> (),
    >(storage::TexSubImage2D.f)(
        target, level, xoffset, yoffset, width, height, format, type_, pixels,
    )
}
/// Fallbacks: Uniform1fARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform1f(location: types::GLint, v0: types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLint, types::GLfloat) -> ()>(
        storage::Uniform1f.f,
    )(location, v0)
}
/// Fallbacks: Uniform1fvARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform1fv(
    location: types::GLint,
    count: types::GLsizei,
    value: *const types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLsizei, *const types::GLfloat) -> (),
    >(storage::Uniform1fv.f)(location, count, value)
}
/// Fallbacks: Uniform1iARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform1i(location: types::GLint, v0: types::GLint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLint, types::GLint) -> ()>(
        storage::Uniform1i.f,
    )(location, v0)
}
/// Fallbacks: Uniform1ivARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform1iv(
    location: types::GLint,
    count: types::GLsizei,
    value: *const types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLsizei, *const types::GLint) -> (),
    >(storage::Uniform1iv.f)(location, count, value)
}
/// Fallbacks: Uniform2fARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform2f(location: types::GLint, v0: types::GLfloat, v1: types::GLfloat) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLfloat, types::GLfloat) -> (),
    >(storage::Uniform2f.f)(location, v0, v1)
}
/// Fallbacks: Uniform2fvARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform2fv(
    location: types::GLint,
    count: types::GLsizei,
    value: *const types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLsizei, *const types::GLfloat) -> (),
    >(storage::Uniform2fv.f)(location, count, value)
}
/// Fallbacks: Uniform2iARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform2i(location: types::GLint, v0: types::GLint, v1: types::GLint) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLint, types::GLint) -> (),
    >(storage::Uniform2i.f)(location, v0, v1)
}
/// Fallbacks: Uniform2ivARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform2iv(
    location: types::GLint,
    count: types::GLsizei,
    value: *const types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLsizei, *const types::GLint) -> (),
    >(storage::Uniform2iv.f)(location, count, value)
}
/// Fallbacks: Uniform3fARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform3f(
    location: types::GLint,
    v0: types::GLfloat,
    v1: types::GLfloat,
    v2: types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLfloat, types::GLfloat, types::GLfloat) -> (),
    >(storage::Uniform3f.f)(location, v0, v1, v2)
}
/// Fallbacks: Uniform3fvARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform3fv(
    location: types::GLint,
    count: types::GLsizei,
    value: *const types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLsizei, *const types::GLfloat) -> (),
    >(storage::Uniform3fv.f)(location, count, value)
}
/// Fallbacks: Uniform3iARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform3i(
    location: types::GLint,
    v0: types::GLint,
    v1: types::GLint,
    v2: types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLint, types::GLint, types::GLint) -> (),
    >(storage::Uniform3i.f)(location, v0, v1, v2)
}
/// Fallbacks: Uniform3ivARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform3iv(
    location: types::GLint,
    count: types::GLsizei,
    value: *const types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLsizei, *const types::GLint) -> (),
    >(storage::Uniform3iv.f)(location, count, value)
}
/// Fallbacks: Uniform4fARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform4f(
    location: types::GLint,
    v0: types::GLfloat,
    v1: types::GLfloat,
    v2: types::GLfloat,
    v3: types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLint,
            types::GLfloat,
            types::GLfloat,
            types::GLfloat,
            types::GLfloat,
        ) -> (),
    >(storage::Uniform4f.f)(location, v0, v1, v2, v3)
}
/// Fallbacks: Uniform4fvARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform4fv(
    location: types::GLint,
    count: types::GLsizei,
    value: *const types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLsizei, *const types::GLfloat) -> (),
    >(storage::Uniform4fv.f)(location, count, value)
}
/// Fallbacks: Uniform4iARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform4i(
    location: types::GLint,
    v0: types::GLint,
    v1: types::GLint,
    v2: types::GLint,
    v3: types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLint,
            types::GLint,
            types::GLint,
            types::GLint,
            types::GLint,
        ) -> (),
    >(storage::Uniform4i.f)(location, v0, v1, v2, v3)
}
/// Fallbacks: Uniform4ivARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Uniform4iv(
    location: types::GLint,
    count: types::GLsizei,
    value: *const types::GLint,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLsizei, *const types::GLint) -> (),
    >(storage::Uniform4iv.f)(location, count, value)
}
/// Fallbacks: UniformMatrix2fvARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn UniformMatrix2fv(
    location: types::GLint,
    count: types::GLsizei,
    transpose: types::GLboolean,
    value: *const types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLint,
            types::GLsizei,
            types::GLboolean,
            *const types::GLfloat,
        ) -> (),
    >(storage::UniformMatrix2fv.f)(location, count, transpose, value)
}
/// Fallbacks: UniformMatrix3fvARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn UniformMatrix3fv(
    location: types::GLint,
    count: types::GLsizei,
    transpose: types::GLboolean,
    value: *const types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLint,
            types::GLsizei,
            types::GLboolean,
            *const types::GLfloat,
        ) -> (),
    >(storage::UniformMatrix3fv.f)(location, count, transpose, value)
}
/// Fallbacks: UniformMatrix4fvARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn UniformMatrix4fv(
    location: types::GLint,
    count: types::GLsizei,
    transpose: types::GLboolean,
    value: *const types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLint,
            types::GLsizei,
            types::GLboolean,
            *const types::GLfloat,
        ) -> (),
    >(storage::UniformMatrix4fv.f)(location, count, transpose, value)
}
/// Fallbacks: UseProgramObjectARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn UseProgram(program: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::UseProgram.f,
    )(program)
}
/// Fallbacks: ValidateProgramARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn ValidateProgram(program: types::GLuint) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint) -> ()>(
        storage::ValidateProgram.f,
    )(program)
}
/// Fallbacks: VertexAttrib1fARB, VertexAttrib1fNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttrib1f(index: types::GLuint, x: types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint, types::GLfloat) -> ()>(
        storage::VertexAttrib1f.f,
    )(index, x)
}
/// Fallbacks: VertexAttrib1fvARB, VertexAttrib1fvNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttrib1fv(index: types::GLuint, v: *const types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint, *const types::GLfloat) -> ()>(
        storage::VertexAttrib1fv.f,
    )(index, v)
}
/// Fallbacks: VertexAttrib2fARB, VertexAttrib2fNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttrib2f(index: types::GLuint, x: types::GLfloat, y: types::GLfloat) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLfloat, types::GLfloat) -> (),
    >(storage::VertexAttrib2f.f)(index, x, y)
}
/// Fallbacks: VertexAttrib2fvARB, VertexAttrib2fvNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttrib2fv(index: types::GLuint, v: *const types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint, *const types::GLfloat) -> ()>(
        storage::VertexAttrib2fv.f,
    )(index, v)
}
/// Fallbacks: VertexAttrib3fARB, VertexAttrib3fNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttrib3f(
    index: types::GLuint,
    x: types::GLfloat,
    y: types::GLfloat,
    z: types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLuint, types::GLfloat, types::GLfloat, types::GLfloat) -> (),
    >(storage::VertexAttrib3f.f)(index, x, y, z)
}
/// Fallbacks: VertexAttrib3fvARB, VertexAttrib3fvNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttrib3fv(index: types::GLuint, v: *const types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint, *const types::GLfloat) -> ()>(
        storage::VertexAttrib3fv.f,
    )(index, v)
}
/// Fallbacks: VertexAttrib4fARB, VertexAttrib4fNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttrib4f(
    index: types::GLuint,
    x: types::GLfloat,
    y: types::GLfloat,
    z: types::GLfloat,
    w: types::GLfloat,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLfloat,
            types::GLfloat,
            types::GLfloat,
            types::GLfloat,
        ) -> (),
    >(storage::VertexAttrib4f.f)(index, x, y, z, w)
}
/// Fallbacks: VertexAttrib4fvARB, VertexAttrib4fvNV
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttrib4fv(index: types::GLuint, v: *const types::GLfloat) -> () {
    __gl_imports::mem::transmute::<_, extern "system" fn(types::GLuint, *const types::GLfloat) -> ()>(
        storage::VertexAttrib4fv.f,
    )(index, v)
}
/// Fallbacks: VertexAttribPointerARB
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn VertexAttribPointer(
    index: types::GLuint,
    size: types::GLint,
    type_: types::GLenum,
    normalized: types::GLboolean,
    stride: types::GLsizei,
    pointer: *const __gl_imports::raw::c_void,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(
            types::GLuint,
            types::GLint,
            types::GLenum,
            types::GLboolean,
            types::GLsizei,
            *const __gl_imports::raw::c_void,
        ) -> (),
    >(storage::VertexAttribPointer.f)(index, size, type_, normalized, stride, pointer)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline]
pub unsafe fn Viewport(
    x: types::GLint,
    y: types::GLint,
    width: types::GLsizei,
    height: types::GLsizei,
) -> () {
    __gl_imports::mem::transmute::<
        _,
        extern "system" fn(types::GLint, types::GLint, types::GLsizei, types::GLsizei) -> (),
    >(storage::Viewport.f)(x, y, width, height)
}

#[allow(missing_copy_implementations)]
pub struct FnPtr {
    /// The function pointer that will be used when calling the function.
    f: *const __gl_imports::raw::c_void,
    /// True if the pointer points to a real function, false if points to a `panic!` fn.
    is_loaded: bool,
}

impl FnPtr {
    /// Creates a `FnPtr` from a load attempt.
    pub fn new(ptr: *const __gl_imports::raw::c_void) -> FnPtr {
        if ptr.is_null() {
            FnPtr {
                f: missing_fn_panic as *const __gl_imports::raw::c_void,
                is_loaded: false,
            }
        } else {
            FnPtr {
                f: ptr,
                is_loaded: true,
            }
        }
    }
}

mod storage {
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    use super::FnPtr;
    use super::__gl_imports::raw;
    pub static mut ActiveTexture: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut AttachShader: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BindAttribLocation: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BindBuffer: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BindFramebuffer: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BindRenderbuffer: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BindTexture: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BlendColor: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BlendEquation: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BlendEquationSeparate: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BlendFunc: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BlendFuncSeparate: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BufferData: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut BufferSubData: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CheckFramebufferStatus: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Clear: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ClearColor: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ClearDepthf: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ClearStencil: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ColorMask: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CompileShader: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CompressedTexImage2D: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CompressedTexSubImage2D: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CopyTexImage2D: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CopyTexSubImage2D: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CreateProgram: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CreateShader: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut CullFace: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DeleteBuffers: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DeleteFramebuffers: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DeleteProgram: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DeleteRenderbuffers: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DeleteShader: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DeleteTextures: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DepthFunc: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DepthMask: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DepthRangef: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DetachShader: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Disable: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DisableVertexAttribArray: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DrawArrays: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut DrawElements: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Enable: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut EnableVertexAttribArray: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Finish: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Flush: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut FramebufferRenderbuffer: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut FramebufferTexture2D: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut FrontFace: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GenBuffers: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GenFramebuffers: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GenRenderbuffers: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GenTextures: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GenerateMipmap: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetActiveAttrib: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetActiveUniform: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetAttachedShaders: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetAttribLocation: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetBooleanv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetBufferParameteriv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetError: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetFloatv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetFramebufferAttachmentParameteriv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetIntegerv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetProgramInfoLog: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetProgramiv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetRenderbufferParameteriv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetShaderInfoLog: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetShaderPrecisionFormat: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetShaderSource: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetShaderiv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetString: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetTexParameterfv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetTexParameteriv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetUniformLocation: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetUniformfv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetUniformiv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetVertexAttribPointerv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetVertexAttribfv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut GetVertexAttribiv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Hint: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut IsBuffer: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut IsEnabled: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut IsFramebuffer: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut IsProgram: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut IsRenderbuffer: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut IsShader: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut IsTexture: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut LineWidth: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut LinkProgram: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut PixelStorei: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut PolygonOffset: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ReadPixels: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ReleaseShaderCompiler: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut RenderbufferStorage: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut SampleCoverage: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Scissor: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ShaderBinary: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ShaderSource: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StencilFunc: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StencilFuncSeparate: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StencilMask: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StencilMaskSeparate: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StencilOp: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut StencilOpSeparate: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut TexImage2D: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut TexParameterf: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut TexParameterfv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut TexParameteri: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut TexParameteriv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut TexSubImage2D: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform1f: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform1fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform1i: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform1iv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform2f: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform2fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform2i: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform2iv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform3f: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform3fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform3i: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform3iv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform4f: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform4fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform4i: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Uniform4iv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut UniformMatrix2fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut UniformMatrix3fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut UniformMatrix4fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut UseProgram: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut ValidateProgram: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttrib1f: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttrib1fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttrib2f: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttrib2fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttrib3f: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttrib3fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttrib4f: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttrib4fv: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut VertexAttribPointer: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
    pub static mut Viewport: FnPtr = FnPtr {
        f: super::missing_fn_panic as *const raw::c_void,
        is_loaded: false,
    };
}

#[allow(non_snake_case)]
pub mod ActiveTexture {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ActiveTexture.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::ActiveTexture = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glActiveTexture",
                &["glActiveTextureARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod AttachShader {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::AttachShader.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::AttachShader = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glAttachShader",
                &["glAttachObjectARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BindAttribLocation {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BindAttribLocation.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BindAttribLocation = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBindAttribLocation",
                &["glBindAttribLocationARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BindBuffer {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BindBuffer.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BindBuffer = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBindBuffer",
                &["glBindBufferARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BindFramebuffer {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BindFramebuffer.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BindFramebuffer = FnPtr::new(metaloadfn(&mut loadfn, "glBindFramebuffer", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod BindRenderbuffer {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BindRenderbuffer.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BindRenderbuffer =
                FnPtr::new(metaloadfn(&mut loadfn, "glBindRenderbuffer", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod BindTexture {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BindTexture.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BindTexture = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBindTexture",
                &["glBindTextureEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BlendColor {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BlendColor.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BlendColor = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBlendColor",
                &["glBlendColorEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BlendEquation {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BlendEquation.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BlendEquation = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBlendEquation",
                &["glBlendEquationEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BlendEquationSeparate {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BlendEquationSeparate.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BlendEquationSeparate = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBlendEquationSeparate",
                &["glBlendEquationSeparateEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BlendFunc {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BlendFunc.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::BlendFunc = FnPtr::new(metaloadfn(&mut loadfn, "glBlendFunc", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod BlendFuncSeparate {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BlendFuncSeparate.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BlendFuncSeparate = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBlendFuncSeparate",
                &["glBlendFuncSeparateEXT", "glBlendFuncSeparateINGR"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BufferData {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BufferData.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BufferData = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBufferData",
                &["glBufferDataARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod BufferSubData {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::BufferSubData.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::BufferSubData = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glBufferSubData",
                &["glBufferSubDataARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod CheckFramebufferStatus {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CheckFramebufferStatus.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::CheckFramebufferStatus = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glCheckFramebufferStatus",
                &["glCheckFramebufferStatusEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Clear {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Clear.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::Clear = FnPtr::new(metaloadfn(&mut loadfn, "glClear", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod ClearColor {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ClearColor.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::ClearColor = FnPtr::new(metaloadfn(&mut loadfn, "glClearColor", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod ClearDepthf {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ClearDepthf.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::ClearDepthf = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glClearDepthf",
                &["glClearDepthfOES"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod ClearStencil {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ClearStencil.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::ClearStencil = FnPtr::new(metaloadfn(&mut loadfn, "glClearStencil", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod ColorMask {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ColorMask.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::ColorMask = FnPtr::new(metaloadfn(&mut loadfn, "glColorMask", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod CompileShader {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CompileShader.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::CompileShader = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glCompileShader",
                &["glCompileShaderARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod CompressedTexImage2D {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CompressedTexImage2D.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::CompressedTexImage2D = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glCompressedTexImage2D",
                &["glCompressedTexImage2DARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod CompressedTexSubImage2D {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CompressedTexSubImage2D.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::CompressedTexSubImage2D = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glCompressedTexSubImage2D",
                &["glCompressedTexSubImage2DARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod CopyTexImage2D {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CopyTexImage2D.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::CopyTexImage2D = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glCopyTexImage2D",
                &["glCopyTexImage2DEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod CopyTexSubImage2D {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CopyTexSubImage2D.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::CopyTexSubImage2D = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glCopyTexSubImage2D",
                &["glCopyTexSubImage2DEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod CreateProgram {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CreateProgram.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::CreateProgram = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glCreateProgram",
                &["glCreateProgramObjectARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod CreateShader {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CreateShader.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::CreateShader = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glCreateShader",
                &["glCreateShaderObjectARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod CullFace {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::CullFace.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::CullFace = FnPtr::new(metaloadfn(&mut loadfn, "glCullFace", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod DeleteBuffers {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DeleteBuffers.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DeleteBuffers = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glDeleteBuffers",
                &["glDeleteBuffersARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod DeleteFramebuffers {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DeleteFramebuffers.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DeleteFramebuffers = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glDeleteFramebuffers",
                &["glDeleteFramebuffersEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod DeleteProgram {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DeleteProgram.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DeleteProgram = FnPtr::new(metaloadfn(&mut loadfn, "glDeleteProgram", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod DeleteRenderbuffers {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DeleteRenderbuffers.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DeleteRenderbuffers = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glDeleteRenderbuffers",
                &["glDeleteRenderbuffersEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod DeleteShader {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DeleteShader.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DeleteShader = FnPtr::new(metaloadfn(&mut loadfn, "glDeleteShader", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod DeleteTextures {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DeleteTextures.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DeleteTextures = FnPtr::new(metaloadfn(&mut loadfn, "glDeleteTextures", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod DepthFunc {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DepthFunc.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::DepthFunc = FnPtr::new(metaloadfn(&mut loadfn, "glDepthFunc", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod DepthMask {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DepthMask.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::DepthMask = FnPtr::new(metaloadfn(&mut loadfn, "glDepthMask", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod DepthRangef {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DepthRangef.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DepthRangef = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glDepthRangef",
                &["glDepthRangefOES"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod DetachShader {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DetachShader.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DetachShader = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glDetachShader",
                &["glDetachObjectARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Disable {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Disable.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::Disable = FnPtr::new(metaloadfn(&mut loadfn, "glDisable", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod DisableVertexAttribArray {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DisableVertexAttribArray.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DisableVertexAttribArray = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glDisableVertexAttribArray",
                &["glDisableVertexAttribArrayARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod DrawArrays {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DrawArrays.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DrawArrays = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glDrawArrays",
                &["glDrawArraysEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod DrawElements {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::DrawElements.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::DrawElements = FnPtr::new(metaloadfn(&mut loadfn, "glDrawElements", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Enable {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Enable.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::Enable = FnPtr::new(metaloadfn(&mut loadfn, "glEnable", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod EnableVertexAttribArray {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::EnableVertexAttribArray.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::EnableVertexAttribArray = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glEnableVertexAttribArray",
                &["glEnableVertexAttribArrayARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Finish {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Finish.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::Finish = FnPtr::new(metaloadfn(&mut loadfn, "glFinish", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod Flush {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Flush.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::Flush = FnPtr::new(metaloadfn(&mut loadfn, "glFlush", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod FramebufferRenderbuffer {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::FramebufferRenderbuffer.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::FramebufferRenderbuffer = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glFramebufferRenderbuffer",
                &["glFramebufferRenderbufferEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod FramebufferTexture2D {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::FramebufferTexture2D.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::FramebufferTexture2D = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glFramebufferTexture2D",
                &["glFramebufferTexture2DEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod FrontFace {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::FrontFace.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::FrontFace = FnPtr::new(metaloadfn(&mut loadfn, "glFrontFace", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod GenBuffers {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GenBuffers.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GenBuffers = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGenBuffers",
                &["glGenBuffersARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GenFramebuffers {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GenFramebuffers.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GenFramebuffers = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGenFramebuffers",
                &["glGenFramebuffersEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GenRenderbuffers {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GenRenderbuffers.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GenRenderbuffers = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGenRenderbuffers",
                &["glGenRenderbuffersEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GenTextures {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GenTextures.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::GenTextures = FnPtr::new(metaloadfn(&mut loadfn, "glGenTextures", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod GenerateMipmap {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GenerateMipmap.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GenerateMipmap = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGenerateMipmap",
                &["glGenerateMipmapEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetActiveAttrib {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetActiveAttrib.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetActiveAttrib = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetActiveAttrib",
                &["glGetActiveAttribARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetActiveUniform {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetActiveUniform.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetActiveUniform = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetActiveUniform",
                &["glGetActiveUniformARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetAttachedShaders {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetAttachedShaders.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetAttachedShaders =
                FnPtr::new(metaloadfn(&mut loadfn, "glGetAttachedShaders", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetAttribLocation {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetAttribLocation.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetAttribLocation = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetAttribLocation",
                &["glGetAttribLocationARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetBooleanv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetBooleanv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::GetBooleanv = FnPtr::new(metaloadfn(&mut loadfn, "glGetBooleanv", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod GetBufferParameteriv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetBufferParameteriv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetBufferParameteriv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetBufferParameteriv",
                &["glGetBufferParameterivARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetError {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetError.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::GetError = FnPtr::new(metaloadfn(&mut loadfn, "glGetError", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod GetFloatv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetFloatv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::GetFloatv = FnPtr::new(metaloadfn(&mut loadfn, "glGetFloatv", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod GetFramebufferAttachmentParameteriv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetFramebufferAttachmentParameteriv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetFramebufferAttachmentParameteriv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetFramebufferAttachmentParameteriv",
                &["glGetFramebufferAttachmentParameterivEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetIntegerv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetIntegerv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::GetIntegerv = FnPtr::new(metaloadfn(&mut loadfn, "glGetIntegerv", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod GetProgramInfoLog {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetProgramInfoLog.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetProgramInfoLog =
                FnPtr::new(metaloadfn(&mut loadfn, "glGetProgramInfoLog", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetProgramiv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetProgramiv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetProgramiv = FnPtr::new(metaloadfn(&mut loadfn, "glGetProgramiv", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetRenderbufferParameteriv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetRenderbufferParameteriv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetRenderbufferParameteriv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetRenderbufferParameteriv",
                &["glGetRenderbufferParameterivEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetShaderInfoLog {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetShaderInfoLog.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetShaderInfoLog =
                FnPtr::new(metaloadfn(&mut loadfn, "glGetShaderInfoLog", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetShaderPrecisionFormat {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetShaderPrecisionFormat.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetShaderPrecisionFormat =
                FnPtr::new(metaloadfn(&mut loadfn, "glGetShaderPrecisionFormat", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetShaderSource {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetShaderSource.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetShaderSource = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetShaderSource",
                &["glGetShaderSourceARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetShaderiv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetShaderiv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::GetShaderiv = FnPtr::new(metaloadfn(&mut loadfn, "glGetShaderiv", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod GetString {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetString.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::GetString = FnPtr::new(metaloadfn(&mut loadfn, "glGetString", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod GetTexParameterfv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetTexParameterfv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetTexParameterfv =
                FnPtr::new(metaloadfn(&mut loadfn, "glGetTexParameterfv", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetTexParameteriv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetTexParameteriv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetTexParameteriv =
                FnPtr::new(metaloadfn(&mut loadfn, "glGetTexParameteriv", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetUniformLocation {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetUniformLocation.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetUniformLocation = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetUniformLocation",
                &["glGetUniformLocationARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetUniformfv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetUniformfv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetUniformfv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetUniformfv",
                &["glGetUniformfvARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetUniformiv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetUniformiv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetUniformiv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetUniformiv",
                &["glGetUniformivARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetVertexAttribPointerv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetVertexAttribPointerv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetVertexAttribPointerv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetVertexAttribPointerv",
                &[
                    "glGetVertexAttribPointervARB",
                    "glGetVertexAttribPointervNV",
                ],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetVertexAttribfv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetVertexAttribfv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetVertexAttribfv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetVertexAttribfv",
                &["glGetVertexAttribfvARB", "glGetVertexAttribfvNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod GetVertexAttribiv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::GetVertexAttribiv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::GetVertexAttribiv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glGetVertexAttribiv",
                &["glGetVertexAttribivARB", "glGetVertexAttribivNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Hint {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Hint.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::Hint = FnPtr::new(metaloadfn(&mut loadfn, "glHint", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod IsBuffer {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::IsBuffer.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::IsBuffer =
                FnPtr::new(metaloadfn(&mut loadfn, "glIsBuffer", &["glIsBufferARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod IsEnabled {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::IsEnabled.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::IsEnabled = FnPtr::new(metaloadfn(&mut loadfn, "glIsEnabled", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod IsFramebuffer {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::IsFramebuffer.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::IsFramebuffer = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glIsFramebuffer",
                &["glIsFramebufferEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod IsProgram {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::IsProgram.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::IsProgram = FnPtr::new(metaloadfn(&mut loadfn, "glIsProgram", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod IsRenderbuffer {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::IsRenderbuffer.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::IsRenderbuffer = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glIsRenderbuffer",
                &["glIsRenderbufferEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod IsShader {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::IsShader.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::IsShader = FnPtr::new(metaloadfn(&mut loadfn, "glIsShader", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod IsTexture {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::IsTexture.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::IsTexture = FnPtr::new(metaloadfn(&mut loadfn, "glIsTexture", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod LineWidth {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::LineWidth.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::LineWidth = FnPtr::new(metaloadfn(&mut loadfn, "glLineWidth", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod LinkProgram {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::LinkProgram.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::LinkProgram = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glLinkProgram",
                &["glLinkProgramARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod PixelStorei {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::PixelStorei.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::PixelStorei = FnPtr::new(metaloadfn(&mut loadfn, "glPixelStorei", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod PolygonOffset {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::PolygonOffset.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::PolygonOffset = FnPtr::new(metaloadfn(&mut loadfn, "glPolygonOffset", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod ReadPixels {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ReadPixels.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::ReadPixels = FnPtr::new(metaloadfn(&mut loadfn, "glReadPixels", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod ReleaseShaderCompiler {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ReleaseShaderCompiler.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::ReleaseShaderCompiler =
                FnPtr::new(metaloadfn(&mut loadfn, "glReleaseShaderCompiler", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod RenderbufferStorage {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::RenderbufferStorage.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::RenderbufferStorage = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glRenderbufferStorage",
                &["glRenderbufferStorageEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod SampleCoverage {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::SampleCoverage.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::SampleCoverage = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glSampleCoverage",
                &["glSampleCoverageARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Scissor {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Scissor.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::Scissor = FnPtr::new(metaloadfn(&mut loadfn, "glScissor", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod ShaderBinary {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ShaderBinary.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::ShaderBinary = FnPtr::new(metaloadfn(&mut loadfn, "glShaderBinary", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod ShaderSource {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ShaderSource.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::ShaderSource = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glShaderSource",
                &["glShaderSourceARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod StencilFunc {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::StencilFunc.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::StencilFunc = FnPtr::new(metaloadfn(&mut loadfn, "glStencilFunc", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod StencilFuncSeparate {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::StencilFuncSeparate.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::StencilFuncSeparate =
                FnPtr::new(metaloadfn(&mut loadfn, "glStencilFuncSeparate", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod StencilMask {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::StencilMask.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::StencilMask = FnPtr::new(metaloadfn(&mut loadfn, "glStencilMask", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod StencilMaskSeparate {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::StencilMaskSeparate.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::StencilMaskSeparate =
                FnPtr::new(metaloadfn(&mut loadfn, "glStencilMaskSeparate", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod StencilOp {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::StencilOp.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::StencilOp = FnPtr::new(metaloadfn(&mut loadfn, "glStencilOp", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod StencilOpSeparate {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::StencilOpSeparate.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::StencilOpSeparate = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glStencilOpSeparate",
                &["glStencilOpSeparateATI"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod TexImage2D {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::TexImage2D.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::TexImage2D = FnPtr::new(metaloadfn(&mut loadfn, "glTexImage2D", &[])) }
    }
}

#[allow(non_snake_case)]
pub mod TexParameterf {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::TexParameterf.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::TexParameterf = FnPtr::new(metaloadfn(&mut loadfn, "glTexParameterf", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod TexParameterfv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::TexParameterfv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::TexParameterfv = FnPtr::new(metaloadfn(&mut loadfn, "glTexParameterfv", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod TexParameteri {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::TexParameteri.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::TexParameteri = FnPtr::new(metaloadfn(&mut loadfn, "glTexParameteri", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod TexParameteriv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::TexParameteriv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::TexParameteriv = FnPtr::new(metaloadfn(&mut loadfn, "glTexParameteriv", &[]))
        }
    }
}

#[allow(non_snake_case)]
pub mod TexSubImage2D {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::TexSubImage2D.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::TexSubImage2D = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glTexSubImage2D",
                &["glTexSubImage2DEXT"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform1f {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform1f.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform1f =
                FnPtr::new(metaloadfn(&mut loadfn, "glUniform1f", &["glUniform1fARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform1fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform1fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform1fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniform1fv",
                &["glUniform1fvARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform1i {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform1i.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform1i =
                FnPtr::new(metaloadfn(&mut loadfn, "glUniform1i", &["glUniform1iARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform1iv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform1iv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform1iv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniform1iv",
                &["glUniform1ivARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform2f {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform2f.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform2f =
                FnPtr::new(metaloadfn(&mut loadfn, "glUniform2f", &["glUniform2fARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform2fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform2fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform2fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniform2fv",
                &["glUniform2fvARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform2i {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform2i.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform2i =
                FnPtr::new(metaloadfn(&mut loadfn, "glUniform2i", &["glUniform2iARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform2iv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform2iv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform2iv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniform2iv",
                &["glUniform2ivARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform3f {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform3f.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform3f =
                FnPtr::new(metaloadfn(&mut loadfn, "glUniform3f", &["glUniform3fARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform3fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform3fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform3fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniform3fv",
                &["glUniform3fvARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform3i {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform3i.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform3i =
                FnPtr::new(metaloadfn(&mut loadfn, "glUniform3i", &["glUniform3iARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform3iv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform3iv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform3iv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniform3iv",
                &["glUniform3ivARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform4f {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform4f.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform4f =
                FnPtr::new(metaloadfn(&mut loadfn, "glUniform4f", &["glUniform4fARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform4fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform4fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform4fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniform4fv",
                &["glUniform4fvARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform4i {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform4i.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform4i =
                FnPtr::new(metaloadfn(&mut loadfn, "glUniform4i", &["glUniform4iARB"]))
        }
    }
}

#[allow(non_snake_case)]
pub mod Uniform4iv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Uniform4iv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::Uniform4iv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniform4iv",
                &["glUniform4ivARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod UniformMatrix2fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::UniformMatrix2fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::UniformMatrix2fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniformMatrix2fv",
                &["glUniformMatrix2fvARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod UniformMatrix3fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::UniformMatrix3fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::UniformMatrix3fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniformMatrix3fv",
                &["glUniformMatrix3fvARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod UniformMatrix4fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::UniformMatrix4fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::UniformMatrix4fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUniformMatrix4fv",
                &["glUniformMatrix4fvARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod UseProgram {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::UseProgram.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::UseProgram = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glUseProgram",
                &["glUseProgramObjectARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod ValidateProgram {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::ValidateProgram.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::ValidateProgram = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glValidateProgram",
                &["glValidateProgramARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttrib1f {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttrib1f.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttrib1f = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttrib1f",
                &["glVertexAttrib1fARB", "glVertexAttrib1fNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttrib1fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttrib1fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttrib1fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttrib1fv",
                &["glVertexAttrib1fvARB", "glVertexAttrib1fvNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttrib2f {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttrib2f.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttrib2f = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttrib2f",
                &["glVertexAttrib2fARB", "glVertexAttrib2fNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttrib2fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttrib2fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttrib2fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttrib2fv",
                &["glVertexAttrib2fvARB", "glVertexAttrib2fvNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttrib3f {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttrib3f.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttrib3f = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttrib3f",
                &["glVertexAttrib3fARB", "glVertexAttrib3fNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttrib3fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttrib3fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttrib3fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttrib3fv",
                &["glVertexAttrib3fvARB", "glVertexAttrib3fvNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttrib4f {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttrib4f.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttrib4f = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttrib4f",
                &["glVertexAttrib4fARB", "glVertexAttrib4fNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttrib4fv {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttrib4fv.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttrib4fv = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttrib4fv",
                &["glVertexAttrib4fvARB", "glVertexAttrib4fvNV"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod VertexAttribPointer {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::VertexAttribPointer.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe {
            storage::VertexAttribPointer = FnPtr::new(metaloadfn(
                &mut loadfn,
                "glVertexAttribPointer",
                &["glVertexAttribPointerARB"],
            ))
        }
    }
}

#[allow(non_snake_case)]
pub mod Viewport {
    use super::FnPtr;
    use super::__gl_imports::raw;
    use super::{metaloadfn, storage};

    #[inline]
    #[allow(dead_code)]
    pub fn is_loaded() -> bool {
        unsafe { storage::Viewport.is_loaded }
    }

    #[allow(dead_code)]
    pub fn load_with<F>(mut loadfn: F)
    where
        F: FnMut(&'static str) -> *const raw::c_void,
    {
        unsafe { storage::Viewport = FnPtr::new(metaloadfn(&mut loadfn, "glViewport", &[])) }
    }
}

#[inline(never)]
fn missing_fn_panic() -> ! {
    panic!("gles2 function was not loaded")
}

/// Load each OpenGL symbol using a custom load function. This allows for the
/// use of functions like `glfwGetProcAddress` or `SDL_GL_GetProcAddress`.
/// ~~~ignore
/// gl::load_with(|s| glfw.get_proc_address(s));
/// ~~~
#[allow(dead_code)]
pub fn load_with<F>(mut loadfn: F)
where
    F: FnMut(&'static str) -> *const __gl_imports::raw::c_void,
{
    #[inline(never)]
    fn inner(loadfn: &mut dyn FnMut(&'static str) -> *const __gl_imports::raw::c_void) {
        ActiveTexture::load_with(&mut *loadfn);
        AttachShader::load_with(&mut *loadfn);
        BindAttribLocation::load_with(&mut *loadfn);
        BindBuffer::load_with(&mut *loadfn);
        BindFramebuffer::load_with(&mut *loadfn);
        BindRenderbuffer::load_with(&mut *loadfn);
        BindTexture::load_with(&mut *loadfn);
        BlendColor::load_with(&mut *loadfn);
        BlendEquation::load_with(&mut *loadfn);
        BlendEquationSeparate::load_with(&mut *loadfn);
        BlendFunc::load_with(&mut *loadfn);
        BlendFuncSeparate::load_with(&mut *loadfn);
        BufferData::load_with(&mut *loadfn);
        BufferSubData::load_with(&mut *loadfn);
        CheckFramebufferStatus::load_with(&mut *loadfn);
        Clear::load_with(&mut *loadfn);
        ClearColor::load_with(&mut *loadfn);
        ClearDepthf::load_with(&mut *loadfn);
        ClearStencil::load_with(&mut *loadfn);
        ColorMask::load_with(&mut *loadfn);
        CompileShader::load_with(&mut *loadfn);
        CompressedTexImage2D::load_with(&mut *loadfn);
        CompressedTexSubImage2D::load_with(&mut *loadfn);
        CopyTexImage2D::load_with(&mut *loadfn);
        CopyTexSubImage2D::load_with(&mut *loadfn);
        CreateProgram::load_with(&mut *loadfn);
        CreateShader::load_with(&mut *loadfn);
        CullFace::load_with(&mut *loadfn);
        DeleteBuffers::load_with(&mut *loadfn);
        DeleteFramebuffers::load_with(&mut *loadfn);
        DeleteProgram::load_with(&mut *loadfn);
        DeleteRenderbuffers::load_with(&mut *loadfn);
        DeleteShader::load_with(&mut *loadfn);
        DeleteTextures::load_with(&mut *loadfn);
        DepthFunc::load_with(&mut *loadfn);
        DepthMask::load_with(&mut *loadfn);
        DepthRangef::load_with(&mut *loadfn);
        DetachShader::load_with(&mut *loadfn);
        Disable::load_with(&mut *loadfn);
        DisableVertexAttribArray::load_with(&mut *loadfn);
        DrawArrays::load_with(&mut *loadfn);
        DrawElements::load_with(&mut *loadfn);
        Enable::load_with(&mut *loadfn);
        EnableVertexAttribArray::load_with(&mut *loadfn);
        Finish::load_with(&mut *loadfn);
        Flush::load_with(&mut *loadfn);
        FramebufferRenderbuffer::load_with(&mut *loadfn);
        FramebufferTexture2D::load_with(&mut *loadfn);
        FrontFace::load_with(&mut *loadfn);
        GenBuffers::load_with(&mut *loadfn);
        GenFramebuffers::load_with(&mut *loadfn);
        GenRenderbuffers::load_with(&mut *loadfn);
        GenTextures::load_with(&mut *loadfn);
        GenerateMipmap::load_with(&mut *loadfn);
        GetActiveAttrib::load_with(&mut *loadfn);
        GetActiveUniform::load_with(&mut *loadfn);
        GetAttachedShaders::load_with(&mut *loadfn);
        GetAttribLocation::load_with(&mut *loadfn);
        GetBooleanv::load_with(&mut *loadfn);
        GetBufferParameteriv::load_with(&mut *loadfn);
        GetError::load_with(&mut *loadfn);
        GetFloatv::load_with(&mut *loadfn);
        GetFramebufferAttachmentParameteriv::load_with(&mut *loadfn);
        GetIntegerv::load_with(&mut *loadfn);
        GetProgramInfoLog::load_with(&mut *loadfn);
        GetProgramiv::load_with(&mut *loadfn);
        GetRenderbufferParameteriv::load_with(&mut *loadfn);
        GetShaderInfoLog::load_with(&mut *loadfn);
        GetShaderPrecisionFormat::load_with(&mut *loadfn);
        GetShaderSource::load_with(&mut *loadfn);
        GetShaderiv::load_with(&mut *loadfn);
        GetString::load_with(&mut *loadfn);
        GetTexParameterfv::load_with(&mut *loadfn);
        GetTexParameteriv::load_with(&mut *loadfn);
        GetUniformLocation::load_with(&mut *loadfn);
        GetUniformfv::load_with(&mut *loadfn);
        GetUniformiv::load_with(&mut *loadfn);
        GetVertexAttribPointerv::load_with(&mut *loadfn);
        GetVertexAttribfv::load_with(&mut *loadfn);
        GetVertexAttribiv::load_with(&mut *loadfn);
        Hint::load_with(&mut *loadfn);
        IsBuffer::load_with(&mut *loadfn);
        IsEnabled::load_with(&mut *loadfn);
        IsFramebuffer::load_with(&mut *loadfn);
        IsProgram::load_with(&mut *loadfn);
        IsRenderbuffer::load_with(&mut *loadfn);
        IsShader::load_with(&mut *loadfn);
        IsTexture::load_with(&mut *loadfn);
        LineWidth::load_with(&mut *loadfn);
        LinkProgram::load_with(&mut *loadfn);
        PixelStorei::load_with(&mut *loadfn);
        PolygonOffset::load_with(&mut *loadfn);
        ReadPixels::load_with(&mut *loadfn);
        ReleaseShaderCompiler::load_with(&mut *loadfn);
        RenderbufferStorage::load_with(&mut *loadfn);
        SampleCoverage::load_with(&mut *loadfn);
        Scissor::load_with(&mut *loadfn);
        ShaderBinary::load_with(&mut *loadfn);
        ShaderSource::load_with(&mut *loadfn);
        StencilFunc::load_with(&mut *loadfn);
        StencilFuncSeparate::load_with(&mut *loadfn);
        StencilMask::load_with(&mut *loadfn);
        StencilMaskSeparate::load_with(&mut *loadfn);
        StencilOp::load_with(&mut *loadfn);
        StencilOpSeparate::load_with(&mut *loadfn);
        TexImage2D::load_with(&mut *loadfn);
        TexParameterf::load_with(&mut *loadfn);
        TexParameterfv::load_with(&mut *loadfn);
        TexParameteri::load_with(&mut *loadfn);
        TexParameteriv::load_with(&mut *loadfn);
        TexSubImage2D::load_with(&mut *loadfn);
        Uniform1f::load_with(&mut *loadfn);
        Uniform1fv::load_with(&mut *loadfn);
        Uniform1i::load_with(&mut *loadfn);
        Uniform1iv::load_with(&mut *loadfn);
        Uniform2f::load_with(&mut *loadfn);
        Uniform2fv::load_with(&mut *loadfn);
        Uniform2i::load_with(&mut *loadfn);
        Uniform2iv::load_with(&mut *loadfn);
        Uniform3f::load_with(&mut *loadfn);
        Uniform3fv::load_with(&mut *loadfn);
        Uniform3i::load_with(&mut *loadfn);
        Uniform3iv::load_with(&mut *loadfn);
        Uniform4f::load_with(&mut *loadfn);
        Uniform4fv::load_with(&mut *loadfn);
        Uniform4i::load_with(&mut *loadfn);
        Uniform4iv::load_with(&mut *loadfn);
        UniformMatrix2fv::load_with(&mut *loadfn);
        UniformMatrix3fv::load_with(&mut *loadfn);
        UniformMatrix4fv::load_with(&mut *loadfn);
        UseProgram::load_with(&mut *loadfn);
        ValidateProgram::load_with(&mut *loadfn);
        VertexAttrib1f::load_with(&mut *loadfn);
        VertexAttrib1fv::load_with(&mut *loadfn);
        VertexAttrib2f::load_with(&mut *loadfn);
        VertexAttrib2fv::load_with(&mut *loadfn);
        VertexAttrib3f::load_with(&mut *loadfn);
        VertexAttrib3fv::load_with(&mut *loadfn);
        VertexAttrib4f::load_with(&mut *loadfn);
        VertexAttrib4fv::load_with(&mut *loadfn);
        VertexAttribPointer::load_with(&mut *loadfn);
        Viewport::load_with(&mut *loadfn);
    }

    inner(&mut loadfn)
}
