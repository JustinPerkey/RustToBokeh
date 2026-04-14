//! # `RustToBokeh`
//!
//! A library for building interactive multi-page [Bokeh](https://bokeh.org/)
//! dashboards from Rust, using [Polars](https://pola.rs/) `DataFrames` for data
//! and either a pure-Rust native renderer or [PyO3](https://pyo3.rs/) to bridge
//! into a vendored Python interpreter.
//!
//! ## Quick start
//!
//! ```ignore
//! use rust_to_bokeh::prelude::*;
//! use polars::prelude::*;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut df = df![
//!         "month"    => ["Jan", "Feb", "Mar"],
//!         "revenue"  => [100.0, 150.0, 200.0f64],
//!         "expenses" => [80.0, 90.0, 110.0f64],
//!     ]?;
//!
//!     let mut dash = Dashboard::new();
//!     dash.add_df("trends", &mut df)?;
//!     dash.add_page(
//!         PageBuilder::new("overview", "Overview", "Overview", 2)
//!             .chart(ChartSpecBuilder::line("Revenue vs Expenses", "trends",
//!                 LineConfig::builder()
//!                     .x("month")
//!                     .y_cols(&["revenue", "expenses"])
//!                     .y_label("USD")
//!                     .build()?
//!             ).at(0, 0, 2).build())
//!             .build()?,
//!     );
//!     dash.render_native(BokehResources::Cdn)?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! Two rendering backends produce the same output:
//!
//! 1. **Native path** — [`Dashboard::render_native`] emits Bokeh's JSON document
//!    model directly from Rust, no Python required.
//! 2. **Python path** — [`Dashboard::render`] (feature `python`) passes serialized
//!    Arrow IPC bytes through PyO3 to the embedded `render.py` script.
//!
//! ## Modules
//!
//! - [`charts`] — Chart config types, builders, layout primitives, and filter definitions.
//! - [`pages`] — Page layout types for multi-page dashboards.
//! - [`modules`] — Content modules: paragraphs and data tables.
//! - [`stats`] — Statistical helpers for histograms and box plots.
//! - [`bokeh_native`] — Pure-Rust Bokeh HTML renderer (no Python).
//! - [`error`] — The [`ChartError`] type used throughout the library.
//! - [`prelude`] — Convenience re-exports for common usage.

pub mod bokeh_native;
pub mod charts;
mod dashboard;
pub mod error;
pub mod modules;
pub mod pages;
pub mod prelude;
pub mod stats;
#[cfg(feature = "python")]
mod python_config;
#[cfg(feature = "python")]
mod render;

pub use bokeh_native::BokehResources;
pub use charts::{
    AxisConfig, AxisConfigBuilder, BoxPlotConfig, BoxPlotConfigBuilder, ChartConfig, ChartSpec,
    ChartSpecBuilder, DateStep, DensityConfig, DensityConfigBuilder, FilterConfig, FilterSpec,
    GridCell, GroupedBarConfig, GroupedBarConfigBuilder, HBarConfig, HBarConfigBuilder,
    HistogramConfig, HistogramConfigBuilder, HistogramDisplay, LineConfig, LineConfigBuilder,
    PaletteSpec, PieConfig, PieConfigBuilder, ScatterConfig, ScatterConfigBuilder, TimeScale,
    TooltipField, TooltipFormat, TooltipSpec, TooltipSpecBuilder, MAX_GRID_COLS,
};
pub use dashboard::Dashboard;
pub use error::ChartError;
pub use modules::{
    ColumnFormat, PageModule, ParagraphSpec, ParagraphSpecBuilder, TableColumn, TableSpec,
    TableSpecBuilder,
};
pub use pages::{Page, PageBuilder};
#[cfg(feature = "python")]
pub use render::render_dashboard;
pub use stats::{compute_box_outliers, compute_box_stats, compute_histogram};
#[cfg(feature = "python")]
pub use python_config::configure_vendored_python;

/// Navigation bar orientation for the rendered dashboard.
///
/// - `Horizontal` (default) — sticky top bar with page links laid out in a row,
///   categories shown as inline labels before their group of links.
/// - `Vertical` — fixed left sidebar with categories as section headings and
///   page links stacked below each heading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavStyle {
    #[default]
    Horizontal,
    Vertical,
}

impl NavStyle {
    #[cfg(feature = "python")]
    fn as_str(self) -> &'static str {
        match self {
            NavStyle::Horizontal => "horizontal",
            NavStyle::Vertical => "vertical",
        }
    }
}

use polars::io::ipc::IpcWriter;
use polars::io::SerWriter;
use polars::prelude::DataFrame;
use std::io::Cursor;

/// Serialize a Polars `DataFrame` to Arrow IPC bytes.
///
/// This is the format used to pass data across the Rust-Python boundary.
/// You typically don't need to call this directly — [`Dashboard::add_df`]
/// handles serialization automatically.
///
/// # Errors
///
/// Returns [`ChartError::Serialization`] if the IPC writer fails.
pub fn serialize_df(df: &mut DataFrame) -> Result<Vec<u8>, ChartError> {
    let mut buf = Cursor::new(Vec::new());
    IpcWriter::new(&mut buf).finish(df)?;
    Ok(buf.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[cfg(feature = "python")]
    #[test]
    fn nav_style_horizontal_str() {
        assert_eq!(NavStyle::Horizontal.as_str(), "horizontal");
    }

    #[cfg(feature = "python")]
    #[test]
    fn nav_style_vertical_str() {
        assert_eq!(NavStyle::Vertical.as_str(), "vertical");
    }

    #[test]
    fn nav_style_default_is_horizontal() {
        assert_eq!(NavStyle::default(), NavStyle::Horizontal);
    }

    #[test]
    fn serialize_df_produces_nonempty_bytes() {
        let mut df = df![
            "x" => [1i64, 2, 3],
            "y" => [4.0f64, 5.0, 6.0],
        ]
        .unwrap();
        let bytes = serialize_df(&mut df).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn serialize_df_bytes_are_valid_ipc() {
        use polars::io::ipc::IpcReader;
        use polars::io::SerReader;
        use std::io::Cursor;

        let mut df = df![
            "name" => ["Alice", "Bob"],
            "score" => [95.0f64, 87.5],
        ]
        .unwrap();
        let bytes = serialize_df(&mut df).unwrap();

        let restored = IpcReader::new(Cursor::new(bytes)).finish().unwrap();
        assert_eq!(restored.height(), 2);
        assert_eq!(restored.width(), 2);
        let names: Vec<&str> = restored
            .get_column_names()
            .iter()
            .map(|s| s.as_str())
            .collect();
        assert!(names.contains(&"name"));
        assert!(names.contains(&"score"));
    }

    #[test]
    fn serialize_df_empty_dataframe() {
        let mut df = df![
            "a" => Vec::<i64>::new(),
        ]
        .unwrap();
        let bytes = serialize_df(&mut df).unwrap();
        assert!(!bytes.is_empty());
    }
}
