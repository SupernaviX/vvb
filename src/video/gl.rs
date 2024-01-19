#![allow(clippy::all)]
#![allow(unused_imports)]

#[cfg(target_os = "android")]
#[link(name = "GLESv2")]
extern "C" {}

include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));

#[warn(unused_imports)]
#[warn(clippy::all)]
pub mod utils;
