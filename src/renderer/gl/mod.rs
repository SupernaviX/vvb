#[cfg(target_os = "android")]
#[link(name = "GLESv2")]
extern "C" {}

include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

pub mod utils;
