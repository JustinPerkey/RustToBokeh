//! PyO3 builders for `ChartConfig` and related visual customisation types
//! (palette, tooltip spec, axis config).

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::charts::{AxisConfig, ChartConfig, PaletteSpec, TooltipFormat, TooltipSpec};

pub(super) fn build_py_palette<'py>(py: Python<'py>, p: &PaletteSpec) -> PyResult<Bound<'py, PyDict>> {
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

pub(super) fn build_py_tooltip_spec<'py>(
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

pub(super) fn build_py_axis_config<'py>(
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

pub(super) fn build_py_chart_config<'py>(
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
                m.set_item("marker", mk.as_str())?;
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
