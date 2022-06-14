#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

use jni::sys::{jobject, JavaVM};
use std::os::raw::*;

mod sys {
    use std::os::raw::{c_float, c_int, c_ulonglong};

    #[repr(C)]
    pub struct CardboardLensDistortion {
        private: [u8; 0],
    }
    #[repr(C)]
    pub struct CardboardDistortionRenderer {
        private: [u8; 0],
    }

    #[repr(C)]
    pub struct CardboardEyeTextureDescription {
        pub texture: c_ulonglong,
        pub left_u: c_float,
        pub right_u: c_float,
        pub top_v: c_float,
        pub bottom_v: c_float,
    }

    #[repr(C)]
    pub struct CardboardMesh {
        pub indices: *const c_int,
        pub n_indices: c_int,
        pub vertices: *const c_float,
        pub uvs: *const c_float,
        pub n_vertices: c_int,
    }

    #[repr(C)]
    pub enum CardboardEye {
        kLeft = 0,
        kRight = 1,
    }
}

pub use sys::CardboardEyeTextureDescription as TextureDescription;
pub use sys::{CardboardEye, CardboardMesh};

#[cfg(target_os = "android")]
#[link(name = "GfxPluginCardboard")]
extern "C" {
    #[link_name = "Cardboard_initializeAndroid"]
    pub fn Cardboard_initializeAndroid(vm: *const JavaVM, context: jobject);

    #[link_name = "CardboardLensDistortion_create"]
    pub fn CardboardLensDistortion_create(
        encoded_device_params: *const c_uchar,
        size: c_int,
        display_width: c_int,
        display_height: c_int,
    ) -> *mut sys::CardboardLensDistortion;
    #[link_name = "CardboardLensDistortion_destroy"]
    pub fn CardboardLensDistortion_destroy(lens_distortion: *mut sys::CardboardLensDistortion);
    #[link_name = "CardboardLensDistortion_getDistortionMesh"]
    pub fn CardboardLensDistortion_getDistortionMesh(
        lens_distortion: *mut sys::CardboardLensDistortion,
        eye: sys::CardboardEye,
        mesh: *mut sys::CardboardMesh,
    );

    #[link_name = "CardboardOpenGlEs2DistortionRenderer_create"]
    pub fn CardboardOpenGlEs2DistortionRenderer_create() -> *mut sys::CardboardDistortionRenderer;
    #[link_name = "CardboardDistortionRenderer_destroy"]
    pub fn CardboardDistortionRenderer_destroy(renderer: *mut sys::CardboardDistortionRenderer);
    #[link_name = "CardboardDistortionRenderer_renderEyeToDisplay"]
    pub fn CardboardDistortionRenderer_renderEyeToDisplay(
        renderer: *mut sys::CardboardDistortionRenderer,
        target: c_ulonglong,
        x: c_int,
        y: c_int,
        width: c_int,
        height: c_int,
        left_eye: *const TextureDescription,
        right_eye: *const TextureDescription,
    );
    #[link_name = "CardboardDistortionRenderer_setMesh"]
    pub fn CardboardDistortionRenderer_setMesh(
        renderer: *mut sys::CardboardDistortionRenderer,
        mesh: *const CardboardMesh,
        eye: CardboardEye,
    );

    #[link_name = "CardboardQrCode_destroy"]
    pub fn CardboardQrCode_destroy(encoded_device_params: *const c_uchar);
    #[link_name = "CardboardQrCode_getSavedDeviceParams"]
    pub fn CardboardQrCode_getSavedDeviceParams(
        encoded_device_params: *mut *const c_uchar,
        size: *mut c_int,
    );
    #[link_name = "CardboardQrCode_scanQrCodeAndSaveDeviceParams"]
    pub fn CardboardQrCode_scanQrCodeAndSaveDeviceParams();
}

pub struct Cardboard;
impl Cardboard {
    pub fn initialize(vm: *const JavaVM, context: jobject) {
        #[cfg(target_os = "android")]
        unsafe {
            Cardboard_initializeAndroid(vm, context);
        }
    }
}

