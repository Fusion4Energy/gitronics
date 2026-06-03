use pyo3::prelude::*;

use crate::run_cli;

#[pyfunction]
fn run(args: Vec<String>) -> PyResult<()> {
    use pyo3::exceptions::PyRuntimeError;
    // Clap exits the process on --help / bad args; that's acceptable.
    run_cli(args).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Python extension module.
#[pymodule]
fn gitronics(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)
}
