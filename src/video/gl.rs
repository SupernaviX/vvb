#![allow(clippy::all)]

#[cfg(target_os = "android")]
#[link(name = "GLESv2")]
extern "C" {}

include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

#[warn(clippy::all)]
pub mod utils;
