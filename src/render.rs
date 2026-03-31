//! Low-level rendering bridge between Rust and Python.
//!
//! This module is intentionally private — use [`Dashboard::render`](crate::Dashboard::render)
//! for the high-level API or [`render_dashboard`] for direct control.

use crate::charts::{AxisConfig, ChartConfig, FilterConfig, FilterSpec, PaletteSpec, TooltipFormat, TooltipSpec};
use crate::error::ChartError;
use crate::modules::{ColumnFormat, PageModule};
use crate::pages::Page;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::ffi::CString;

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

fn build_py_palette<'py>(py: Python<'py>, p: &PaletteSpec) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    match p {
        PaletteSpec::Named(name) => {
            d.set_item("kind", "named")?;
            d.set_item("value", name)?;
        }
        PaletteSpec::Custom(colors) => {
            d.set_item("kind", "custom")?;
            d.set_item("value", PyList::new(py, colors)?)?;
        }
    }
    Ok(d)
}

fn build_py_tooltip_spec<'py>(
    py: Python<'py>,
    spec: &TooltipSpec,
) -> PyResult<Bound<'py, PyList>> {
    let list = PyList::empty(py);
    for field in &spec.fields {
        let d = PyDict::new(py);
        d.set_item("column", &field.column)?;
        d.set_item("label", &field.label)?;
        match &field.format {
            TooltipFormat::Text => {
                d.set_item("format", "text")?;
                d.set_item("decimals", py.None())?;
            }
            TooltipFormat::Number(dec) => {
                d.set_item("format", "number")?;
                match dec {
                    Some(n) => d.set_item("decimals", n)?,
                    None => d.set_item("decimals", py.None())?,
                }
            }
            TooltipFormat::Percent(dec) => {
                d.set_item("format", "percent")?;
                match dec {
                    Some(n) => d.set_item("decimals", n)?,
                    None => d.set_item("decimals", py.None())?,
                }
            }
            TooltipFormat::Currency => {
                d.set_item("format", "currency")?;
                d.set_item("decimals", py.None())?;
            }
            TooltipFormat::DateTime(scale) => {
                d.set_item("format", "datetime")?;
                d.set_item("time_scale", scale.as_str())?;
                d.set_item("decimals", py.None())?;
            }
        }
        list.append(d)?;
    }
    Ok(list)
}

fn build_py_axis_config<'py>(
    py: Python<'py>,
    axis: &AxisConfig,
) -> PyResult<Bound<'py, PyDict>> {
    let d = PyDict::new(py);
    match axis.start {
        Some(v) => d.set_item("start", v)?,
        None => d.set_item("start", py.None())?,
    }
    match axis.end {
        Some(v) => d.set_item("end", v)?,
        None => d.set_item("end", py.None())?,
    }
    match axis.bounds_min {
        Some(v) => d.set_item("bounds_min", v)?,
        None => d.set_item("bounds_min", py.None())?,
    }
    match axis.bounds_max {
        Some(v) => d.set_item("bounds_max", v)?,
        None => d.set_item("bounds_max", py.None())?,
    }
    match axis.label_rotation {
        Some(v) => d.set_item("label_rotation", v)?,
        None => d.set_item("label_rotation", py.None())?,
    }
    match &axis.tick_format {
        Some(fmt) => d.set_item("tick_format", fmt)?,
        None => d.set_item("tick_format", py.None())?,
    }
    d.set_item("show_grid", axis.show_grid)?;
    match &axis.time_scale {
        Some(scale) => d.set_item("time_scale", scale.as_str())?,
        None => d.set_item("time_scale", py.None())?,
    }
    Ok(d)
}

