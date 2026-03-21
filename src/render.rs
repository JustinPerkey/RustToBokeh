use crate::charts::{ChartConfig, FilterConfig};
use crate::pages::Page;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::ffi::CString;

/// Render a multi-page Bokeh dashboard to HTML files.
///
/// Takes serialized DataFrames (Arrow IPC bytes keyed by name), page
/// definitions, and an output directory.  Each page produces one HTML file
/// with inter-page navigation.
pub fn render_dashboard(
    frame_data: &[(&str, Vec<u8>)],
    pages: &[Page],
    output_dir: &str,
) -> PyResult<()> {
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

        // Pages with nested specs
        let py_pages = PyList::empty(py);
        for page in pages {
            let p = PyDict::new(py);
            p.set_item("slug", &page.slug)?;
            p.set_item("title", &page.title)?;
            p.set_item("grid_cols", page.grid_cols)?;

            let py_specs = PyList::empty(py);
            for spec in &page.specs {
                let s = PyDict::new(py);
                s.set_item("title", &spec.title)?;
                s.set_item("chart_type", spec.config.chart_type_str())?;
                s.set_item("source_key", &spec.source_key)?;
                s.set_item("grid_row", spec.grid.row)?;
                s.set_item("grid_col", spec.grid.col)?;
                s.set_item("grid_col_span", spec.grid.col_span)?;
                s.set_item("filtered", spec.filtered)?;
                match &spec.config {
                    ChartConfig::GroupedBar(c) => {
                        s.set_item("x_col", &c.x_col)?;
                        s.set_item("group_col", &c.group_col)?;
                        s.set_item("value_col", &c.value_col)?;
                        s.set_item("y_label", &c.y_label)?;
                    }
                    ChartConfig::Line(c) => {
                        s.set_item("x_col", &c.x_col)?;
                        s.set_item("y_cols", c.y_cols.join(","))?;
                        s.set_item("y_label", &c.y_label)?;
                    }
                    ChartConfig::HBar(c) => {
                        s.set_item("category_col", &c.category_col)?;
                        s.set_item("value_col", &c.value_col)?;
                        s.set_item("x_label", &c.x_label)?;
                    }
                    ChartConfig::Scatter(c) => {
                        s.set_item("x_col", &c.x_col)?;
                        s.set_item("y_col", &c.y_col)?;
                        s.set_item("x_label", &c.x_label)?;
                        s.set_item("y_label", &c.y_label)?;
                    }
                }
                py_specs.append(s)?;
            }
            p.set_item("specs", py_specs)?;

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

        let code = CString::new(python_script).expect("Python script contains null byte");
        py.run(code.as_c_str(), Some(&locals), Some(&locals))?;

        println!("Dashboard generated: {} pages in {}/", pages.len(), output_dir);
        Ok(())
    })
}
