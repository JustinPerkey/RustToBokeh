//! Low-level rendering bridge between Rust and Python.
//!
//! This module is intentionally private — use [`Dashboard::render`](crate::Dashboard::render)
//! for the high-level API or [`render_dashboard`] for direct control.

use crate::charts::{ChartConfig, FilterConfig};
use crate::error::ChartError;
use crate::modules::{ColumnFormat, PageModule};
use crate::pages::Page;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::ffi::CString;

/// Render a multi-page Bokeh dashboard to HTML files.
///
/// This is the lower-level rendering function. It takes pre-serialized
/// DataFrames (Arrow IPC bytes keyed by name), page definitions, and an
/// output directory. Each page produces one HTML file with inter-page
/// navigation.
///
/// For most use cases, prefer [`Dashboard::render`](crate::Dashboard::render)
/// which handles serialization automatically.
///
/// # Arguments
///
/// * `frame_data` — Slice of `(key, bytes)` pairs where each `bytes` is a
///   Polars DataFrame serialized to Arrow IPC format via [`serialize_df`](crate::serialize_df).
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
) -> Result<(), ChartError> {
    crate::configure_vendored_python();

    let python_script = include_str!("../python/render.py");
    let html_template = include_str!("../templates/chart.html");

    Python::with_gil(|py| {
        // Frames dict: source_key -> Arrow IPC bytes
        let py_frames = PyDict::new(py);
        for (key, bytes) in frame_data {
            py_frames.set_item(*key, PyBytes::new(py, bytes))?;
        }

        // Nav links for all pages
        let py_nav = PyList::empty(py);
        for page in pages {
            let d = PyDict::new(py);
            d.set_item("slug", &page.slug)?;
            d.set_item("label", &page.nav_label)?;
            py_nav.append(d)?;
        }

        // Pages with nested modules
        let py_pages = PyList::empty(py);
        for page in pages {
            let p = PyDict::new(py);
            p.set_item("slug", &page.slug)?;
            p.set_item("title", &page.title)?;
            p.set_item("grid_cols", page.grid_cols)?;

            let py_modules = PyList::empty(py);
            for module in &page.modules {
                let m = PyDict::new(py);
                match module {
                    PageModule::Chart(spec) => {
                        m.set_item("module_type", "chart")?;
                        m.set_item("title", &spec.title)?;
                        m.set_item("chart_type", spec.config.chart_type_str())?;
                        m.set_item("source_key", &spec.source_key)?;
                        m.set_item("grid_row", spec.grid.row)?;
                        m.set_item("grid_col", spec.grid.col)?;
                        m.set_item("grid_col_span", spec.grid.col_span)?;
                        m.set_item("filtered", spec.filtered)?;
                        match &spec.config {
                            ChartConfig::GroupedBar(c) => {
                                m.set_item("x_col", &c.x_col)?;
                                m.set_item("group_col", &c.group_col)?;
                                m.set_item("value_col", &c.value_col)?;
                                m.set_item("y_label", &c.y_label)?;
                            }
                            ChartConfig::Line(c) => {
                                m.set_item("x_col", &c.x_col)?;
                                m.set_item("y_cols", c.y_cols.join(","))?;
                                m.set_item("y_label", &c.y_label)?;
                            }
                            ChartConfig::HBar(c) => {
                                m.set_item("category_col", &c.category_col)?;
                                m.set_item("value_col", &c.value_col)?;
                                m.set_item("x_label", &c.x_label)?;
                            }
                            ChartConfig::Scatter(c) => {
                                m.set_item("x_col", &c.x_col)?;
                                m.set_item("y_col", &c.y_col)?;
                                m.set_item("x_label", &c.x_label)?;
                                m.set_item("y_label", &c.y_label)?;
                            }
                        }
                    }
                    PageModule::Paragraph(spec) => {
                        m.set_item("module_type", "paragraph")?;
                        m.set_item("title", spec.title.as_deref().unwrap_or(""))?;
                        m.set_item("has_title", spec.title.is_some())?;
                        m.set_item("text", &spec.text)?;
                        m.set_item("grid_row", spec.grid.row)?;
                        m.set_item("grid_col", spec.grid.col)?;
                        m.set_item("grid_col_span", spec.grid.col_span)?;
                    }
                    PageModule::Table(spec) => {
                        m.set_item("module_type", "table")?;
                        m.set_item("title", &spec.title)?;
                        m.set_item("source_key", &spec.source_key)?;
                        m.set_item("grid_row", spec.grid.row)?;
                        m.set_item("grid_col", spec.grid.col)?;
                        m.set_item("grid_col_span", spec.grid.col_span)?;

                        let py_cols = PyList::empty(py);
                        for col in &spec.columns {
                            let c = PyDict::new(py);
                            c.set_item("key", &col.key)?;
                            c.set_item("label", &col.label)?;
                            match &col.format {
                                ColumnFormat::Text => {
                                    c.set_item("format", "text")?;
                                }
                                ColumnFormat::Number { decimals } => {
                                    c.set_item("format", "number")?;
                                    c.set_item("decimals", *decimals)?;
                                }
                                ColumnFormat::Currency { symbol, decimals } => {
                                    c.set_item("format", "currency")?;
                                    c.set_item("symbol", symbol.as_str())?;
                                    c.set_item("decimals", *decimals)?;
                                }
                                ColumnFormat::Percent { decimals } => {
                                    c.set_item("format", "percent")?;
                                    c.set_item("decimals", *decimals)?;
                                }
                            }
                            py_cols.append(c)?;
                        }
                        m.set_item("columns", py_cols)?;
                    }
                }
                py_modules.append(m)?;
            }
            p.set_item("modules", py_modules)?;

            let py_filters = PyList::empty(py);
            for filter in &page.filters {
                let f = PyDict::new(py);
                f.set_item("source_key", &filter.source_key)?;
                f.set_item("column", &filter.column)?;
                f.set_item("label", &filter.label)?;
                match &filter.config {
                    FilterConfig::Range { min, max, step } => {
                        f.set_item("kind", "range")?;
                        f.set_item("min", *min)?;
                        f.set_item("max", *max)?;
                        f.set_item("step", *step)?;
                    }
                    FilterConfig::Select { options } => {
                        f.set_item("kind", "select")?;
                        let py_opts = PyList::new(py, options)?;
                        f.set_item("options", py_opts)?;
                    }
                    FilterConfig::Group { options } => {
                        f.set_item("kind", "group")?;
                        let py_opts = PyList::new(py, options)?;
                        f.set_item("options", py_opts)?;
                    }
                    FilterConfig::Threshold { value, above } => {
                        f.set_item("kind", "threshold")?;
                        f.set_item("value", *value)?;
                        f.set_item("above", *above)?;
                    }
                    FilterConfig::TopN { max_n, descending } => {
                        f.set_item("kind", "top_n")?;
                        f.set_item("max_n", *max_n)?;
                        f.set_item("descending", *descending)?;
                    }
                }
                py_filters.append(f)?;
            }
            p.set_item("filters", py_filters)?;
            py_pages.append(p)?;
        }

        let locals = PyDict::new(py);
        locals.set_item("frames", py_frames)?;
        locals.set_item("pages", py_pages)?;
        locals.set_item("nav_links", py_nav)?;
        locals.set_item("html_template", html_template)?;
        locals.set_item("output_dir", output_dir)?;

        let code = CString::new(python_script).map_err(|_| ChartError::InvalidScript)?;
        py.run(code.as_c_str(), Some(&locals), Some(&locals))?;

        println!("Dashboard generated: {} pages in {}/", pages.len(), output_dir);
        Ok(())
    })
}