fn build_py_chart_config<'py>(
    py: Python<'py>,
    m: &Bound<'py, PyDict>,
    config: &ChartConfig,
) -> PyResult<()> {
    match config {
        ChartConfig::GroupedBar(c) => {
            m.set_item("x_col", &c.x_col)?;
            m.set_item("group_col", &c.group_col)?;
            m.set_item("value_col", &c.value_col)?;
            m.set_item("y_label", &c.y_label)?;
            if let Some(p) = &c.palette {
                m.set_item("palette", build_py_palette(py, p)?)?;
            }
            if let Some(w) = c.bar_width {
                m.set_item("bar_width", w)?;
            }
            if let Some(tt) = &c.tooltips {
                m.set_item("tooltips", build_py_tooltip_spec(py, tt)?)?;
            }
            if let Some(ax) = &c.x_axis {
                m.set_item("x_axis", build_py_axis_config(py, ax)?)?;
            }
            if let Some(ax) = &c.y_axis {
                m.set_item("y_axis", build_py_axis_config(py, ax)?)?;
            }
        }
        ChartConfig::Line(c) => {
            m.set_item("x_col", &c.x_col)?;
            m.set_item("y_cols", c.y_cols.join(","))?;
            m.set_item("y_label", &c.y_label)?;
            if let Some(p) = &c.palette {
                m.set_item("palette", build_py_palette(py, p)?)?;
            }
            if let Some(w) = c.line_width {
                m.set_item("line_width", w)?;
            }
            if let Some(s) = c.point_size {
                m.set_item("point_size", s)?;
            }
            if let Some(tt) = &c.tooltips {
                m.set_item("tooltips", build_py_tooltip_spec(py, tt)?)?;
            }
            if let Some(ax) = &c.x_axis {
                m.set_item("x_axis", build_py_axis_config(py, ax)?)?;
            }
            if let Some(ax) = &c.y_axis {
                m.set_item("y_axis", build_py_axis_config(py, ax)?)?;
            }
        }
        ChartConfig::HBar(c) => {
            m.set_item("category_col", &c.category_col)?;
            m.set_item("value_col", &c.value_col)?;
            m.set_item("x_label", &c.x_label)?;
            if let Some(col) = &c.color {
                m.set_item("color", col)?;
            }
            if let Some(tt) = &c.tooltips {
                m.set_item("tooltips", build_py_tooltip_spec(py, tt)?)?;
            }
            if let Some(ax) = &c.x_axis {
                m.set_item("x_axis", build_py_axis_config(py, ax)?)?;
            }
            if let Some(ax) = &c.y_axis {
                m.set_item("y_axis", build_py_axis_config(py, ax)?)?;
            }
        }
        ChartConfig::Scatter(c) => {
            m.set_item("x_col", &c.x_col)?;
            m.set_item("y_col", &c.y_col)?;
            m.set_item("x_label", &c.x_label)?;
            m.set_item("y_label", &c.y_label)?;
            if let Some(col) = &c.color {
                m.set_item("color", col)?;
            }
            if let Some(mk) = &c.marker {
                m.set_item("marker", mk)?;
            }
            if let Some(sz) = c.marker_size {
                m.set_item("marker_size", sz)?;
            }
            if let Some(a) = c.alpha {
                m.set_item("alpha", a)?;
            }
            if let Some(tt) = &c.tooltips {
                m.set_item("tooltips", build_py_tooltip_spec(py, tt)?)?;
            }
            if let Some(ax) = &c.x_axis {
                m.set_item("x_axis", build_py_axis_config(py, ax)?)?;
            }
            if let Some(ax) = &c.y_axis {
                m.set_item("y_axis", build_py_axis_config(py, ax)?)?;
            }
        }
        ChartConfig::Pie(c) => {
            m.set_item("label_col", &c.label_col)?;
            m.set_item("value_col", &c.value_col)?;
            if let Some(r) = c.inner_radius {
                m.set_item("inner_radius", r)?;
            }
            if let Some(p) = &c.palette {
                m.set_item("palette", build_py_palette(py, p)?)?;
            }
            if let Some(tt) = &c.tooltips {
                m.set_item("tooltips", build_py_tooltip_spec(py, tt)?)?;
            }
            if let Some(show) = c.show_legend {
                m.set_item("show_legend", show)?;
            }
            if let Some(side) = &c.legend_side {
                m.set_item("legend_side", side.as_str())?;
            }
        }
        ChartConfig::Histogram(c) => {
            m.set_item("x_label", &c.x_label)?;
            m.set_item("display", c.display.as_ref().map_or("count", |d| d.as_str()))?;
            if let Some(s) = &c.y_label {
                m.set_item("y_label", s)?;
            }
            if let Some(s) = &c.color {
                m.set_item("color", s)?;
            }
            if let Some(s) = &c.line_color {
                m.set_item("line_color", s)?;
            }
            if let Some(a) = c.alpha {
                m.set_item("alpha", a)?;
            }
            if let Some(tt) = &c.tooltips {
                m.set_item("tooltips", build_py_tooltip_spec(py, tt)?)?;
            }
            if let Some(ax) = &c.x_axis {
                m.set_item("x_axis", build_py_axis_config(py, ax)?)?;
            }
            if let Some(ax) = &c.y_axis {
                m.set_item("y_axis", build_py_axis_config(py, ax)?)?;
            }
        }
        ChartConfig::BoxPlot(c) => {
            m.set_item("category_col", &c.category_col)?;
            m.set_item("q1_col",       &c.q1_col)?;
            m.set_item("q2_col",       &c.q2_col)?;
            m.set_item("q3_col",       &c.q3_col)?;
            m.set_item("lower_col",    &c.lower_col)?;
            m.set_item("upper_col",    &c.upper_col)?;
            m.set_item("y_label",      &c.y_label)?;
            if let Some(col) = &c.color {
                m.set_item("color", col)?;
            }
            if let Some(a) = c.alpha {
                m.set_item("alpha", a)?;
            }
            if let Some(tt) = &c.tooltips {
                m.set_item("tooltips", build_py_tooltip_spec(py, tt)?)?;
            }
            if let Some(ax) = &c.y_axis {
                m.set_item("y_axis", build_py_axis_config(py, ax)?)?;
            }
            if let Some(p) = &c.palette {
                m.set_item("palette", build_py_palette(py, p)?)?;
            }
            if let Some(key) = &c.outlier_source_key {
                m.set_item("outlier_source_key", key)?;
            }
            if let Some(col) = &c.outlier_value_col {
                m.set_item("outlier_value_col", col)?;
            }
        }
        ChartConfig::Density(c) => {
            m.set_item("category_col", &c.category_col)?;
            m.set_item("value_col",    &c.value_col)?;
            m.set_item("y_label",      &c.y_label)?;
            if let Some(p) = &c.palette {
                m.set_item("palette", build_py_palette(py, p)?)?;
            }
            if let Some(col) = &c.color {
                m.set_item("color", col)?;
            }
            if let Some(a) = c.alpha {
                m.set_item("alpha", a)?;
            }
            if let Some(ax) = &c.y_axis {
                m.set_item("y_axis", build_py_axis_config(py, ax)?)?;
            }
            if let Some(t) = c.point_threshold {
                m.set_item("point_threshold", t)?;
            }
        }
    }
    Ok(())
}

