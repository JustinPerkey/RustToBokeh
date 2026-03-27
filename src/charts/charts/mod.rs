pub mod grouped_bar;
pub mod hbar;
pub mod histogram;
pub mod line;
pub mod pie;
pub mod scatter;
pub mod spec;

pub use grouped_bar::{GroupedBarConfig, GroupedBarConfigBuilder};
pub use hbar::{HBarConfig, HBarConfigBuilder};
pub use histogram::{HistogramConfig, HistogramConfigBuilder, HistogramDisplay};
pub use line::{LineConfig, LineConfigBuilder};
pub use pie::{PieConfig, PieConfigBuilder};
pub use scatter::{ScatterConfig, ScatterConfigBuilder};
pub use spec::ChartSpecBuilder;

// ── Grid constants ────────────────────────────────────────────────────────────

/// Maximum number of columns allowed in a page grid.
///
/// Grids wider than this are rejected by [`PageBuilder::build`](crate::pages::PageBuilder::build)
/// with a [`ChartError::GridValidation`](crate::error::ChartError::GridValidation) error.
pub const MAX_GRID_COLS: usize = 6;

// ── Chart config enum ────────────────────────────────────────────────────────

/// Enum wrapping all supported chart configuration types.
///
/// Each variant holds the typed configuration for one chart type. This is
/// stored inside [`ChartSpec`] and used by the renderer to determine how to
/// build the Bokeh figure. You typically won't construct this directly;
/// instead use the convenience constructors on [`ChartSpecBuilder`] (e.g.
/// [`ChartSpecBuilder::bar`], [`ChartSpecBuilder::line`]).
pub enum ChartConfig {
    /// A vertical grouped bar chart. See [`GroupedBarConfig`].
    GroupedBar(GroupedBarConfig),
    /// A multi-line chart. See [`LineConfig`].
    Line(LineConfig),
    /// A horizontal bar chart. See [`HBarConfig`].
    HBar(HBarConfig),
    /// A scatter plot. See [`ScatterConfig`].
    Scatter(ScatterConfig),
    /// A pie or donut chart. See [`PieConfig`].
    Pie(PieConfig),
    /// A histogram. See [`HistogramConfig`].
    Histogram(HistogramConfig),
}

impl ChartConfig {
    /// Return the string identifier used by the Python renderer to dispatch
    /// chart construction (e.g. `"grouped_bar"`, `"line_multi"`).
    #[must_use]
    pub fn chart_type_str(&self) -> &'static str {
        match self {
            ChartConfig::GroupedBar(_) => "grouped_bar",
            ChartConfig::Line(_) => "line_multi",
            ChartConfig::HBar(_) => "hbar",
            ChartConfig::Scatter(_) => "scatter",
            ChartConfig::Pie(_) => "pie",
            ChartConfig::Histogram(_) => "histogram",
        }
    }
}

// ── Layout structs ──────────────────────────────────────────────────────────

/// Position and span of a chart within the page's CSS grid layout.
///
/// The page grid has a fixed number of columns (set by
/// [`PageBuilder::new`](crate::pages::PageBuilder::new)). Each chart occupies
/// one or more columns in a given row.
pub struct GridCell {
    /// Zero-based row index.
    pub row: usize,
    /// Zero-based column index.
    pub col: usize,
    /// Number of grid columns this chart spans (minimum 1).
    pub col_span: usize,
}

/// A fully specified chart ready to be placed on a page.
///
/// Combines a chart's title, data source reference, typed configuration,
/// grid position, and filter opt-in flag. Constructed via
/// [`ChartSpecBuilder`].
pub struct ChartSpec {
    /// Display title rendered above the chart.
    pub title: String,
    /// Key into the frames dictionary identifying which `DataFrame` to use.
    /// Must match a key registered with [`Dashboard::add_df`](crate::Dashboard::add_df).
    pub source_key: String,
    /// Typed chart configuration (bar, line, hbar, or scatter).
    pub config: ChartConfig,
    /// Position of this chart in the page grid.
    pub grid: GridCell,
    /// Whether this chart participates in page-level filtering via
    /// [`CDSView`](https://docs.bokeh.org/en/latest/docs/reference/models/sources.html#cdsview).
    /// Set to `true` by calling [`ChartSpecBuilder::filtered`].
    pub filtered: bool,
    /// Explicit figure width in pixels.  When `None`, the chart uses
    /// `sizing_mode="stretch_width"` to fill its grid cell.
    pub width: Option<u32>,
    /// Explicit figure height in pixels.  When `None`, each chart type uses
    /// a sensible default (typically 400 px).
    pub height: Option<u32>,
}
