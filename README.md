# RustToBokeh

A Rust library for building interactive multi-page [Bokeh](https://bokeh.org/) dashboards. Data processing happens in Rust with [Polars](https://pola.rs/) DataFrames, then [PyO3](https://pyo3.rs/) bridges into Python where Bokeh renders the charts and [Jinja2](https://jinja.palletsprojects.com/) produces self-contained HTML files.

## How It Works

```
Rust (Polars DataFrames)
        │  serialize to Arrow IPC
        ▼  PyO3 FFI
Python (Bokeh + Jinja2)
        │  render charts, apply filters
        ▼
  output/*.html  (one file per page, with navigation)
```

1. **Build DataFrames** in Rust using Polars — one DataFrame per data source.
2. **Register data** with `Dashboard::add_df()`, which serializes each DataFrame to Arrow IPC bytes.
3. **Define pages** with `PageBuilder`, adding chart specs and optional interactive filters.
4. **Call `Dashboard::render()`** — PyO3 acquires the Python GIL, passes everything to the embedded `render.py`, and writes one HTML file per page.

The Python script and HTML template are embedded into the binary at compile time using `include_str!()`, so the final executable has no runtime file dependencies beyond a Python interpreter and the required Python packages.

## Quick Start

```rust
use rust_to_bokeh::prelude::*;
use polars::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut df = df![
        "month"    => ["Jan", "Feb", "Mar"],
        "revenue"  => [100.0, 150.0, 200.0f64],
        "expenses" => [80.0, 90.0, 110.0f64],
    ]?;

    let mut dash = Dashboard::new();
    dash.add_df("trends", &mut df)?;
    dash.add_page(
        PageBuilder::new("overview", "Overview", "Overview", 2)
            .chart(ChartSpecBuilder::line("Revenue vs Expenses", "trends",
                LineConfig::builder()
                    .x("month")
                    .y_cols(&["revenue", "expenses"])
                    .y_label("USD")
                    .build()?
            ).at(0, 0, 2).build())
            .build(),
    );
    dash.render()?;
    Ok(())
}
```

## Prerequisites

- Rust toolchain (1.75+)
- `curl` or `wget` (for downloading Python)
- No system Python installation required

## Setup

Run the vendor script once after cloning. It downloads a standalone Python build and installs the required pip packages:

```bash
bash scripts/setup_vendor.sh
```

This creates `vendor/python/` with a portable Python interpreter and writes `.cargo/config.toml` to point PyO3 at it.

### Alternative: System Python

If you prefer to use your own Python installation:

```bash
pip install -r requirements.txt
```

### Offline builds

To make the project buildable on a machine with no internet access, comment out the `vendor/` line in `.gitignore` and commit the `vendor/python/` directory. This adds ~300 MB to the repo but allows cloning and building with zero downloads.

## Building & Running

```bash
cargo build --release
cargo run --bin example-dashboard --release
```

On success the dashboard is written to `output/` in the current directory (one HTML file per page). Open any file in a browser to explore the interactive charts with cross-page navigation.

## Library Usage

Add `rust-to-bokeh` as a dependency in your `Cargo.toml` and import the prelude:

```rust
use rust_to_bokeh::prelude::*;
```

The prelude re-exports everything you need: `Dashboard`, all chart config types and their builders, `ChartSpecBuilder`, `PageBuilder`, `FilterSpec`, `FilterConfig`, `ChartError`, and utility functions.

### Supported Chart Types

| Type | Config | Builder | Description |
|---|---|---|---|
| Grouped bar | `GroupedBarConfig` | `GroupedBarConfig::builder()` | Vertical bars grouped by category |
| Multi-line | `LineConfig` | `LineConfig::builder()` | One or more line series on a shared axis |
| Horizontal bar | `HBarConfig` | `HBarConfig::builder()` | Horizontal bars for ranked/categorical data |
| Scatter plot | `ScatterConfig` | `ScatterConfig::builder()` | X-Y scatter with circle markers |

### Interactive Filters

Filters are added per-page and affect charts that share their data source. Charts must opt in by calling `.filtered()` on the `ChartSpecBuilder`.

| Filter | Factory Method | Widget | Description |
|---|---|---|---|
| Range | `FilterSpec::range(...)` | `RangeSlider` | Filter rows by numeric range |
| Select | `FilterSpec::select(...)` | Dropdown (with "All") | Filter by exact value match |
| Group | `FilterSpec::group(...)` | Dropdown | Bokeh `GroupFilter` (no "All" option) |
| Threshold | `FilterSpec::threshold(...)` | Toggle switch | Show rows above/below a value |
| Top N | `FilterSpec::top_n(...)` | Slider | Limit to top/bottom N rows |

Multiple filters on the same data source combine automatically via Bokeh's `IntersectionFilter`.

### Error Handling

All fallible operations return `Result<T, ChartError>`. The error type covers:

- **`MissingField`** — a required builder field was not set
- **`Serialization`** — Polars failed to serialize a DataFrame
- **`Python`** — Python raised an exception during rendering
- **`InvalidScript`** — the embedded script is malformed (should not occur in practice)

`ChartError` implements `From<PolarsError>` and `From<PyErr>`, so it works with `?` in functions returning `Result<T, ChartError>` or `Result<T, Box<dyn Error>>`.

## Project Structure

```
RustToBokeh/
├── src/
│   ├── lib.rs               # Library root: Dashboard builder, serialize_df()
│   ├── charts.rs             # Chart configs, builders, ChartSpec, FilterSpec
│   ├── pages.rs              # Page and PageBuilder
│   ├── error.rs              # ChartError enum
│   ├── render.rs             # PyO3 bridge to Python
│   ├── prelude.rs            # Convenience re-exports
│   └── bin/
│       └── example_dashboard.rs  # 20-page demo dashboard
├── python/
│   └── render.py             # Python renderer (embedded at compile time)
├── templates/
│   └── chart.html            # Jinja2 HTML template (embedded at compile time)
├── scripts/
│   └── setup_vendor.sh       # Downloads standalone Python into vendor/python/
├── build.rs                  # Copies vendored Python DLLs to target dir (Windows)
├── output/                   # Generated HTML output (committed for preview)
├── Cargo.toml
└── requirements.txt          # Pinned Python dependencies
```

## Dependencies

| Language | Crate / Package | Version | Purpose |
|---|---|---|---|
| Rust | [pyo3](https://crates.io/crates/pyo3) | 0.23 | Rust-Python FFI and GIL management |
| Rust | [polars](https://crates.io/crates/polars) | 0.53 | DataFrame construction and Arrow IPC serialization |
| Python | [bokeh](https://pypi.org/project/bokeh/) | 3.6.3 | Interactive chart rendering |
| Python | [polars](https://pypi.org/project/polars/) | 1.24.0 | Arrow IPC deserialization |
| Python | [jinja2](https://pypi.org/project/Jinja2/) | 3.1.6 | HTML template rendering |

## License

MIT — see [LICENSE](LICENSE).
