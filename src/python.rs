use std::path::PathBuf;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use crate::build_model::build_model;
use crate::init_logger;
use crate::inspect::inspect_project;
use crate::migrate_model::migrate_model;
use crate::run_cli;

#[pyfunction]
fn run(args: Vec<String>) -> PyResult<()> {
    init_logger();
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
    init_logger();
    build_model(&config_path, &output_path).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Migrate a monolithic MCNP model into a new gitronics project.
///
/// Args:
///     mcnp_input: Path to the monolithic MCNP input file.
///     output_path: Directory where the new project will be created.
#[pyfunction]
fn py_migrate_model(mcnp_input: PathBuf, output_path: PathBuf) -> PyResult<()> {
    init_logger();
    migrate_model(&mcnp_input, &output_path).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Inspect a whole gitronics project and write an interactive project report.
///
/// Args:
///     project_dir: Path to the project directory (containing `configurations/`).
///     output_path: Directory where `project_report.{json,html}` will be written.
#[pyfunction]
fn py_inspect_project(project_dir: PathBuf, output_path: PathBuf) -> PyResult<()> {
    init_logger();
    inspect_project(&project_dir, &output_path).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Python extension module.
#[pymodule]
fn gitronics(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(py_build_model, m)?)?;
    m.add_function(wrap_pyfunction!(py_migrate_model, m)?)?;
    m.add_function(wrap_pyfunction!(py_inspect_project, m)?)
}
