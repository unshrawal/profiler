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

    pub fn profile(&mut self, py: Python, func: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.tic();
        let result = func.call0()?;
        self.toc();
        Ok(result.into())
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
        println!("Function {} took {:.3} seconds", name, elapsed.as_secs_f64());
        Ok(ret)
    }
}