//! PyO3 builders for `PageModule`, `ColumnFormat`, and `FilterConfig`.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::charts::{FilterConfig, FilterSpec};
use crate::modules::{ColumnFormat, PageModule};

use super::chart_config::build_py_chart_config;

pub(super) fn build_py_module<'py>(py: Python<'py>, module: &PageModule) -> PyResult<Bound<'py, PyDict>> {
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

pub(super) fn build_py_filter<'py>(py: Python<'py>, filter: &FilterSpec) -> PyResult<Bound<'py, PyDict>> {
    let f = PyDict::new(py);
    f.set_item("source_key", &filter.source_key)?;
    f.set_item("column", &filter.column)?;
    f.set_item("label", &filter.label)?;
    build_py_filter_config(py, &f, &filter.config)?;
    Ok(f)
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
