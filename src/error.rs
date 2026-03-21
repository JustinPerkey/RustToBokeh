//! Error types for chart building and dashboard rendering.
//!
//! [`ChartError`] is the single error type used throughout the library. It
//! covers builder validation, DataFrame serialization, and Python execution
//! failures.
//!
//! # Handling errors
//!
//! All fallible operations in the library return `Result<T, ChartError>`.
//! The error implements [`std::error::Error`] and provides [`From`]
//! conversions for [`polars::error::PolarsError`] and [`pyo3::PyErr`], so
//! it works seamlessly with the `?` operator in functions that return
//! `Result<T, ChartError>` or `Result<T, Box<dyn Error>>`.
//!
//! ```ignore
//! use rust_to_bokeh::prelude::*;
//!
//! fn run() -> Result<(), ChartError> {
//!     let config = GroupedBarConfig::builder()
//!         .x("month")
//!         .group("category")
//!         .value("amount")
//!         .y_label("USD")
//!         .build()?;  // Returns ChartError::MissingField if a setter was skipped
//!     Ok(())
//! }
//! ```

use std::fmt;

/// Errors that can occur when building chart configurations, serializing
/// DataFrames, or rendering dashboards.
///
/// # Variants
///
/// | Variant | Cause | Typical fix |
/// |---|---|---|
/// | [`MissingField`](Self::MissingField) | A required builder field was not set | Call the missing setter before `build()` |
/// | [`Serialization`](Self::Serialization) | Polars failed to write Arrow IPC | Check DataFrame schema and column types |
/// | [`Python`](Self::Python) | Python raised an exception during rendering | Check `render.py` logic and Python dependencies |
/// | [`InvalidScript`](Self::InvalidScript) | Embedded script contains a null byte | Should not occur in normal usage |
#[derive(Debug)]
pub enum ChartError {
    /// A required field was not set on a config builder.
    ///
    /// The contained `&'static str` is the name of the missing field
    /// (e.g. `"x_col"`, `"value_col"`).
    MissingField(&'static str),

    /// DataFrame serialization to Arrow IPC format failed.
    ///
    /// Wraps the underlying [`polars::error::PolarsError`].
    Serialization(polars::error::PolarsError),

    /// Python execution failed during dashboard rendering.
    ///
    /// Wraps the underlying [`pyo3::PyErr`]. Common causes include missing
    /// Python packages, data schema mismatches, or bugs in `render.py`.
    Python(pyo3::PyErr),

    /// The embedded Python script contains a null byte, preventing it from
    /// being passed to the Python interpreter via `CString`.
    ///
    /// This should not occur under normal circumstances since the script is
    /// embedded at compile time via `include_str!()`.
    InvalidScript,
}

impl fmt::Display for ChartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChartError::MissingField(field) => write!(f, "missing required field: {field}"),
            ChartError::Serialization(e) => write!(f, "DataFrame serialization failed: {e}"),
            ChartError::Python(e) => write!(f, "Python execution failed: {e}"),
            ChartError::InvalidScript => write!(f, "embedded Python script contains a null byte"),
        }
    }
}

impl std::error::Error for ChartError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ChartError::Serialization(e) => Some(e),
            ChartError::Python(e) => Some(e),
            _ => None,
        }
    }
}

impl From<polars::error::PolarsError> for ChartError {
    fn from(e: polars::error::PolarsError) -> Self {
        ChartError::Serialization(e)
    }
}

impl From<pyo3::PyErr> for ChartError {
    fn from(e: pyo3::PyErr) -> Self {
        ChartError::Python(e)
    }
}
