#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_char, c_float, c_int, c_void};

// Error types
pub type RDError = c_int;

pub const RDEOK: RDError = 0;
pub const RDENOMEM: RDError = 1;
pub const RDEINTERNAL: RDError = 2;
pub const RDEINVAL: RDError = 3;
pub const RDEUNSUPP: RDError = 4;
pub const RDETOOBIG: RDError = 5;
pub const RDEPARAM: RDError = 6;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RDResolution {
    pub index: usize,
    pub confidence: c_float,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RDMethod {
    pub name: *const c_char,
    pub func: Option<unsafe extern "C" fn()>,
    pub threshold: c_float,
}

// Opaque types
#[repr(C)]
pub struct RDParameters {
    _private: [u8; 0],
}

#[repr(C)]
pub struct RDAnalysis {
    _private: [u8; 0],
}

#[repr(C)]
pub struct RDImage {
    _private: [u8; 0],
}

extern "C" {
    pub fn resdet_libversion() -> *const c_char;

    pub fn resdet_error_str(error: RDError) -> *const c_char;

    pub fn resdet_methods() -> *mut RDMethod;

    pub fn resdet_get_method(name: *const c_char) -> *mut RDMethod;

    pub fn resdet_default_range() -> usize;

    pub fn resdet_alloc_default_parameters() -> *mut RDParameters;

    pub fn resdet_parameters_set_range(params: *mut RDParameters, range: usize) -> RDError;

    pub fn resdet_parameters_set_threshold(
        params: *mut RDParameters,
        threshold: c_float,
    ) -> RDError;

    pub fn resdet_open_image(
        filename: *const c_char,
        type_: *const c_char,
        width: *mut usize,
        height: *mut usize,
        imagebuf: *mut *mut c_float,
        error: *mut RDError,
    ) -> *mut RDImage;

    pub fn resdet_read_image_frame(
        image: *mut RDImage,
        image_buf: *mut c_float,
        error: *mut RDError,
    ) -> bool;

    pub fn resdet_seek_frame(
        image: *mut RDImage,
        offset: u64,
        progress: Option<unsafe extern "C" fn(ctx: *mut c_void, frameno: u64)>,
        progress_ctx: *mut c_void,
        error: *mut RDError,
    ) -> bool;

    pub fn resdet_close_image(image: *mut RDImage);

    pub fn resdet_read_image(
        filename: *const c_char,
        filetype: *const c_char,
        image: *mut *mut c_float,
        nimages: *mut usize,
        width: *mut usize,
        height: *mut usize,
    ) -> RDError;

    pub fn resdet_create_analysis(
        method: *mut RDMethod,
        width: usize,
        height: usize,
        params: *const RDParameters,
        error: *mut RDError,
    ) -> *mut RDAnalysis;

    pub fn resdet_analyze_image(analysis: *mut RDAnalysis, image: *mut c_float) -> RDError;

    pub fn resdet_analysis_results(
        analysis: *mut RDAnalysis,
        resw: *mut *mut RDResolution,
        countw: *mut usize,
        resh: *mut *mut RDResolution,
        counth: *mut usize,
    ) -> RDError;

    pub fn resdet_destroy_analysis(analysis: *mut RDAnalysis);

    pub fn resdetect(
        image: *mut c_float,
        nimages: usize,
        width: usize,
        height: usize,
        resw: *mut *mut RDResolution,
        countw: *mut usize,
        resh: *mut *mut RDResolution,
        counth: *mut usize,
        method: *mut RDMethod,
        params: *const RDParameters,
    ) -> RDError;

    pub fn resdetect_file(
        filename: *const c_char,
        filetype: *const c_char,
        resw: *mut *mut RDResolution,
        countw: *mut usize,
        resh: *mut *mut RDResolution,
        counth: *mut usize,
        method: *mut RDMethod,
        params: *const RDParameters,
    ) -> RDError;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        unsafe {
            let version = resdet_libversion();
            assert!(!version.is_null());
        }
    }
}
