use std::path::PathBuf;

use pyo3::prelude::*;

use crate::build_model::build_model;
use crate::run_cli;

#[pyfunction]
fn run(args: Vec<String>) -> PyResult<()> {
    use pyo3::exceptions::PyRuntimeError;
    // Clap exits the process on --help / bad args; that's acceptable.
    run_cli(args).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Build an MCNP model from a gitronics configuration file.
///
/// Args:
///     config_path: Path to the YAML configuration file.
///     output_path: Directory where the assembled model will be written.
#[pyfunction]
fn py_build_model(config_path: PathBuf, output_path: PathBuf) -> PyResult<()> {
    use pyo3::exceptions::PyRuntimeError;
    build_model(&config_path, &output_path).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Python extension module.
#[pymodule]
fn gitronics(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(py_build_model, m)?)
}
