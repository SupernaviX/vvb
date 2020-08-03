#![allow(unused_variables)]
#![allow(dead_code)]

use jni::sys::{JavaVM, jobject};
use std::os::raw::*;

mod sys {
    #[repr(C)]
    pub struct CardboardLensDistortion { private: [u8; 0] }
}

#[cfg(target_os = "android")]
#[link(name="cardboard_api")]
extern "C" {
    #[link_name="Cardboard_initializeAndroid"]
    pub fn Cardboard_initializeAndroid(vm: *const JavaVM, context: jobject);

    #[link_name="CardboardLensDistortion_create"]
    pub fn CardboardLensDistortion_create(encoded_device_params: *const c_uchar, size: c_int, display_width: c_int, display_height: c_int) -> *mut sys::CardboardLensDistortion;
    #[link_name="CardboardLensDistortion_destroy"]
    pub fn CardboardLensDistortion_destroy(lens_distortion: *mut sys::CardboardLensDistortion);

    #[link_name="CardboardQrCode_destroy"]
    pub fn CardboardQrCode_destroy(encoded_device_params: *const c_uchar);
    #[link_name="CardboardQrCode_getSavedDeviceParams"]
    pub fn CardboardQrCode_getSavedDeviceParams(encoded_device_params: *mut *const c_uchar, size: *mut c_int);
    #[link_name="CardboardQrCode_scanQrCodeAndSaveDeviceParams"]
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

pub struct CardboardLensDistortion(*mut sys::CardboardLensDistortion);
impl CardboardLensDistortion {
    pub fn create(encoded_device_params: &EncodedDeviceParams, width: i32, height: i32) -> CardboardLensDistortion {
        #[cfg(target_os = "android")]
        let raw = unsafe {
            CardboardLensDistortion_create(encoded_device_params.buffer, encoded_device_params.size, width, height)
        };
        #[cfg(not(target_os = "android"))]
        let raw = 0 as *mut sys::CardboardLensDistortion;
        CardboardLensDistortion(raw)
    }
}
impl Drop for CardboardLensDistortion {
    fn drop(&mut self) {
        #[cfg(target_os = "android")]
        unsafe {
            CardboardLensDistortion_destroy(self.0);
        }
    }
}

pub struct CardboardQrCode;
impl CardboardQrCode {
    #[allow(unused_mut)]
    pub fn get_saved_device_params() -> Option<EncodedDeviceParams> {
        let mut buffer= 0 as *const c_uchar;
        let mut size: c_int = 0;
        #[cfg(target_os = "android")]
        unsafe {
            CardboardQrCode_getSavedDeviceParams(&mut buffer, &mut size);
        }
        match size {
            0 => None,
            _ => Some(EncodedDeviceParams{ buffer, size })
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
pub struct EncodedDeviceParams {
    buffer: *const c_uchar,
    size: c_int
}
impl Drop for EncodedDeviceParams {
    fn drop(&mut self) {
        #[cfg(target_os = "android")]
        unsafe {
            CardboardQrCode_destroy(self.buffer);
        }
    }
}