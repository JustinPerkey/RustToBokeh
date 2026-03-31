//! # `RustToBokeh`
//!
//! A library for building interactive multi-page [Bokeh](https://bokeh.org/)
//! dashboards from Rust, using [Polars](https://pola.rs/) `DataFrames` for data
//! and [PyO3](https://pyo3.rs/) to bridge into Python for rendering.
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
//!     dash.render()?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The rendering pipeline works as follows:
//!
//! 1. **Build `DataFrames`** in Rust using Polars.
//! 2. **Register data** with [`Dashboard::add_df`], which serializes each
//!    `DataFrame` to Arrow IPC bytes.
//! 3. **Define pages** with [`PageBuilder`], adding [`ChartSpec`](charts::ChartSpec)s
//!    and optional [`FilterSpec`](charts::FilterSpec)s.
//! 4. **Call [`Dashboard::render`]**, which acquires the Python GIL via `PyO3`,
//!    passes all data and page definitions to `render.py`, and produces one
//!    HTML file per page in the output directory.
//!
//! The Python script (`render.py`) and HTML template (`chart.html`) are
//! embedded into the binary at compile time via `include_str!()`, so the
//! final executable has no runtime file dependencies beyond a Python
//! interpreter with the required packages installed.
//!
//! ## Modules
//!
//! - [`charts`] — Chart config types, builders, layout primitives, and filter definitions.
//! - [`pages`] — Page layout types for multi-page dashboards.
//! - [`modules`] — Content modules: paragraphs and data tables.
//! - [`stats`] — Statistical helpers for histograms and box plots.
//! - [`error`] — The [`ChartError`] type used throughout the library.
//! - [`prelude`] — Convenience re-exports for common usage.

pub mod charts;
pub mod error;
pub mod modules;
pub mod pages;
pub mod prelude;
pub mod stats;
mod python_config;
mod render;

pub use charts::{
    AxisConfig, AxisConfigBuilder, BoxPlotConfig, BoxPlotConfigBuilder, ChartConfig, ChartSpec,
    ChartSpecBuilder, DateStep, DensityConfig, DensityConfigBuilder, FilterConfig, FilterSpec,
    GridCell, GroupedBarConfig, GroupedBarConfigBuilder, HBarConfig, HBarConfigBuilder,
    HistogramConfig, HistogramConfigBuilder, HistogramDisplay, LineConfig, LineConfigBuilder,
    PaletteSpec, PieConfig, PieConfigBuilder, ScatterConfig, ScatterConfigBuilder, TimeScale,
    TooltipField, TooltipFormat, TooltipSpec, TooltipSpecBuilder, MAX_GRID_COLS,
};
pub use error::ChartError;
pub use modules::{
    ColumnFormat, PageModule, ParagraphSpec, ParagraphSpecBuilder, TableColumn, TableSpec,
    TableSpecBuilder,
};
pub use pages::{Page, PageBuilder};
pub use render::render_dashboard;
pub use stats::{compute_box_outliers, compute_box_stats, compute_histogram};
pub use python_config::configure_vendored_python;

/// Navigation bar orientation for the rendered dashboard.
///
/// - `Horizontal` (default) — sticky top bar with page links laid out in a row,
///   categories shown as inline labels before their group of links.
/// - `Vertical` — fixed left sidebar with categories as section headings and
///   page links stacked below each heading. The main content shifts right to
///   accommodate the sidebar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavStyle {
    #[default]
    Horizontal,
    Vertical,
}

