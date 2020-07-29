extern crate gl_generator;
use gl_generator::{Api, Fallbacks, Profile, Registry};
use std::{env, fs::File, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let dest = Path::new(&out_dir);

    let mut file = File::create(&dest.join("gl_bindings.rs")).unwrap();

    Registry::new(
        Api::Gles2,
        (2, 0),
        Profile::Core,
        Fallbacks::All,
        [],
    )
    .write_bindings(gl_generator::StaticGenerator, &mut file)
    .unwrap();
}