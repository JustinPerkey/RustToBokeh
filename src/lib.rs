pub mod charts;
pub mod pages;
mod render;

pub use charts::{
    ChartConfig, ChartSpecBuilder, FilterConfig, FilterSpec, GridCell,
    GroupedBarConfig, GroupedBarConfigBuilder,
    HBarConfig, HBarConfigBuilder,
    LineConfig, LineConfigBuilder,
    ScatterConfig, ScatterConfigBuilder,
};
pub use pages::{Page, PageBuilder};
pub use render::render_dashboard;

use polars::io::ipc::IpcWriter;
use polars::io::SerWriter;
use polars::prelude::DataFrame;
use std::io::Cursor;

/// Serialize a Polars DataFrame to Arrow IPC bytes for passing to the renderer.
pub fn serialize_df(df: &mut DataFrame) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    IpcWriter::new(&mut buf)
        .finish(df)
        .expect("Failed to serialize DataFrame");
    buf.into_inner()
}

/// High-level dashboard builder that collects DataFrames and pages, then
/// renders everything in one call.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::*;
/// use polars::prelude::*;
///
/// let mut df = df!["x" => [1, 2, 3], "y" => [4, 5, 6]].unwrap();
///
/// Dashboard::new()
///     .add_df("my_data", &mut df)
///     .add_page(
///         PageBuilder::new("overview", "Overview", "Overview", 2)
///             .chart(ChartSpecBuilder::scatter("X vs Y", "my_data",
///                 ScatterConfig::builder()
///                     .x("x").y("y").x_label("X").y_label("Y")
///                     .build()
///             ).at(0, 0, 2).build())
///             .build(),
///     )
///     .render()
///     .expect("render failed");
/// ```
pub struct Dashboard {
    frames: Vec<(String, Vec<u8>)>,
    pages: Vec<Page>,
    output_dir: String,
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            pages: Vec::new(),
            output_dir: "output".into(),
        }
    }

    /// Set the output directory (default: `"output"`).
    pub fn output_dir(mut self, dir: &str) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Add a DataFrame, serializing it to Arrow IPC bytes under the given key.
    pub fn add_df(&mut self, key: &str, df: &mut DataFrame) -> &mut Self {
        self.frames.push((key.into(), serialize_df(df)));
        self
    }

    /// Add a pre-built Page to the dashboard.
    pub fn add_page(&mut self, page: Page) -> &mut Self {
        self.pages.push(page);
        self
    }

    /// Render all pages to HTML files in the output directory.
    pub fn render(&self) -> pyo3::PyResult<()> {
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
/// library, and installed packages. Called automatically by [`render_dashboard`]
/// and [`Dashboard::render`].
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
