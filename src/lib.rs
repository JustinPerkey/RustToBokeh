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
//! - [`error`] — The [`ChartError`] type used throughout the library.
//! - [`prelude`] — Convenience re-exports for common usage.

pub mod charts;
pub mod error;
pub mod modules;
pub mod pages;
pub mod prelude;
mod render;

pub use charts::{
    AxisConfig, AxisConfigBuilder, ChartConfig, ChartSpec, ChartSpecBuilder, FilterConfig,
    FilterSpec, GridCell, GroupedBarConfig, GroupedBarConfigBuilder, HBarConfig,
    HBarConfigBuilder, LineConfig, LineConfigBuilder, PaletteSpec, ScatterConfig,
    ScatterConfigBuilder, TimeScale, TooltipField, TooltipFormat, TooltipSpec, TooltipSpecBuilder,
    MAX_GRID_COLS,
};
pub use error::ChartError;
pub use modules::{
    ColumnFormat, PageModule, ParagraphSpec, ParagraphSpecBuilder, TableColumn, TableSpec,
    TableSpecBuilder,
};
pub use pages::{Page, PageBuilder};
pub use render::render_dashboard;
// compute_histogram is defined above; NavStyle is defined below.
// Both are re-exported via prelude.

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

/// Compute equal-width histogram statistics from a numeric DataFrame column.
///
/// Given a `DataFrame`, a column name, and the desired number of bins, this
/// function computes bin edges and returns a new `DataFrame` with five columns:
///
/// | Column  | Type | Description |
/// |---------|------|-------------|
/// | `left`  | f64  | Left edge of each bin |
/// | `right` | f64  | Right edge of each bin |
/// | `count` | f64  | Number of values that fall in each bin |
/// | `pdf`   | f64  | Probability density: `count / (n × bin_width)` |
/// | `cdf`   | f64  | Cumulative fraction of values up to each bin's right edge |
///
/// The result is intended to be registered with [`Dashboard::add_df`] and
/// referenced by a [`ChartSpecBuilder::histogram`](charts::ChartSpecBuilder::histogram)
/// spec. Use [`HistogramConfig`](charts::HistogramConfig) with
/// [`HistogramDisplay`](charts::HistogramDisplay) to choose which statistic
/// to render.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
/// use polars::prelude::*;
///
/// let raw = df!["salary" => [42.0f64, 65.0, 80.0, 95.0]].unwrap();
/// let mut hist = compute_histogram(&raw, "salary", 12)?;
/// dash.add_df("salary_hist", &mut hist)?;
/// ```
///
/// # Errors
///
/// Returns [`ChartError::Serialization`] if the column does not exist or
/// cannot be cast to `f64`.
pub fn compute_histogram(
    df: &DataFrame,
    column: &str,
    num_bins: usize,
) -> Result<DataFrame, ChartError> {
    use polars::prelude::*;

    let num_bins = num_bins.max(1);
    let series = df.column(column)?;
    let cast = series.cast(&DataType::Float64)?;
    let ca = cast.f64()?;
    let values: Vec<f64> = ca.iter().filter_map(|v| v).collect();

    if values.is_empty() {
        return Ok(df![
            "left"  => Vec::<f64>::new(),
            "right" => Vec::<f64>::new(),
            "count" => Vec::<f64>::new(),
            "pdf"   => Vec::<f64>::new(),
            "cdf"   => Vec::<f64>::new(),
        ]?);
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Guard against all-identical values to avoid zero-width bins.
    let (bin_min, bin_max) = if (max - min).abs() < f64::EPSILON {
        (min - 0.5, max + 0.5)
    } else {
        (min, max)
    };

    let bin_width = (bin_max - bin_min) / num_bins as f64;
    let mut counts = vec![0u64; num_bins];
    for &v in &values {
        let idx = ((v - bin_min) / bin_width).floor() as usize;
        counts[idx.min(num_bins - 1)] += 1;
    }

    let total = values.len() as f64;
    let left: Vec<f64> = (0..num_bins).map(|i| bin_min + i as f64 * bin_width).collect();
    let right: Vec<f64> = (0..num_bins).map(|i| bin_min + (i + 1) as f64 * bin_width).collect();
    let count_vals: Vec<f64> = counts.iter().map(|&c| c as f64).collect();
    let pdf: Vec<f64> = counts.iter().map(|&c| c as f64 / (total * bin_width)).collect();
    let mut cum = 0.0_f64;
    let cdf: Vec<f64> = counts
        .iter()
        .map(|&c| {
            cum += c as f64 / total;
            cum
        })
        .collect();

    Ok(df![
        "left"  => left,
        "right" => right,
        "count" => count_vals,
        "pdf"   => pdf,
        "cdf"   => cdf,
    ]?)
}

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

/// Configure the vendored Python so `PyO3` can find the interpreter, standard
/// library, and installed packages.
///
/// This is called automatically by [`render_dashboard`] and
/// [`Dashboard::render`]. It searches for a vendored Python installation in
/// several candidate directories relative to the current executable, and if
/// found, sets `PYTHONHOME`, `PYTHONPATH`, and `PATH` accordingly.
pub fn configure_vendored_python() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::Path::to_path_buf));

    let candidates = [
        exe_dir.as_ref().map(|d| d.join("../../vendor/python")),
        exe_dir.as_ref().map(|d| d.join("vendor/python")),
        Some(std::path::PathBuf::from("vendor/python")),
    ];

    for candidate in candidates.iter().flatten() {
        if let Ok(mut canon) = candidate.canonicalize() {
            if cfg!(windows) {
                let s = canon.to_string_lossy().to_string();
                if let Some(stripped) = s.strip_prefix(r"\\?\") {
                    canon = std::path::PathBuf::from(stripped);
                }
            }
            if canon.join("python.exe").exists() || canon.join("bin/python3").exists() {
                std::env::set_var("PYTHONHOME", &canon);

                let site_packages = if cfg!(windows) {
                    canon.join("Lib").join("site-packages")
                } else {
                    let lib = canon.join("lib");
                    std::fs::read_dir(&lib)
                        .ok()
                        .and_then(|mut entries| {
                            entries.find_map(|e| {
                                let name = e.ok()?.file_name().to_string_lossy().to_string();
                                name.starts_with("python3")
                                    .then(|| lib.join(name).join("site-packages"))
                            })
                        })
                        .unwrap_or_else(|| lib.join("python3").join("site-packages"))
                };
                std::env::set_var("PYTHONPATH", &site_packages);

                let path_var = std::env::var_os("PATH").unwrap_or_default();
                let mut paths = std::env::split_paths(&path_var).collect::<Vec<_>>();
                paths.insert(0, canon);
                if let Ok(new_path) = std::env::join_paths(&paths) {
                    std::env::set_var("PATH", &new_path);
                }
                return;
            }
        }
    }
}
