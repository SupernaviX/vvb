#[cfg(target_os = "android")]
#[link(name = "GLESv2")]
extern "C" {}

#[cfg(target_os = "android")]
include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

#[cfg(target_os = "windows")]
mod gl_bindings;
#[cfg(target_os = "windows")]
pub use gl_bindings::*;

pub mod utils;
