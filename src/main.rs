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

    // Walk up from the exe to find vendor/python/ relative to the project root.
    // When run via `cargo run`, the exe is in target/release/ or target/debug/.
    // When run standalone, vendor/python/ should be next to the exe or in the cwd.
    let candidates = [
        exe_dir
            .as_ref()
            .map(|d| d.join("../../vendor/python")),
        exe_dir.as_ref().map(|d| d.join("vendor/python")),
        Some(std::path::PathBuf::from("vendor/python")),
    ];

    for candidate in candidates.iter().flatten() {
        if let Ok(mut canon) = candidate.canonicalize() {
            // On Windows, canonicalize() returns \\?\ extended-length paths which
            // confuse the embedded Python interpreter. Strip the prefix.
            if cfg!(windows) {
                let s = canon.to_string_lossy().to_string();
                if let Some(stripped) = s.strip_prefix(r"\\?\") {
                    canon = std::path::PathBuf::from(stripped);
                }
            }
            if canon.join("python.exe").exists() || canon.join("bin/python3").exists() {
                // Set PYTHONHOME so the embedded interpreter finds its stdlib
                std::env::set_var("PYTHONHOME", &canon);

                // Set PYTHONPATH so installed packages (site-packages) are importable
                let site_packages = if cfg!(windows) {
                    canon.join("Lib").join("site-packages")
                } else {
                    // Unix install_only layout: lib/python3.XX/site-packages
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

                // Prepend to PATH so DLLs can be found at runtime
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
}

impl ChartType {
    fn as_str(&self) -> &'static str {
        match self {
            ChartType::GroupedBar => "grouped_bar",
        }
    }
}

struct ChartSpec {
    title: String,
    chart_type: ChartType,
    bytes: Vec<u8>,
    x_col: String,
    group_col: String,
    value_col: String,
    y_label: String,
}

fn build_monthly_dataframe() -> DataFrame {
    df![
        "month" => [
            "Jan","Jan","Feb","Feb","Mar","Mar","Apr","Apr",
            "May","May","Jun","Jun","Jul","Jul","Aug","Aug",
            "Sep","Sep","Oct","Oct","Nov","Nov","Dec","Dec"
        ],
        "category" => [
            "Revenue","Expenses","Revenue","Expenses","Revenue","Expenses",
            "Revenue","Expenses","Revenue","Expenses","Revenue","Expenses",
            "Revenue","Expenses","Revenue","Expenses","Revenue","Expenses",
            "Revenue","Expenses","Revenue","Expenses","Revenue","Expenses"
        ],
        "value" => [
            120.5, 95.0,  135.2, 102.5, 148.7, 110.3, 162.3, 118.7,
            175.0, 125.2, 190.8, 132.8, 205.1, 140.1, 198.4, 136.5,
            210.7, 145.2, 225.3, 152.7, 240.6, 160.3, 280.9, 175.5f64
        ]
    ]
    .expect("Failed to build monthly DataFrame")
}

fn build_quarterly_dataframe() -> DataFrame {
    df![
        "quarter" => ["Q1","Q1","Q1","Q2","Q2","Q2","Q3","Q3","Q3","Q4","Q4","Q4"],
        "product" => [
            "Product A","Product B","Product C",
            "Product A","Product B","Product C",
            "Product A","Product B","Product C",
            "Product A","Product B","Product C"
        ],
        "value" => [
            320.5, 210.0, 140.3,
            410.2, 275.8, 165.0,
            390.7, 305.3, 195.5,
            520.1, 380.6, 240.9f64
        ]
    ]
    .expect("Failed to build quarterly DataFrame")
}

fn main() -> PyResult<()> {
    configure_vendored_python();

    let mut monthly_df = build_monthly_dataframe();
    let mut quarterly_df = build_quarterly_dataframe();

    println!("Monthly DataFrame:\n{}", monthly_df);
    println!("Quarterly DataFrame:\n{}", quarterly_df);

    let specs: Vec<ChartSpec> = vec![
        ChartSpec {
            title: "Monthly Revenue vs Expenses (2024)".to_string(),
            chart_type: ChartType::GroupedBar,
            bytes: serialize_df(&mut monthly_df),
            x_col: "month".to_string(),
            group_col: "category".to_string(),
            value_col: "value".to_string(),
            y_label: "Amount (USD thousands)".to_string(),
        },
        ChartSpec {
            title: "Quarterly Product Revenue".to_string(),
            chart_type: ChartType::GroupedBar,
            bytes: serialize_df(&mut quarterly_df),
            x_col: "quarter".to_string(),
            group_col: "product".to_string(),
            value_col: "value".to_string(),
            y_label: "Revenue (USD thousands)".to_string(),
        },
    ];

    let python_script = include_str!("../python/render.py");
    let html_template = include_str!("../templates/chart.html");

    Python::with_gil(|py| {
        let chart_specs = PyList::empty(py);
        for spec in &specs {
            let d = PyDict::new(py);
            d.set_item("title", &spec.title)?;
            d.set_item("chart_type", spec.chart_type.as_str())?;
            d.set_item("bytes", PyBytes::new(py, &spec.bytes))?;
            d.set_item("x_col", &spec.x_col)?;
            d.set_item("group_col", &spec.group_col)?;
            d.set_item("value_col", &spec.value_col)?;
            d.set_item("y_label", &spec.y_label)?;
            chart_specs.append(d)?;
        }

        let locals = PyDict::new(py);
        locals.set_item("chart_specs", chart_specs)?;
        locals.set_item("html_template", html_template)?;
        locals.set_item("output_path", "output.html")?;

        let code = CString::new(python_script).expect("Python script contains null byte");
        // Pass locals as both globals and locals so list comprehensions can see
        // script-defined variables (Python 3 comprehensions have their own scope).
        py.run(code.as_c_str(), Some(&locals), Some(&locals))?;

        println!("Charts saved to output.html");
        Ok(())
    })
}