fn build_py_column_format(c: &Bound<'_, PyDict>, format: &ColumnFormat) -> PyResult<()> {
    match format {
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
    Ok(())
}

fn build_py_filter_config<'py>(
    py: Python<'py>,
    f: &Bound<'py, PyDict>,
    config: &FilterConfig,
) -> PyResult<()> {
    match config {
        FilterConfig::Range { min, max, step } => {
            f.set_item("kind", "range")?;
            f.set_item("min", *min)?;
            f.set_item("max", *max)?;
            f.set_item("step", *step)?;
        }
        FilterConfig::Select { options } => {
            f.set_item("kind", "select")?;
            f.set_item("options", PyList::new(py, options)?)?;
        }
        FilterConfig::Group { options } => {
            f.set_item("kind", "group")?;
            f.set_item("options", PyList::new(py, options)?)?;
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
        FilterConfig::DateRange { min_ms, max_ms, step, scale } => {
            f.set_item("kind", "date_range")?;
            f.set_item("min_ms", *min_ms)?;
            f.set_item("max_ms", *max_ms)?;
            f.set_item("step_ms", step.as_ms())?;
            f.set_item("time_scale", scale.as_str())?;
        }
        FilterConfig::RangeTool { y_column, start, end, time_scale } => {
            f.set_item("kind", "range_tool")?;
            f.set_item("y_column", y_column)?;
            f.set_item("start", *start)?;
            f.set_item("end", *end)?;
            match time_scale {
                Some(s) => f.set_item("time_scale", s.as_str())?,
                None => f.set_item("time_scale", py.None())?,
            }
        }
    }
    Ok(())
}

fn build_py_module<'py>(py: Python<'py>, module: &PageModule) -> PyResult<Bound<'py, PyDict>> {
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
            m.set_item("width", spec.width)?;
            m.set_item("height", spec.height)?;
            build_py_chart_config(py, &m, &spec.config)?;
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
                build_py_column_format(&c, &col.format)?;
                py_cols.append(c)?;
            }
            m.set_item("columns", py_cols)?;
        }
    }
    Ok(m)
}

fn build_py_filter<'py>(py: Python<'py>, filter: &FilterSpec) -> PyResult<Bound<'py, PyDict>> {
    let f = PyDict::new(py);
    f.set_item("source_key", &filter.source_key)?;
    f.set_item("column", &filter.column)?;
    f.set_item("label", &filter.label)?;
    build_py_filter_config(py, &f, &filter.config)?;
    Ok(f)
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

    let python_script = include_str!("../python/render.py");
    let html_template = include_str!("../templates/chart.html");

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
