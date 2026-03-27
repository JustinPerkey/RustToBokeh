//! Chart configuration types, layout primitives, and filter definitions.
//!
//! This module contains everything needed to declaratively describe charts
//! and filters in Rust. The types defined here are consumed by the Python
//! renderer ([`crate::render::render_dashboard`]) to produce interactive Bokeh
//! visualizations.
//!
//! # Chart types
//!
//! Each supported chart type has a dedicated config struct and builder:
//!
//! | Chart type | Config struct | Builder | Description |
//! |---|---|---|---|
//! | Grouped bar | [`GroupedBarConfig`] | [`GroupedBarConfigBuilder`] | Vertical bars grouped by category |
//! | Multi-line | [`LineConfig`] | [`LineConfigBuilder`] | One or more line series on a shared axis |
//! | Horizontal bar | [`HBarConfig`] | [`HBarConfigBuilder`] | Horizontal bars for ranked/categorical data |
//! | Scatter plot | [`ScatterConfig`] | [`ScatterConfigBuilder`] | X-Y scatter with circle markers |
//!
//! All config builders follow the same pattern: call the type's `builder()`
//! method, chain setter methods for each required field, then call `build()` to
//! get a `Result<Config, ChartError>`.
//!
//! # Layout
//!
//! Charts are positioned on a page grid using [`ChartSpecBuilder::at`], which
//! sets the row, column, and column span. The grid dimensions are defined by
//! [`PageBuilder::new`].
//!
//! # Filters
//!
//! Interactive filters are defined with [`FilterSpec`] factory methods and
//! attached to pages via [`crate::pages::PageBuilder::filter`]. Charts must be
//! marked with [`ChartSpecBuilder::filtered`] to opt into filtering.

pub mod charts;
pub mod customization;

pub use charts::{
    ChartConfig, ChartSpec, ChartSpecBuilder, GridCell, MAX_GRID_COLS,
    GroupedBarConfig, GroupedBarConfigBuilder,
    HBarConfig, HBarConfigBuilder,
    HistogramConfig, HistogramConfigBuilder, HistogramDisplay,
    LineConfig, LineConfigBuilder,
    PieConfig, PieConfigBuilder,
    ScatterConfig, ScatterConfigBuilder,
};
pub use customization::{
    AxisConfig, AxisConfigBuilder,
    DateStep,
    FilterConfig, FilterSpec,
    PaletteSpec, TimeScale,
    TooltipField, TooltipFormat, TooltipSpec, TooltipSpecBuilder,
};
