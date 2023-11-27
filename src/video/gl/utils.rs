use super::types::GLvoid;
use super::{GetError, NO_ERROR};
use anyhow::Result;

pub fn clear_errors() {
    let mut error = unsafe { GetError() };
    while error != NO_ERROR {
        log::warn!("Ignoring GL error 0x{:04x}", error);
        error = unsafe { GetError() };
    }
}

pub fn check_error(action: &str) -> Result<()> {
    let error = unsafe { GetError() };
    match error {
        NO_ERROR => Ok(()),
        _ => Err(anyhow::anyhow!(
            "OpenGL threw code 0x{:04X} while trying to {}!",
            error,
            action
        )),
    }
}

pub fn temp_array<T: Copy + Default, F: FnOnce(*mut T)>(cb: F) -> T {
    let mut tmp_array: [T; 1] = Default::default();
    cb(tmp_array.as_mut_ptr());
    tmp_array[0]
}

pub trait AsVoidptr {
    fn as_voidptr(&self) -> *const GLvoid;
    fn as_mut_voidptr(&mut self) -> *mut GLvoid;
}

impl<T> AsVoidptr for [T] {
    fn as_voidptr(&self) -> *const GLvoid {
        self.as_ptr() as *const GLvoid
    }

    fn as_mut_voidptr(&mut self) -> *mut GLvoid {
        self.as_mut_ptr() as *mut GLvoid
    }
}
