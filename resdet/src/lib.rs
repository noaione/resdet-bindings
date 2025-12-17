use std::ffi::{CStr, CString};
use std::ptr;
use std::slice;
use thiserror::Error;

pub use resdet_sys::{RDMethod, RDResolution};

#[derive(Error, Debug)]
pub enum ResdetError {
    #[error("Out of memory")]
    OutOfMemory,
    #[error("Internal error")]
    Internal,
    #[error("Invalid argument")]
    Invalid,
    #[error("Unsupported operation")]
    Unsupported,
    #[error("Image too big")]
    TooBig,
    #[error("Invalid parameter")]
    Param,
    #[error("Unknown error: {0}")]
    Unknown(i32),
    #[error("Null pointer")]
    NullPointer,
    #[error("UTF-8 conversion error")]
    Utf8Error,
}

impl From<i32> for ResdetError {
    fn from(error: i32) -> Self {
        match error {
            resdet_sys::RDENOMEM => ResdetError::OutOfMemory,
            resdet_sys::RDEINTERNAL => ResdetError::Internal,
            resdet_sys::RDEINVAL => ResdetError::Invalid,
            resdet_sys::RDEUNSUPP => ResdetError::Unsupported,
            resdet_sys::RDETOOBIG => ResdetError::TooBig,
            resdet_sys::RDEPARAM => ResdetError::Param,
            _ => ResdetError::Unknown(error),
        }
    }
}

pub type Result<T> = std::result::Result<T, ResdetError>;

/// Get the libresdet version string
pub fn lib_version() -> Result<String> {
    unsafe {
        let version = resdet_sys::resdet_libversion();
        if version.is_null() {
            return Err(ResdetError::NullPointer);
        }
        CStr::from_ptr(version)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|_| ResdetError::Utf8Error)
    }
}

/// Get the default detection range
pub fn default_range() -> usize {
    unsafe { resdet_sys::resdet_default_range() }
}

/// Parameters for resolution detection
pub struct Parameters {
    ptr: *mut resdet_sys::RDParameters,
}

// SAFETY: Parameters contains no thread-local state and can be safely sent between threads.
unsafe impl Send for Parameters {}

impl Parameters {
    /// Create parameters with default values
    pub fn new() -> Result<Self> {
        unsafe {
            let ptr = resdet_sys::resdet_alloc_default_parameters();
            if ptr.is_null() {
                return Err(ResdetError::OutOfMemory);
            }
            Ok(Parameters { ptr })
        }
    }

    /// Set the detection range
    pub fn set_range(&mut self, range: usize) -> Result<()> {
        unsafe {
            let error = resdet_sys::resdet_parameters_set_range(self.ptr, range);
            if error != resdet_sys::RDEOK {
                return Err(error.into());
            }
            Ok(())
        }
    }

    /// Set the detection threshold
    pub fn set_threshold(&mut self, threshold: f32) -> Result<()> {
        unsafe {
            let error = resdet_sys::resdet_parameters_set_threshold(self.ptr, threshold);
            if error != resdet_sys::RDEOK {
                return Err(error.into());
            }
            Ok(())
        }
    }

    pub(crate) fn as_ptr(&self) -> *const resdet_sys::RDParameters {
        self.ptr
    }
}

impl Drop for Parameters {
    fn drop(&mut self) {
        // Parameters are freed by the analysis or detection functions
        // We don't need to manually free them
    }
}

/// Detection method
pub struct Method {
    ptr: *mut resdet_sys::RDMethod,
}

// SAFETY: Method points to static read-only data in the C library.
// The methods array is immutable global state that is safe to access
// from multiple threads concurrently.
unsafe impl Send for Method {}
unsafe impl Sync for Method {}

impl Method {
    /// Get a method by name
    pub fn by_name(name: &str) -> Result<Self> {
        let c_name = CString::new(name).map_err(|_| ResdetError::Invalid)?;
        unsafe {
            let ptr = resdet_sys::resdet_get_method(c_name.as_ptr());
            if ptr.is_null() {
                return Err(ResdetError::Invalid);
            }
            Ok(Method { ptr })
        }
    }

    /// Get all available methods
    pub fn all() -> Vec<Self> {
        unsafe {
            let mut methods = Vec::new();
            let mut ptr = resdet_sys::resdet_methods();

            while !ptr.is_null() && !(*ptr).name.is_null() {
                methods.push(Method { ptr });
                ptr = ptr.add(1);
            }

            methods
        }
    }

    /// Get the method name
    pub fn name(&self) -> Result<String> {
        unsafe {
            if (*self.ptr).name.is_null() {
                return Err(ResdetError::NullPointer);
            }
            CStr::from_ptr((*self.ptr).name)
                .to_str()
                .map(|s| s.to_string())
                .map_err(|_| ResdetError::Utf8Error)
        }
    }

    /// Get the method threshold
    pub fn threshold(&self) -> f32 {
        unsafe { (*self.ptr).threshold }
    }

    pub(crate) fn as_ptr(&self) -> *mut resdet_sys::RDMethod {
        self.ptr
    }
}

/// Result of resolution detection
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub width_resolutions: Vec<RDResolution>,
    pub height_resolutions: Vec<RDResolution>,
}

