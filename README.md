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
- Python 3.8+ with the following packages:

```bash
pip install bokeh jinja2
```

PyO3 links against the Python interpreter at build time. Make sure the `python3` / `python` on your `PATH` is the one that has the packages installed, or set `PYO3_PYTHON` to point at a specific interpreter:

```bash
export PYO3_PYTHON=$(which python3)
```

## Building & Running

```bash
cargo build --release
cargo run --release
```

On success the chart is written to **`output.html`** in the current directory. Open it in any browser to explore the interactive chart.

## Project Structure

```
RustToBokeh/
├── src/
│   └── main.rs           # Rust entry point — DataFrame + PyO3 bridge
├── python/
│   └── render.py         # Python script executed via PyO3
├── templates/
│   └── chart.html        # Jinja2 HTML template
├── output.html           # Sample generated output (committed for preview)
├── Cargo.toml
└── Cargo.lock
```

## Dependencies

| Crate / Package | Version | Purpose |
|---|---|---|
| [pyo3](https://crates.io/crates/pyo3) | 0.23 | Rust ↔ Python FFI |
| [polars](https://crates.io/crates/polars) | 0.53 | DataFrame construction |
| [bokeh](https://pypi.org/project/bokeh/) | latest | Interactive chart rendering |
| [jinja2](https://pypi.org/project/Jinja2/) | latest | HTML templating |

## License

MIT — see [LICENSE](LICENSE).
