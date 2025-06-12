mod profiler;
pub mod stack_node;

use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::profiler::Profiler;


#[pymodule]
#[pyo3(name = "profiler")]
fn profiler_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Profiler>()?;
    Ok(())
}