impl DetectionResult {
    /// Get the best guess for width
    pub fn best_width(&self) -> Option<&RDResolution> {
        self.width_resolutions.first()
    }

    /// Get the best guess for height
    pub fn best_height(&self) -> Option<&RDResolution> {
        self.height_resolutions.first()
    }
}

/// Detect resolution from raw image data
///
/// # Arguments
/// * `image` - Raw image data as f32 values (typically 0.0-1.0 range)
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `method` - Detection method to use (None for default)
/// * `params` - Detection parameters (None for default)
pub fn detect(
    image: &[f32],
    width: usize,
    height: usize,
    method: Option<&Method>,
    params: Option<&Parameters>,
) -> Result<DetectionResult> {
    if image.len() != width * height {
        return Err(ResdetError::Invalid);
    }

    unsafe {
        let mut resw: *mut RDResolution = ptr::null_mut();
        let mut countw: usize = 0;
        let mut resh: *mut RDResolution = ptr::null_mut();
        let mut counth: usize = 0;

        let method_ptr = method.map(|m| m.as_ptr()).unwrap_or(ptr::null_mut());
        let params_ptr = params.map(|p| p.as_ptr()).unwrap_or(ptr::null());

        let error = resdet_sys::resdetect(
            image.as_ptr() as *mut f32,
            1, // nimages
            width,
            height,
            &mut resw,
            &mut countw,
            &mut resh,
            &mut counth,
            method_ptr,
            params_ptr,
        );

        if error != resdet_sys::RDEOK {
            return Err(error.into());
        }

        let width_resolutions = if !resw.is_null() && countw > 0 {
            slice::from_raw_parts(resw, countw).to_vec()
        } else {
            Vec::new()
        };

        let height_resolutions = if !resh.is_null() && counth > 0 {
            slice::from_raw_parts(resh, counth).to_vec()
        } else {
            Vec::new()
        };

        Ok(DetectionResult {
            width_resolutions,
            height_resolutions,
        })
    }
}

/// Analysis context for processing multiple images
///
/// # Thread Safety
///
/// `Analysis` is `Send` but NOT `Sync`. You can transfer ownership between threads,
/// but you cannot share it across threads (e.g., via `Arc`).
///
/// Each `Analysis` instance maintains internal mutable state for FFT computation
/// and result accumulation. While the C library doesn't use explicit locks,
/// the state is not designed for concurrent access.
pub struct Analysis {
    ptr: *mut resdet_sys::RDAnalysis,
}

// SAFETY: Analysis owns its internal state through the pointer and contains no
// thread-local data. The C library allocates separate buffers for each Analysis
// instance, with no shared mutable state between instances.
// It can be safely moved between threads.
unsafe impl Send for Analysis {}

impl Analysis {
    /// Create a new analysis context
    pub fn new(
        width: usize,
        height: usize,
        method: Option<&Method>,
        params: Option<&Parameters>,
    ) -> Result<Self> {
        unsafe {
            let mut error = resdet_sys::RDEOK;
            let method_ptr = method.map(|m| m.as_ptr()).unwrap_or(ptr::null_mut());
            let params_ptr = params.map(|p| p.as_ptr()).unwrap_or(ptr::null());

            let ptr = resdet_sys::resdet_create_analysis(
                method_ptr, width, height, params_ptr, &mut error,
            );

            if ptr.is_null() {
                return Err(error.into());
            }

            Ok(Analysis { ptr })
        }
    }

    /// Analyze an image
    pub fn analyze(&mut self, image: &[f32]) -> Result<()> {
        unsafe {
            let error = resdet_sys::resdet_analyze_image(self.ptr, image.as_ptr() as *mut f32);

            if error != resdet_sys::RDEOK {
                return Err(error.into());
            }

            Ok(())
        }
    }

    /// Get the detection results
    pub fn results(&self) -> Result<DetectionResult> {
        unsafe {
            let mut resw: *mut RDResolution = ptr::null_mut();
            let mut countw: usize = 0;
            let mut resh: *mut RDResolution = ptr::null_mut();
            let mut counth: usize = 0;

            let error = resdet_sys::resdet_analysis_results(
                self.ptr,
                &mut resw,
                &mut countw,
                &mut resh,
                &mut counth,
            );

            if error != resdet_sys::RDEOK {
                return Err(error.into());
            }

            let width_resolutions = if !resw.is_null() && countw > 0 {
                slice::from_raw_parts(resw, countw).to_vec()
            } else {
                Vec::new()
            };

            let height_resolutions = if !resh.is_null() && counth > 0 {
                slice::from_raw_parts(resh, counth).to_vec()
            } else {
                Vec::new()
            };

            Ok(DetectionResult {
                width_resolutions,
                height_resolutions,
            })
        }
    }
}

impl Drop for Analysis {
    fn drop(&mut self) {
        unsafe {
            resdet_sys::resdet_destroy_analysis(self.ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = lib_version();
        assert!(version.is_ok());
    }

    #[test]
    fn test_default_range() {
        let range = default_range();
        assert!(range > 0);
    }

    #[test]
    fn test_parameters() {
        let mut params = Parameters::new().unwrap();
        assert!(params.set_range(100).is_ok());
        assert!(params.set_threshold(0.5).is_ok());
    }
}