pub struct LensDistortion(*mut sys::CardboardLensDistortion);
impl LensDistortion {
    pub fn create(encoded_device_params: &DeviceParams, width: i32, height: i32) -> LensDistortion {
        #[cfg(target_os = "android")]
        let raw = unsafe {
            CardboardLensDistortion_create(
                encoded_device_params.buffer,
                encoded_device_params.size,
                width,
                height,
            )
        };
        #[cfg(not(target_os = "android"))]
        let raw = std::ptr::null_mut();
        LensDistortion(raw)
    }
    #[allow(unused_mut, clippy::let_and_return)]
    pub fn get_distortion_mesh(&self, eye: CardboardEye) -> CardboardMesh {
        let mut mesh = CardboardMesh {
            indices: std::ptr::null(),
            n_indices: 0,
            vertices: std::ptr::null(),
            uvs: std::ptr::null(),
            n_vertices: 0,
        };
        #[cfg(target_os = "android")]
        unsafe {
            CardboardLensDistortion_getDistortionMesh(self.0, eye, &mut mesh);
        }
        mesh
    }
}
unsafe impl Send for LensDistortion {}
unsafe impl Sync for LensDistortion {}
impl Drop for LensDistortion {
    fn drop(&mut self) {
        #[cfg(target_os = "android")]
        unsafe {
            CardboardLensDistortion_destroy(self.0);
        }
    }
}

pub struct DistortionRenderer(*mut sys::CardboardDistortionRenderer);
impl DistortionRenderer {
    pub fn create() -> DistortionRenderer {
        #[cfg(target_os = "android")]
        let raw = unsafe { CardboardOpenGlEs2DistortionRenderer_create() };
        #[cfg(not(target_os = "android"))]
        let raw = std::ptr::null_mut();
        DistortionRenderer(raw)
    }

    pub fn set_mesh(&mut self, mesh: &CardboardMesh, eye: CardboardEye) {
        #[cfg(target_os = "android")]
        unsafe {
            CardboardDistortionRenderer_setMesh(self.0, mesh, eye);
        }
    }

    pub fn render_eye_to_display(
        &self,
        target: u64,
        position: (i32, i32),
        size: (i32, i32),
        left_eye: &TextureDescription,
        right_eye: &TextureDescription,
    ) {
        #[cfg(target_os = "android")]
        unsafe {
            CardboardDistortionRenderer_renderEyeToDisplay(
                self.0, target, position.0, position.1, size.0, size.1, left_eye, right_eye,
            );
        }
    }
}
unsafe impl Send for DistortionRenderer {}
unsafe impl Sync for DistortionRenderer {}
impl Drop for DistortionRenderer {
    fn drop(&mut self) {
        #[cfg(target_os = "android")]
        unsafe {
            CardboardDistortionRenderer_destroy(self.0);
        }
    }
}

pub struct QrCode;
impl QrCode {
    #[allow(unused_mut, clippy::wrong_self_convention)]
    pub fn get_saved_device_params() -> Option<DeviceParams> {
        let mut buffer = std::ptr::null();
        let mut size: c_int = 0;
        #[cfg(target_os = "android")]
        unsafe {
            CardboardQrCode_getSavedDeviceParams(&mut buffer, &mut size);
        }
        match size {
            0 => None,
            _ => Some(DeviceParams { buffer, size }),
        }
    }

    pub fn scan_qr_code_and_save_device_params() {
        #[cfg(target_os = "android")]
        unsafe {
            CardboardQrCode_scanQrCodeAndSaveDeviceParams();
        }
    }
}

#[derive(Debug)]
pub struct DeviceParams {
    buffer: *const c_uchar,
    size: c_int,
}
unsafe impl Send for DeviceParams {}
unsafe impl Sync for DeviceParams {}
impl Drop for DeviceParams {
    fn drop(&mut self) {
        #[cfg(target_os = "android")]
        unsafe {
            CardboardQrCode_destroy(self.buffer);
        }
    }
}
