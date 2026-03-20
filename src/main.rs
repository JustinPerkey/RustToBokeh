use polars::io::ipc::IpcWriter;
use polars::io::SerWriter;
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::ffi::CString;
use std::io::Cursor;

/// Configure the vendored Python so PyO3 can find the interpreter, standard
/// library, and installed packages. Must run before any PyO3 call.
fn configure_vendored_python() {
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
                                name.starts_with("python3").then(|| lib.join(name).join("site-packages"))
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

fn serialize_df(df: &mut DataFrame) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    IpcWriter::new(&mut buf)
        .finish(df)
        .expect("Failed to serialize DataFrame");
    buf.into_inner()
}

enum ChartType {
    GroupedBar,
    LineMulti,
    HBar,
    ScatterPlot,
}

impl ChartType {
    fn as_str(&self) -> &'static str {
        match self {
            ChartType::GroupedBar => "grouped_bar",
            ChartType::LineMulti  => "line_multi",
            ChartType::HBar       => "hbar",
            ChartType::ScatterPlot => "scatter_plot",
        }
    }
}

/// Describes a single chart panel. `source_key` links to an entry in the
/// `frames` dict; charts sharing the same key share one ColumnDataSource,
/// enabling linked hover/selection in the browser.
struct ChartSpec {
    title: String,
    chart_type: ChartType,
    /// Must match a key in the `frames` dict passed to Python.
    source_key: String,
    /// For GroupedBar/LineMulti/HBar: categorical axis column.
    /// For ScatterPlot: numeric x column.
    x_col: String,
    /// Column names for the value series (wide-format DataFrame).
    value_cols: Vec<String>,
    y_label: String,
    /// Layout hint for Jinja: charts wider than 700 span the full grid row.
    width: u32,
    height: u32,
    /// Optional static row filter. Ignored when the page has `has_filter = true`
    /// (the interactive RangeSlider overrides it for that source_key).
    indices: Option<Vec<usize>>,
}

/// A page groups chart specs that share a single HTML output file.
/// Only the frames referenced by that page's specs are embedded in its output.
struct Page {
    title: String,
    /// Short label shown in the nav bar on every page.
    nav_label: String,
    /// Output filename without extension; page writes to `output_dir/slug.html`.
    slug: String,
    /// When true, Python adds a RangeSlider whose CustomJS updates a shared
    /// IndexFilter on all charts using the first spec's source_key,
    /// demonstrating that both charts filter from a single CDS.
    has_filter: bool,
    specs: Vec<ChartSpec>,
}

// ── DataFrames ───────────────────────────────────────────────────────────────

/// Wide format: one row per month, one column per series.
fn build_monthly_dataframe() -> DataFrame {
    df![
        "month"    => ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"],
        "Revenue"  => [120.5,135.2,148.7,162.3,175.0,190.8,205.1,198.4,210.7,225.3,240.6,280.9f64],
        "Expenses" => [ 95.0,102.5,110.3,118.7,125.2,132.8,140.1,136.5,145.2,152.7,160.3,175.5f64],
    ]
    .expect("Failed to build monthly DataFrame")
}

/// Wide format: one row per quarter, one column per product.
fn build_quarterly_dataframe() -> DataFrame {
    df![
        "quarter"   => ["Q1","Q2","Q3","Q4"],
        "Product A" => [320.5, 410.2, 390.7, 520.1f64],
        "Product B" => [210.0, 275.8, 305.3, 380.6f64],
        "Product C" => [140.3, 165.0, 195.5, 240.9f64],
    ]
    .expect("Failed to build quarterly DataFrame")
}

