use std::sync::atomic::{AtomicUsize, Ordering};

use pyo3::{
    exceptions::{PyRuntimeError, PyValueError},
    prelude::*,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[pymodule(gil_used = false)]
fn _resdet(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Metadata
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    m.add_class::<Analysis>()?;
    m.add_class::<DetectedResolution>()?;
    m.add_class::<DetectionResult>()?;
    m.add_function(wrap_pyfunction!(lib_version, m)?)?;
    m.add_function(wrap_pyfunction!(normalize_image_gray, m)?)?;

    Ok(())
}

#[pyclass]
struct Analysis {
    range: AtomicUsize,
    threshold: Option<f32>,
}

#[pymethods]
impl Analysis {
    #[new]
    fn new() -> Self {
        Analysis {
            range: AtomicUsize::new(resdet::default_range()),
            threshold: None,
        }
    }

    #[getter]
    fn range(&self) -> usize {
        self.range.load(Ordering::Relaxed)
    }

    #[setter]
    fn set_range(&self, range: usize) {
        self.range.store(range, Ordering::Relaxed);
    }

    #[getter]
    fn threshold(&self) -> Option<f32> {
        self.threshold
    }

    #[setter]
    fn set_threshold(&mut self, threshold: Option<f32>) {
        // check that threshold is between 0.0 and 1.0 if Some
        self.threshold = threshold;
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!(
            "Analysis(range={}, threshold={:?})",
            self.range(),
            self.threshold
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        self.__repr__()
    }

    fn analyze(
        &self,
        frame_data: Vec<f32>,
        width: usize,
        height: usize,
    ) -> PyResult<DetectionResult> {
        let mut default_params = resdet::Parameters::new().map_err(transform_error)?;

        if let Some(threshold) = self.threshold {
            // if threshold not in 0.0..=1.0, return error
            if !is_norm_value(threshold) {
                return Err(PyValueError::new_err(
                    "threshold must be between 0.0 and 1.0",
                ));
            }

            default_params
                .set_threshold(threshold)
                .map_err(transform_error)?;
        }

        validate_frame_data(&frame_data, width, height)?;

        default_params
            .set_range(self.range())
            .map_err(transform_error)?;

        let detected = resdet::detect(&frame_data, width, height, None, Some(&default_params))
            .map_err(transform_error)?;

        Ok(DetectionResult::from(detected))
    }
}

#[pyclass(frozen)]
#[derive(Clone)]
pub struct DetectedResolution {
    #[pyo3(get)]
    size: usize,
    #[pyo3(get)]
    confidence: f32,
}

#[pymethods]
impl DetectedResolution {
    fn __repr__(&self) -> String {
        format!(
            "DetectedResolution(size={}, confidence={:.4})",
            self.size, self.confidence
        )
    }
}

#[pyclass(frozen)]
pub struct DetectionResult {
    widths: Vec<DetectedResolution>,
    heights: Vec<DetectedResolution>,
}

#[pymethods]
impl DetectionResult {
    #[getter]
    fn widths(&self) -> Vec<DetectedResolution> {
        self.widths.clone()
    }

    #[getter]
    fn heights(&self) -> Vec<DetectedResolution> {
        self.heights.clone()
    }

    fn best_width(&self) -> Option<DetectedResolution> {
        self.widths.first().cloned()
    }

    fn best_height(&self) -> Option<DetectedResolution> {
        self.heights.first().cloned()
    }

    fn __repr__(&self) -> PyResult<String> {
        let widths = self
            .widths
            .iter()
            .map(|r| r.__repr__())
            .collect::<Vec<String>>()
            .join(", ");
        let heights = self
            .heights
            .iter()
            .map(|r| r.__repr__())
            .collect::<Vec<String>>()
            .join(", ");

        Ok(format!(
            "DetectionResult(widths=[{}], heights=[{}])",
            widths, heights
        ))
    }
}

impl From<resdet::DetectionResult> for DetectionResult {
    fn from(value: resdet::DetectionResult) -> Self {
        DetectionResult {
            widths: value
                .width_resolutions
                .into_iter()
                .map(|r| DetectedResolution {
                    size: r.index,
                    confidence: r.confidence,
                })
                .collect(),
            heights: value
                .height_resolutions
                .into_iter()
                .map(|r| DetectedResolution {
                    size: r.index,
                    confidence: r.confidence,
                })
                .collect(),
        }
    }
}

#[pyfunction]
fn lib_version() -> PyResult<String> {
    resdet::lib_version().map_err(transform_error)
}

#[pyfunction]
fn normalize_image_gray(image: Vec<u8>) -> PyResult<Vec<f32>> {
    if image.is_empty() {
        return Err(PyValueError::new_err("Input image data is empty"));
    }

    // do fast normalization
    let normalized: Vec<f32> = image
        .par_iter()
        .map(|&v| (v as f32) / (u8::MAX as f32))
        .collect();

    Ok(normalized)
}

fn transform_error(err: resdet::ResdetError) -> PyErr {
    let err_str = err.to_string();
    PyRuntimeError::new_err(err_str)
}

fn validate_frame_data(frame_data: &[f32], width: usize, height: usize) -> PyResult<()> {
    let expected_size = width
        .checked_mul(height)
        .ok_or_else(|| PyValueError::new_err("Width * Height overflowed"))?;

    if frame_data.len() != expected_size {
        return Err(PyValueError::new_err(format!(
            "Frame data length ({}) does not match expected size (width: {}, height: {}, expected: {})",
            frame_data.len(),
            width,
            height,
            expected_size
        )));
    }

    // Check if all values are in 0.0..=1.0
    if !frame_data.par_iter().all(|&v| is_norm_value(v)) {
        return Err(PyValueError::new_err(
            "Frame data contains values outside the range [0.0, 1.0]",
        ));
    }

    Ok(())
}

fn is_norm_value(v: f32) -> bool {
    (0.0..=1.0).contains(&v)
}
