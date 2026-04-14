//! Low-level rendering bridge between Rust and Python.
//!
//! This module is intentionally private — use [`Dashboard::render`](crate::Dashboard::render)
//! for the high-level API or [`render_dashboard`] for direct control.
//!
//! The PyO3 serialization logic is split into two sub-modules:
//! - [`chart_config`] — `ChartConfig`, palette, tooltip, axis builders.
//! - [`module`] — `PageModule`, `ColumnFormat`, `FilterConfig` builders.

mod chart_config;
mod module;

use crate::error::ChartError;
use crate::pages::Page;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::ffi::CString;

use self::module::{build_py_filter, build_py_module};

fn build_py_frames<'py>(
    py: Python<'py>,
    frame_data: &[(&str, Vec<u8>)],
) -> PyResult<Bound<'py, PyDict>> {
    let py_frames = PyDict::new(py);
    for (key, bytes) in frame_data {
        py_frames.set_item(*key, PyBytes::new(py, bytes))?;
    }
    Ok(py_frames)
}

fn build_py_nav<'py>(py: Python<'py>, pages: &[Page]) -> PyResult<Bound<'py, PyList>> {
    let py_nav = PyList::empty(py);
    for page in pages {
        let d = PyDict::new(py);
        d.set_item("slug", &page.slug)?;
        d.set_item("label", &page.nav_label)?;
        d.set_item("category", page.category.as_deref().unwrap_or(""))?;
        py_nav.append(d)?;
    }
    Ok(py_nav)
}

fn build_py_page<'py>(py: Python<'py>, page: &Page) -> PyResult<Bound<'py, PyDict>> {
    let p = PyDict::new(py);
    p.set_item("slug", &page.slug)?;
    p.set_item("title", &page.title)?;
    p.set_item("grid_cols", page.grid_cols)?;

    let py_modules = PyList::empty(py);
    for module in &page.modules {
        py_modules.append(build_py_module(py, module)?)?;
    }
    p.set_item("modules", py_modules)?;

    let py_filters = PyList::empty(py);
    for filter in &page.filters {
        py_filters.append(build_py_filter(py, filter)?)?;
    }
    p.set_item("filters", py_filters)?;
    Ok(p)
}

fn build_py_pages<'py>(py: Python<'py>, pages: &[Page]) -> PyResult<Bound<'py, PyList>> {
    let py_pages = PyList::empty(py);
    for page in pages {
        py_pages.append(build_py_page(py, page)?)?;
    }
    Ok(py_pages)
}

/// Render a multi-page Bokeh dashboard to HTML files.
///
/// This is the lower-level rendering function. It takes pre-serialized
/// `DataFrames` (Arrow IPC bytes keyed by name), page definitions, and an
/// output directory. Each page produces one HTML file with inter-page
/// navigation.
///
/// For most use cases, prefer [`Dashboard::render`](crate::Dashboard::render)
/// which handles serialization automatically.
///
/// # Arguments
///
/// * `frame_data` — Slice of `(key, bytes)` pairs where each `bytes` is a
///   Polars `DataFrame` serialized to Arrow IPC format via [`serialize_df`](crate::serialize_df).
/// * `pages` — Slice of [`Page`] definitions describing the modules and
///   filters for each output HTML file.
/// * `output_dir` — Directory path where HTML files will be written. Created
///   automatically if it does not exist.
///
/// # Errors
///
/// Returns [`ChartError::InvalidScript`] if the embedded Python script
/// contains a null byte, or [`ChartError::Python`] if the Python script
/// raises an exception during execution.
pub fn render_dashboard(
    frame_data: &[(&str, Vec<u8>)],
    pages: &[Page],
    output_dir: &str,
    report_title: &str,
    nav_style: &str,
) -> Result<(), ChartError> {
    crate::python_config::configure_vendored_python();

    let python_script = include_str!("../../python/render.py");
    let html_template = include_str!("../../templates/chart.html");

    Python::with_gil(|py| {
        let locals = PyDict::new(py);
        locals.set_item("frames", build_py_frames(py, frame_data)?)?;
        locals.set_item("pages", build_py_pages(py, pages)?)?;
        locals.set_item("nav_links", build_py_nav(py, pages)?)?;
        locals.set_item("html_template", html_template)?;
        locals.set_item("output_dir", output_dir)?;
        locals.set_item("report_title", report_title)?;
        locals.set_item("nav_style", nav_style)?;

        let code = CString::new(python_script).map_err(|_| ChartError::InvalidScript)?;
        py.run(code.as_c_str(), Some(&locals), Some(&locals))?;

        println!(
            "Dashboard generated: {} pages in {}/",
            pages.len(),
            output_dir
        );
        Ok(())
    })
}
