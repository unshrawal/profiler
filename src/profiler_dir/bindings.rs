use pyo3::{prelude::*, types::{PyDict, PyTuple}};
use crate::profiler_dir::core::Profiler;

#[pyclass]
pub struct PyProfiler {
    inner: Profiler,
    wraps: Option<Py<PyAny>>,
}

#[pymethods]
impl PyProfiler {
    #[new]
    #[pyo3(signature = (wraps = None))]
    pub fn new(wraps: Option<Py<PyAny>>) -> Self{
        Self {
            inner: Profiler::new(),
            wraps
        }
    }

    pub fn tic(&mut self){
        self.inner.tic();
    }

    pub fn toc(&mut self) -> f64{
        self.inner.toc().as_secs_f64()
    }

    pub fn profile(&mut self, py: Python, func: &Bound<'_, PyAny>) -> PyResult<(Py<PyAny>, f64)> {
        self.tic();
        let result = func.call0()?;
        let secs = self.toc();
        Ok((
            result.into(),
            secs
        ))
    }

    #[pyo3(signature = (*args, **kwargs))]
    pub fn __call__(
        &mut self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        self.inner.tic();
        let name = self.wraps
                                .as_ref()
                                .expect("error")
                                .getattr(py, "__name__")?;
        let ret = self.wraps
                                        .as_ref()
                                        .expect("error")
                                        .call(py, args, kwargs)?;
        let elapsed = self.inner.toc();
        println!("Function {} took {:.3} seconds to complete", name, elapsed.as_secs_f64());
        Ok(ret)
    }

    pub fn __enter__(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.tic();
        slf
    }

    pub fn __exit__(
        &mut self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<f64> {
        let duration = self.toc();
        println!("Context took {} seconds to complete", duration);
        Ok(duration)
    }
}