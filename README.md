# RustToBokeh

A demonstration of bridging Rust data processing with Python visualization. Rust builds a [Polars](https://pola.rs/) DataFrame, then hands the data to Python via [PyO3](https://pyo3.rs/), where [Bokeh](https://bokeh.org/) renders an interactive grouped bar chart and [Jinja2](https://jinja.palletsprojects.com/) produces the final HTML output.

## How It Works

```
Rust (Polars DataFrame)
        │
        ▼  PyO3 FFI
Python (Bokeh + Jinja2)
        │
        ▼
    output.html
```

1. **`src/main.rs`** — builds a 12-month revenue/expenses DataFrame with Polars, extracts the columns, and calls the embedded Python script via PyO3.
2. **`python/render.py`** — receives the data as Python lists, builds a grouped bar chart with Bokeh, and renders it into the HTML template.
3. **`templates/chart.html`** — Jinja2 template that wires up the Bokeh JS/CSS CDN resources and injects the chart script and div.

The Python script and HTML template are embedded into the binary at compile time using `include_str!`, so the final executable has no runtime file dependencies beyond a Python interpreter and the required Python packages.

## Prerequisites

- Rust toolchain (1.75+)
- `curl` or `wget` (for downloading Python)
- No Python installation required

## Setup

Run the vendor script once after cloning. It downloads a standalone Python build (no system Python needed) and installs the required pip packages:

```bash
bash scripts/setup_vendor.sh
```

This creates `vendor/python/` with a portable Python interpreter and writes `.cargo/config.toml` to point PyO3 at it.

### Offline builds

To make the project buildable on a machine with no internet access, comment out the `vendor/` line in `.gitignore` and commit the `vendor/python/` directory. This adds ~300 MB to the repo but allows cloning and building with zero downloads.

## Building & Running

```bash
cargo build --release
cargo run --release
```

On success the chart is written to **`output.html`** in the current directory. Open it in any browser to explore the interactive charts.

## Project Structure

```
RustToBokeh/
├── src/
│   └── main.rs           # Rust entry point — DataFrame + PyO3 bridge
├── python/
│   └── render.py         # Python script executed via PyO3
├── templates/
│   └── chart.html        # Jinja2 HTML template
├── requirements.txt      # Pinned Python dependencies
├── output.html           # Sample generated output (committed for preview)
├── Cargo.toml
└── Cargo.lock
```

## Dependencies

| Crate / Package | Version | Purpose |
|---|---|---|
| [pyo3](https://crates.io/crates/pyo3) | 0.23 | Rust ↔ Python FFI |
| [polars](https://crates.io/crates/polars) | 0.53 | DataFrame construction (Rust) |
| [bokeh](https://pypi.org/project/bokeh/) | see requirements.txt | Interactive chart rendering |
| [jinja2](https://pypi.org/project/Jinja2/) | see requirements.txt | HTML templating |
| [polars](https://pypi.org/project/polars/) | see requirements.txt | Arrow IPC deserialization (Python) |

## License

MIT — see [LICENSE](LICENSE).
