//! # RustToBokeh
//!
//! A library for building interactive multi-page [Bokeh](https://bokeh.org/)
//! dashboards from Rust, using [Polars](https://pola.rs/) DataFrames for data
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
//!             .build(),
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
//! 1. **Build DataFrames** in Rust using Polars.
//! 2. **Register data** with [`Dashboard::add_df`], which serializes each
//!    DataFrame to Arrow IPC bytes.
//! 3. **Define pages** with [`PageBuilder`], adding [`ChartSpec`](charts::ChartSpec)s
//!    and optional [`FilterSpec`](charts::FilterSpec)s.
//! 4. **Call [`Dashboard::render`]**, which acquires the Python GIL via PyO3,
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
    ChartConfig, ChartSpec, ChartSpecBuilder, FilterConfig, FilterSpec, GridCell,
    GroupedBarConfig, GroupedBarConfigBuilder,
    HBarConfig, HBarConfigBuilder,
    LineConfig, LineConfigBuilder,
    ScatterConfig, ScatterConfigBuilder,
};
pub use error::ChartError;
pub use modules::{
    ColumnFormat, PageModule,
    ParagraphSpec, ParagraphSpecBuilder,
    TableColumn, TableSpec, TableSpecBuilder,
};
pub use pages::{Page, PageBuilder};
pub use render::render_dashboard;

use polars::io::ipc::IpcWriter;
use polars::io::SerWriter;
use polars::prelude::DataFrame;
use std::io::Cursor;

/// Serialize a Polars DataFrame to Arrow IPC bytes.
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

/// High-level dashboard builder that collects DataFrames and pages, then
/// renders everything in one call.
///
/// # Workflow
///
/// 1. Create a dashboard with [`Dashboard::new`].
/// 2. Optionally set the output directory with [`output_dir`](Dashboard::output_dir)
///    (defaults to `"output"`).
/// 3. Register DataFrames with [`add_df`](Dashboard::add_df). Each DataFrame
///    is serialized immediately and stored under the given key.
/// 4. Add pages with [`add_page`](Dashboard::add_page). Charts on each page
///    reference DataFrames by their registered key.
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
///         .build(),
/// );
/// dash.render()?;
/// ```
pub struct Dashboard {
    frames: Vec<(String, Vec<u8>)>,
    pages: Vec<Page>,
    output_dir: String,
}

impl Dashboard {
    /// Create an empty dashboard with the default output directory (`"output"`).
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            pages: Vec::new(),
            output_dir: "output".into(),
        }
    }

    /// Set the output directory for generated HTML files.
    ///
    /// Defaults to `"output"`. The directory is created automatically by the
    /// Python renderer if it does not exist.
    pub fn output_dir(mut self, dir: &str) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Register a DataFrame under the given key.
    ///
    /// The DataFrame is serialized to Arrow IPC bytes immediately. Charts
    /// reference this data by using the same `key` as their `source_key`.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::Serialization`] if the DataFrame cannot be
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
    /// This acquires the Python GIL, passes all serialized DataFrames and
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
        render_dashboard(&refs, &self.pages, &self.output_dir)
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

/// Configure the vendored Python so PyO3 can find the interpreter, standard
/// library, and installed packages.
///
/// This is called automatically by [`render_dashboard`] and
/// [`Dashboard::render`]. It searches for a vendored Python installation in
/// several candidate directories relative to the current executable, and if
/// found, sets `PYTHONHOME`, `PYTHONPATH`, and `PATH` accordingly.
pub fn configure_vendored_python() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    let candidates = [
        exe_dir
            .as_ref()
            .map(|d| d.join("../../vendor/python")),
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
