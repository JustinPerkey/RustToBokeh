//! Convenience re-exports for common usage.
//!
//! Importing the prelude brings all the types you need to define and render
//! dashboards into scope:
//!
//! ```ignore
//! use rust_to_bokeh::prelude::*;
//! ```
//!
//! This re-exports the [`Dashboard`] builder, all chart config types and their
//! builders, [`ChartSpecBuilder`] for placing charts on a grid,
//! [`PageBuilder`] and [`Page`] for assembling multi-page layouts,
//! [`FilterSpec`] and [`FilterConfig`] for adding interactive filters, and
//! [`ChartError`] for error handling.

pub use crate::charts::{
    AxisConfig, AxisConfigBuilder, BoxPlotConfig, BoxPlotConfigBuilder, ChartConfig, ChartSpec,
    ChartSpecBuilder, DateStep, DensityConfig, DensityConfigBuilder, FilterConfig, FilterSpec,
    GridCell, GroupedBarConfig, GroupedBarConfigBuilder, HBarConfig, HBarConfigBuilder,
    HistogramConfig, HistogramConfigBuilder, HistogramDisplay, LineConfig, LineConfigBuilder,
    MarkerType, PaletteSpec, PieConfig, PieConfigBuilder, ScatterConfig, ScatterConfigBuilder,
    TimeScale, TooltipField, TooltipFormat, TooltipSpec, TooltipSpecBuilder, MAX_GRID_COLS,
};
pub use crate::error::ChartError;
pub use crate::modules::{
    ColumnFormat, PageModule, ParagraphSpec, ParagraphSpecBuilder, TableColumn, TableSpec,
    TableSpecBuilder,
};
pub use crate::pages::{Page, PageBuilder};
pub use crate::{
    compute_box_outliers, compute_box_stats, compute_histogram, serialize_df, BokehResources,
    Dashboard, DfHandle, NavStyle,
};
#[cfg(feature = "python")]
pub use crate::render_dashboard;