fn main() -> PyResult<()> {
    configure_vendored_python();

    let mut monthly_df = build_monthly_dataframe();
    let mut quarterly_df = build_quarterly_dataframe();

    println!("Monthly DataFrame:\n{}", monthly_df);
    println!("Quarterly DataFrame:\n{}", quarterly_df);

    // `monthly_corr` is a separate frame key from `monthly` so the correlation
    // page's HTML embeds only its own data, not the monthly page's CDS.
    let frame_data: Vec<(&str, Vec<u8>)> = vec![
        ("monthly",      serialize_df(&mut monthly_df)),
        ("quarterly",    serialize_df(&mut quarterly_df)),
        ("monthly_corr", serialize_df(&mut monthly_df)),
    ];

    let pages: Vec<Page> = vec![
        // ── Page 1: shared CDS + interactive RangeSlider filter ─────────────
        // Both charts reference source_key="monthly" → same CDS object.
        // The RangeSlider drives a shared IndexFilter; moving the slider
        // filters both charts simultaneously from one source.
        Page {
            title: "Monthly Performance".to_string(),
            nav_label: "Monthly".to_string(),
            slug: "monthly".to_string(),
            has_filter: true,
            specs: vec![
                ChartSpec {
                    title: "Revenue vs Expenses by Month".to_string(),
                    chart_type: ChartType::GroupedBar,
                    source_key: "monthly".to_string(),
                    x_col: "month".to_string(),
                    value_cols: vec!["Revenue".to_string(), "Expenses".to_string()],
                    y_label: "Amount (USD thousands)".to_string(),
                    width: 900,
                    height: 380,
                    indices: None,
                },
                ChartSpec {
                    title: "Revenue vs Expenses Scatter".to_string(),
                    chart_type: ChartType::ScatterPlot,
                    source_key: "monthly".to_string(),
                    x_col: "Revenue".to_string(),
                    value_cols: vec!["Expenses".to_string()],
                    y_label: "Expenses (USD thousands)".to_string(),
                    width: 500,
                    height: 380,
                    indices: None,
                },
            ],
        },

        // ── Page 2: shared CDS + BoxSelectTool / TapTool linked selection ───
        // Both charts reference source_key="quarterly" → same CDS object.
        // Selecting a quarter bar in one chart immediately highlights the
        // corresponding row in the other; no CustomJS required.
        Page {
            title: "Quarterly Breakdown".to_string(),
            nav_label: "Quarterly".to_string(),
            slug: "quarterly".to_string(),
            has_filter: false,
            specs: vec![
                ChartSpec {
                    title: "Quarterly Product Revenue (select to link)".to_string(),
                    chart_type: ChartType::GroupedBar,
                    source_key: "quarterly".to_string(),
                    x_col: "quarter".to_string(),
                    value_cols: vec![
                        "Product A".to_string(),
                        "Product B".to_string(),
                        "Product C".to_string(),
                    ],
                    y_label: "Revenue (USD thousands)".to_string(),
                    width: 900,
                    height: 380,
                    indices: None,
                },
                ChartSpec {
                    title: "Product A Revenue by Quarter (linked)".to_string(),
                    chart_type: ChartType::HBar,
                    source_key: "quarterly".to_string(),
                    x_col: "quarter".to_string(),
                    value_cols: vec!["Product A".to_string()],
                    y_label: "Product A Revenue (USD thousands)".to_string(),
                    width: 500,
                    height: 300,
                    indices: None,
                },
            ],
        },

        // ── Page 3: shared CDS + LassoSelect reveals month identities ───────
        // Both charts reference source_key="monthly_corr" → same CDS object.
        // Lasso-selecting a cluster in the scatter (by Revenue/Expenses values)
        // highlights those same rows (months) in the bar chart, revealing
        // which months form that cluster.
        Page {
            title: "Revenue Correlation".to_string(),
            nav_label: "Correlation".to_string(),
            slug: "correlation".to_string(),
            has_filter: false,
            specs: vec![
                ChartSpec {
                    title: "Revenue vs Expenses Scatter (lasso to link)".to_string(),
                    chart_type: ChartType::ScatterPlot,
                    source_key: "monthly_corr".to_string(),
                    x_col: "Revenue".to_string(),
                    value_cols: vec!["Expenses".to_string()],
                    y_label: "Expenses (USD thousands)".to_string(),
                    width: 500,
                    height: 400,
                    indices: None,
                },
                ChartSpec {
                    title: "Revenue vs Expenses by Month (linked)".to_string(),
                    chart_type: ChartType::GroupedBar,
                    source_key: "monthly_corr".to_string(),
                    x_col: "month".to_string(),
                    value_cols: vec!["Revenue".to_string(), "Expenses".to_string()],
                    y_label: "Amount (USD thousands)".to_string(),
                    width: 900,
                    height: 380,
                    indices: None,
                },
            ],
        },
    ];

    let python_script = include_str!("../python/render.py");
    let html_template = include_str!("../templates/chart.html");

    Python::with_gil(|py| {
        // frames dict: source_key -> Arrow IPC bytes (all pages combined)
        let frames = PyDict::new(py);
        for (key, bytes) in &frame_data {
            frames.set_item(*key, PyBytes::new(py, bytes))?;
        }

        // pages list: one dict per page, each containing its own specs list
        let py_pages = PyList::empty(py);
        for page in &pages {
            let page_dict = PyDict::new(py);
            page_dict.set_item("title", &page.title)?;
            page_dict.set_item("nav_label", &page.nav_label)?;
            page_dict.set_item("slug", &page.slug)?;
            page_dict.set_item("has_filter", page.has_filter)?;

            let page_specs = PyList::empty(py);
            for spec in &page.specs {
                let d = PyDict::new(py);
                d.set_item("title", &spec.title)?;
                d.set_item("chart_type", spec.chart_type.as_str())?;
                d.set_item("source_key", &spec.source_key)?;
                d.set_item("x_col", &spec.x_col)?;
                let value_cols = PyList::empty(py);
                for col in &spec.value_cols {
                    value_cols.append(col.as_str())?;
                }
                d.set_item("value_cols", value_cols)?;
                d.set_item("y_label", &spec.y_label)?;
                d.set_item("width", spec.width)?;
                d.set_item("height", spec.height)?;
                match &spec.indices {
                    Some(idx) => {
                        let py_idx = PyList::empty(py);
                        for &i in idx {
                            py_idx.append(i)?;
                        }
                        d.set_item("indices", py_idx)?;
                    }
                    None => d.set_item("indices", py.None())?,
                }
                page_specs.append(d)?;
            }
            page_dict.set_item("specs", page_specs)?;
            py_pages.append(page_dict)?;
        }

        let locals = PyDict::new(py);
        locals.set_item("frames", frames)?;
        locals.set_item("pages", py_pages)?;
        locals.set_item("html_template", html_template)?;
        locals.set_item("output_dir", "output")?;

        let code = CString::new(python_script).expect("Python script contains null byte");
        py.run(code.as_c_str(), Some(&locals), Some(&locals))?;

        println!("Pages written to output/");
        Ok(())
    })
}