impl NavStyle {
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
/// Returns [`ChartError::Serialization`] if the IPC writer fails (e.g. due
/// to an unsupported column type).
pub fn serialize_df(df: &mut DataFrame) -> Result<Vec<u8>, ChartError> {
    let mut buf = Cursor::new(Vec::new());
    IpcWriter::new(&mut buf).finish(df)?;
    Ok(buf.into_inner())
}

/// High-level dashboard builder that collects `DataFrames` and pages, then
/// renders everything in one call.
///
/// # Workflow
///
/// 1. Create a dashboard with [`Dashboard::new`].
/// 2. Optionally set the output directory with [`output_dir`](Dashboard::output_dir)
///    (defaults to `"output"`).
/// 3. Register `DataFrames` with [`add_df`](Dashboard::add_df). Each `DataFrame`
///    is serialized immediately and stored under the given key.
/// 4. Add pages with [`add_page`](Dashboard::add_page). Charts on each page
///    reference `DataFrames` by their registered key.
/// 5. Call [`render`](Dashboard::render) to produce the HTML files.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
/// use polars::prelude::*;
///
/// let mut df = df!["x" => [1, 2, 3], "y" => [4, 5, 6]].unwrap();
///
/// let mut dash = Dashboard::new();
/// dash.add_df("my_data", &mut df)?;
/// dash.add_page(
///     PageBuilder::new("overview", "Overview", "Overview", 2)
///         .chart(ChartSpecBuilder::scatter("X vs Y", "my_data",
///             ScatterConfig::builder()
///                 .x("x").y("y").x_label("X").y_label("Y")
///                 .build()?
///         ).at(0, 0, 2).build())
///         .build()?,
/// );
/// dash.render()?;
/// ```
pub struct Dashboard {
    frames: Vec<(String, Vec<u8>)>,
    pages: Vec<Page>,
    output_dir: String,
    title: String,
    nav_style: NavStyle,
}

impl Dashboard {
    /// Create an empty dashboard with the default output directory (`"output"`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            pages: Vec::new(),
            output_dir: "output".into(),
            title: String::new(),
            nav_style: NavStyle::Horizontal,
        }
    }

    /// Set the report title displayed in the navigation bar on every page.
    ///
    /// When set, the title appears as a prominent label at the leading edge of
    /// the navigation (horizontal mode) or at the top of the sidebar (vertical
    /// mode). Defaults to empty (no title shown).
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.into();
        self
    }

    /// Set the navigation bar orientation.
    ///
    /// Defaults to [`NavStyle::Horizontal`]. Use [`NavStyle::Vertical`] to
    /// render a fixed left sidebar instead of a sticky top bar.
    #[must_use]
    pub fn nav_style(mut self, style: NavStyle) -> Self {
        self.nav_style = style;
        self
    }

    /// Set the output directory for generated HTML files.
    ///
    /// Defaults to `"output"`. The directory is created automatically by the
    /// Python renderer if it does not exist.
    #[must_use]
    pub fn output_dir(mut self, dir: &str) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Register a `DataFrame` under the given key.
    ///
    /// The `DataFrame` is serialized to Arrow IPC bytes immediately. Charts
    /// reference this data by using the same `key` as their `source_key`.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::Serialization`] if the `DataFrame` cannot be
    /// serialized (e.g. unsupported column types).
    pub fn add_df(&mut self, key: &str, df: &mut DataFrame) -> Result<&mut Self, ChartError> {
        self.frames.push((key.into(), serialize_df(df)?));
        Ok(self)
    }

    /// Add a pre-built [`Page`] to the dashboard.
    ///
    /// Pages are rendered in the order they are added. The navigation bar
    /// reflects this ordering.
    pub fn add_page(&mut self, page: Page) -> &mut Self {
        self.pages.push(page);
        self
    }

    /// Render all pages to HTML files in the output directory.
    ///
    /// This acquires the Python GIL, passes all serialized `DataFrames` and
    /// page definitions to the embedded `render.py` script, and writes one
    /// HTML file per page.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::Python`] if the Python script raises an
    /// exception, or [`ChartError::InvalidScript`] if the embedded script
    /// is malformed.
    pub fn render(&self) -> Result<(), ChartError> {
        let refs: Vec<(&str, Vec<u8>)> = self
            .frames
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        render_dashboard(
            &refs,
            &self.pages,
            &self.output_dir,
            &self.title,
            self.nav_style.as_str(),
        )
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    // ── NavStyle ──────────────────────────────────────────────────────────────

    #[test]
    fn nav_style_horizontal_str() {
        assert_eq!(NavStyle::Horizontal.as_str(), "horizontal");
    }

    #[test]
    fn nav_style_vertical_str() {
        assert_eq!(NavStyle::Vertical.as_str(), "vertical");
    }

    #[test]
    fn nav_style_default_is_horizontal() {
        assert_eq!(NavStyle::default(), NavStyle::Horizontal);
    }

    // ── serialize_df ──────────────────────────────────────────────────────────

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

        // Re-read the bytes and verify the data round-trips correctly.
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

    // ── Dashboard builder ─────────────────────────────────────────────────────

    #[test]
    fn dashboard_new_defaults() {
        let dash = Dashboard::new();
        assert_eq!(dash.output_dir, "output");
        assert_eq!(dash.title, "");
        assert_eq!(dash.nav_style, NavStyle::Horizontal);
        assert!(dash.frames.is_empty());
        assert!(dash.pages.is_empty());
    }

    #[test]
    fn dashboard_default_matches_new() {
        let a = Dashboard::new();
        let b = Dashboard::default();
        assert_eq!(a.output_dir, b.output_dir);
        assert_eq!(a.title, b.title);
    }

    #[test]
    fn dashboard_title_sets_title() {
        let dash = Dashboard::new().title("My Report");
        assert_eq!(dash.title, "My Report");
    }

    #[test]
    fn dashboard_output_dir_sets_dir() {
        let dash = Dashboard::new().output_dir("/tmp/test-output");
        assert_eq!(dash.output_dir, "/tmp/test-output");
    }

    #[test]
    fn dashboard_nav_style_sets_style() {
        let dash = Dashboard::new().nav_style(NavStyle::Vertical);
        assert_eq!(dash.nav_style, NavStyle::Vertical);
    }

    #[test]
    fn dashboard_add_df_stores_frame() {
        let mut df = df![
            "a" => [1i64, 2],
        ]
        .unwrap();
        let mut dash = Dashboard::new();
        dash.add_df("my_data", &mut df).unwrap();
        assert_eq!(dash.frames.len(), 1);
        assert_eq!(dash.frames[0].0, "my_data");
        assert!(!dash.frames[0].1.is_empty());
    }

    #[test]
    fn dashboard_add_df_multiple_keys() {
        let mut df1 = df!["a" => [1i64]].unwrap();
        let mut df2 = df!["b" => [2i64]].unwrap();
        let mut dash = Dashboard::new();
        dash.add_df("first", &mut df1).unwrap();
        dash.add_df("second", &mut df2).unwrap();
        assert_eq!(dash.frames.len(), 2);
        assert_eq!(dash.frames[0].0, "first");
        assert_eq!(dash.frames[1].0, "second");
    }

    #[test]
    fn dashboard_add_df_returns_self_for_chaining() {
        let mut df = df!["a" => [1i64]].unwrap();
        let mut dash = Dashboard::new();
        // add_df returns &mut Self, so multiple calls can be chained
        dash.add_df("k1", &mut df)
            .unwrap()
            .add_df("k2", &mut df)
            .unwrap();
        assert_eq!(dash.frames.len(), 2);
    }

    #[test]
    fn dashboard_add_page_stores_page() {
        use crate::charts::{ChartSpecBuilder, HBarConfig};
        use crate::pages::PageBuilder;

        let cfg = HBarConfig::builder()
            .category("c")
            .value("v")
            .x_label("X")
            .build()
            .unwrap();
        let page = PageBuilder::new("overview", "Overview", "Ov", 1)
            .chart(
                ChartSpecBuilder::hbar("Chart", "data", cfg)
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap();

        let mut dash = Dashboard::new();
        dash.add_page(page);
        assert_eq!(dash.pages.len(), 1);
        assert_eq!(dash.pages[0].slug, "overview");
    }

    #[test]
    fn dashboard_add_page_multiple() {
        use crate::charts::{ChartSpecBuilder, HBarConfig};
        use crate::pages::PageBuilder;

        let make_page = |slug: &str| {
            let cfg = HBarConfig::builder()
                .category("c")
                .value("v")
                .x_label("X")
                .build()
                .unwrap();
            PageBuilder::new(slug, "Title", "Label", 1)
                .chart(ChartSpecBuilder::hbar("C", "d", cfg).at(0, 0, 1).build())
                .build()
                .unwrap()
        };

        let mut dash = Dashboard::new();
        dash.add_page(make_page("page-one"));
        dash.add_page(make_page("page-two"));
        assert_eq!(dash.pages.len(), 2);
    }

    #[test]
    fn dashboard_output_dir_used_in_render_config() {
        // Verifies the builder chain correctly stores a custom output dir.
        // Full rendering requires Python; this just checks the configuration.
        let dash = Dashboard::new()
            .output_dir("/custom/path")
            .title("Test")
            .nav_style(NavStyle::Vertical);
        assert_eq!(dash.output_dir, "/custom/path");
        assert_eq!(dash.title, "Test");
        assert_eq!(dash.nav_style, NavStyle::Vertical);
    }
}
