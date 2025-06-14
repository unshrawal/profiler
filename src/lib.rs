pub mod profiler_dir;
use profiler_dir::bindings::PyProfiler;
// mod profiler_impl;
// mod stack_node;

use pyo3::{prelude::*};

#[pymodule]
fn profiler(m: &Bound<'_, PyModule>) -> PyResult<()>{
    m.add_class::<PyProfiler>()?;
    Ok(())
}