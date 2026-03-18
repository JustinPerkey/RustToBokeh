use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::ffi::CString;

fn build_dataframe() -> DataFrame {
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
    .expect("Failed to build DataFrame")
}

fn main() -> PyResult<()> {
    let df = build_dataframe();
    println!("DataFrame:\n{}", df);

    // Extract month column as Vec<String>
    let months: Vec<String> = df
        .column("month")
        .unwrap()
        .str()
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap().to_string())
        .collect();

    // Extract revenue column as Vec<f64>
    let revenue: Vec<f64> = df
        .column("revenue")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap())
        .collect();

    // Extract expenses column as Vec<f64>
    let expenses: Vec<f64> = df
        .column("expenses")
        .unwrap()
        .f64()
        .unwrap()
        .into_iter()
        .map(|v| v.unwrap())
        .collect();

    // Embed Python render script and Jinja2 template at compile time
    let python_script = include_str!("../python/render.py");
    let html_template = include_str!("../templates/chart.html");

    Python::with_gil(|py| {
        let locals = PyDict::new(py);
        locals.set_item("months", PyList::new(py, &months)?)?;
        locals.set_item("revenue", PyList::new(py, &revenue)?)?;
        locals.set_item("expenses", PyList::new(py, &expenses)?)?;
        locals.set_item("html_template", html_template)?;
        locals.set_item("output_path", "output.html")?;

        let code = CString::new(python_script).expect("Python script contains null byte");
        // Pass locals as both globals and locals so list comprehensions can see
        // script-defined variables (Python 3 comprehensions have their own scope).
        py.run(code.as_c_str(), Some(&locals), Some(&locals))?;

        println!("Chart saved to output.html");
        Ok(())
    })
}
