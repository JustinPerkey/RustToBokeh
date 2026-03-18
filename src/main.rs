use polars::io::ipc::IpcWriter;
use polars::io::SerWriter;
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::ffi::CString;
use std::io::Cursor;

fn serialize_df(df: &mut DataFrame) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    IpcWriter::new(&mut buf)
        .finish(df)
        .expect("Failed to serialize DataFrame");
    buf.into_inner()
}

fn build_monthly_dataframe() -> DataFrame {
    df![
        "month" => [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
        ],
        "revenue" => [
            120.5, 135.2, 148.7, 162.3, 175.0, 190.8,
            205.1, 198.4, 210.7, 225.3, 240.6, 280.9f64
        ],
        "expenses" => [
            95.0, 102.5, 110.3, 118.7, 125.2, 132.8,
            140.1, 136.5, 145.2, 152.7, 160.3, 175.5f64
        ]
    ]
    .expect("Failed to build monthly DataFrame")
}

fn build_quarterly_dataframe() -> DataFrame {
    df![
        "quarter"   => ["Q1", "Q2", "Q3", "Q4"],
        "product_a" => [320.5, 410.2, 390.7, 520.1f64],
        "product_b" => [210.0, 275.8, 305.3, 380.6f64],
        "product_c" => [140.3, 165.0, 195.5, 240.9f64]
    ]
    .expect("Failed to build quarterly DataFrame")
}

fn main() -> PyResult<()> {
    let mut monthly_df = build_monthly_dataframe();
    let mut quarterly_df = build_quarterly_dataframe();

    println!("Monthly DataFrame:\n{}", monthly_df);
    println!("Quarterly DataFrame:\n{}", quarterly_df);

    // Serialize each DataFrame to Arrow IPC bytes
    let monthly_bytes = serialize_df(&mut monthly_df);
    let quarterly_bytes = serialize_df(&mut quarterly_df);

    println!(
        "Serialized monthly frame: {} bytes, quarterly frame: {} bytes",
        monthly_bytes.len(),
        quarterly_bytes.len()
    );

    let python_script = include_str!("../python/render.py");
    let html_template = include_str!("../templates/chart.html");

    Python::with_gil(|py| {
        // Build frames dict: name -> Arrow IPC bytes
        let frames = PyDict::new(py);
        frames.set_item("monthly", PyBytes::new(py, &monthly_bytes))?;
        frames.set_item("quarterly", PyBytes::new(py, &quarterly_bytes))?;

        let locals = PyDict::new(py);
        locals.set_item("frames", frames)?;
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
