# CLAUDE.md — AI Assistant Guide for RustToBokeh

## Project Overview

RustToBokeh is a demonstration of seamless Rust ↔ Python interoperability for data pipeline workflows. It uses Rust (with Polars) for data processing and Python (with Bokeh) for interactive visualization, bridged via PyO3.

**Core idea**: Rust builds DataFrames → serializes to Arrow IPC binary → passes to embedded Python → Python renders multi-page interactive Bokeh dashboards → outputs self-contained HTML files with cross-chart linking.

---

## Repository Structure

```
RustToBokeh/
├── src/
│   └── main.rs              # Rust entry point; builds DataFrames, defines Pages/ChartSpecs, calls Python via PyO3
├── python/
│   └── render.py            # Python script; deserializes data, renders multi-page Bokeh dashboards
├── templates/
│   └── chart.html           # Jinja2 HTML template with navigation bar and responsive grid layout
├── scripts/
│   └── setup_vendor.sh      # Downloads standalone Python into vendor/python/
├── build.rs                 # Copies vendored Python DLLs to target dir on Windows
├── .cargo/
│   └── config.toml          # Sets PYO3_PYTHON to vendored interpreter
├── Cargo.toml               # Rust package manifest
├── Cargo.lock               # Pinned dependency versions (do not edit manually)
├── requirements.txt         # Pinned Python dependencies (bokeh, jinja2, polars)
├── output/                  # Generated multi-page HTML output (committed for preview)
│   ├── monthly.html
│   ├── quarterly.html
│   └── correlation.html
├── output.html              # Legacy single-page output (kept for backwards compatibility)
├── vendor/                  # Vendored standalone Python (gitignored, created by setup_vendor.sh)
├── README.md                # User-facing setup and usage documentation
└── LICENSE                  # MIT License
```

---

## Architecture and Data Flow

```
[Rust - src/main.rs]
  │  Build Polars DataFrames (wide format: one row per category, one column per series)
  │  Serialize to Arrow IPC binary (Vec<u8>)
  │  Define Pages and ChartSpecs (chart type, source key, columns, layout)
  │  Embed python/render.py and templates/chart.html at compile time via include_str!()
  ↓
[PyO3 Bridge]
  │  Acquire Python GIL (Python::with_gil)
  │  Inject: { frames, pages, html_template, output_dir }
  │  Execute render.py in that context
  ↓
[Python - python/render.py]
  │  Deserialize Arrow IPC bytes → Polars DataFrames
  │  Build charts from ChartSpec dicts (grouped bar, line, hbar, scatter)
  │  Line/scatter charts sharing the same source_key share one ColumnDataSource (linked selection)
  │  Build Bokeh filter objects (BooleanFilter, GroupFilter, IndexFilter) from FilterSpecs
  │  Combine filters via IntersectionFilter → CDSView on filtered chart renderers
  │  Create widgets (RangeSlider, Select, Switch, Slider) with CustomJS callbacks
  │  Render each Page to its own HTML file via Jinja2
  │  Write output/<slug>.html files with inter-page navigation
```

Key architectural concepts:
- `include_str!()` embeds `render.py` and `chart.html` as string literals at **compile time** — no file I/O needed at runtime for these resources.
- **ChartSpec**: declarative chart definition (type, data source key, columns, dimensions, `filtered` flag). Defined in Rust, consumed by Python.
- **FilterSpec**: declarative filter definition (source_key, column, label, `FilterConfig` variant). Defined in Rust per-page, consumed by Python to build Bokeh filter objects and widgets.
- **FilterConfig** enum: `Range` (RangeSlider → BooleanFilter), `Select` (dropdown with "All" → BooleanFilter), `Group` (dropdown → GroupFilter), `Threshold` (Switch toggle → BooleanFilter), `TopN` (Slider → IndexFilter).
- **Page**: groups ChartSpecs + FilterSpecs into a single HTML file. Each page embeds only the data it needs.
- **Shared ColumnDataSource**: line/scatter charts on the same page that reference the same `source_key` share one flat CDS, enabling linked hover/selection. Grouped bar and hbar use chart-type-specific CDS shapes.
- **CDSView filtering**: filtered charts receive a `CDSView` with combined Bokeh filter objects (via `IntersectionFilter` when multiple filters target the same source). Widgets update filter properties via `CustomJS` callbacks.

---

## Build and Run

### Prerequisites

- Rust toolchain (edition 2021, Rust 1.75+)

### Setup (Vendored Python — recommended)

The project vendors a standalone Python interpreter so no system Python is required:

```bash
bash scripts/setup_vendor.sh
```

This downloads a portable Python into `vendor/python/` and installs dependencies from `requirements.txt`. The `.cargo/config.toml` points `PYO3_PYTHON` at this vendored interpreter. On Windows, `build.rs` copies the required DLLs to the target directory automatically.

### Alternative: System Python

If not using vendored Python, install dependencies manually:

```bash
pip install -r requirements.txt
```

### Build and Run

```bash
cargo build --release
cargo run --release
```

This produces HTML files in the `output/` directory (one per page).

---

## Key Dependencies

| Language | Crate/Package | Version | Purpose |
|----------|--------------|---------|---------|
| Rust | `pyo3` | 0.23 | Rust ↔ Python FFI, GIL management |
| Rust | `polars` | 0.53 | DataFrame creation, Arrow IPC serialization |
| Python | `bokeh` | 3.6.3 | Interactive chart generation |
| Python | `polars` | 1.24.0 | Arrow IPC deserialization |
| Python | `jinja2` | 3.1.6 | HTML template rendering |

Polars features enabled in `Cargo.toml`: `lazy`, `ipc`, `parquet`.
PyO3 feature: `auto-initialize` (Python interpreter auto-initialized by Rust).
Python versions are pinned in `requirements.txt`.

---

## Code Conventions

### Rust (`src/main.rs`)

- Use Polars `df!` macro for DataFrame construction in **wide format** (one row per category, one column per series).
- Serialize DataFrames with `IpcWriter` writing into a `std::io::Cursor`.
- Define charts declaratively using `ChartSpec` and group them into `Page` structs.
- Line/scatter charts sharing a `source_key` within a page share a single flat `ColumnDataSource` in the browser.
- Define filters declaratively using `FilterSpec` with a `FilterConfig` enum variant (`Range`, `Select`, `Group`, `Threshold`, `TopN`).
- Mark charts as `filtered: true` to opt them into CDSView-based filtering.
- Pass data to Python as PyO3 dicts/lists (not `HashMap`).
- Use `.expect()` for error handling (acceptable for this demo; update to `?` propagation if error handling is needed in production extensions).
- Imports grouped by crate: `polars`, then `pyo3`, then `std`.

**Supported chart types** (`ChartType` enum): `GroupedBar`, `LineMulti`, `HBar`, `ScatterPlot`.

**Pattern for adding a new chart to an existing page:**
1. If needed, add a `build_*_dataframe()` function and serialize it into the `frame_data` vec.
2. Add a `ChartSpec` to the relevant `Page`'s `specs` vec, referencing the correct `source_key`.
3. Python's `render.py` handles the rest generically — no Python changes needed unless adding a new chart type.

**Pattern for adding a new page:**
1. Add a `Page` struct to the `pages` vec with a unique `slug`, `nav_label`, and its `ChartSpec`s.
2. Ensure referenced `source_key`s exist in `frame_data`.
3. Navigation is generated automatically by the template.

### Python (`python/render.py`)

- Script-style execution with helper functions — data arrives via injected local variables.
- Available variables at runtime: `frames` (dict of `str → bytes`), `pages` (list of page dicts), `nav_links` (list), `html_template` (str), `output_dir` (str).
- Deserialize frames: `polars.read_ipc(io.BytesIO(frames["key"]))`.
- Chart rendering is driven by ChartSpec dicts — the Python code is generic, not per-chart.
- Uses Bokeh's `ColumnDataSource`, `CDSView`, `FactorRange`, `factor_cmap()`, and `CustomJS` for interactivity.
- Filtering uses Bokeh native filter models: `BooleanFilter`, `GroupFilter`, `IndexFilter`, combined via `IntersectionFilter`.
- Use `bokeh.embed.components()` for embedding and Bokeh CDN for JS/CSS resources.

### HTML Template (`templates/chart.html`)

- Jinja2 template with responsive CSS grid layout and inter-page navigation bar.
- Receives: `bokeh_js_urls` (list of CDN URLs), `bokeh_css` (CDN URL), `plots` (list of dicts with `script` and `div`), `plot_script` (inline JS), `nav_links` (list of page links), `page_title`.
- Charts wider than 700px span the full grid row automatically.
- Styling: system font stack, `#4C72B0` primary color, `#2c3e50` dark text.

---

## How to Extend the Project

### Add a New Chart Type

1. **In `src/main.rs`**: Add a variant to the `ChartType` enum and its `as_str()` mapping.
2. **In `python/render.py`**: Add a rendering branch for the new chart type string in the chart-building loop.
3. Use the new type in a `ChartSpec`.

### Add a New Page

1. **In `src/main.rs`**: Add a `Page` to the `pages` vec with desired `ChartSpec`s and `filters: vec![]`. Add any new DataFrames to `frame_data`.
2. Navigation updates automatically — no template changes needed.

### Add a Filter to a Page

1. **In `src/main.rs`**: Add a `FilterSpec` to the page's `filters` vec with the desired `FilterConfig` variant.
2. Set `filtered: true` on any `ChartSpec`s that should respond to the filter (must share the same `source_key`).
3. Available filter types: `Range` (RangeSlider), `Select` (dropdown with "All"), `Group` (dropdown, single group — uses `GroupFilter`), `Threshold` (toggle switch), `TopN` (slider for top N rows — uses `IndexFilter`).
4. Multiple filters on the same `source_key` combine via `IntersectionFilter` automatically.
5. Python handles widget creation and `CustomJS` callbacks generically — no Python changes needed for existing filter types.

### Add a New Filter Type

1. **In `src/main.rs`**: Add a variant to `FilterConfig` with its parameters.
2. **In `src/main.rs`**: Add serialization for the new variant in the PyO3 bridge `match` block.
3. **In `python/render.py`**: Add a handler in `build_filter_objects()` that creates the Bokeh filter model, widget, and `CustomJS` callback.

---

## What NOT to Do

- **Do not** edit `Cargo.lock` manually — it is auto-managed by Cargo.
- **Do not** assume `render.py` is loaded from disk at runtime — it is compiled into the binary via `include_str!()`. Changes to `render.py` require a recompile.
- **Do not** add Python dependencies without adding them to `requirements.txt` and documenting in the README.
- **Do not** bypass PyO3's GIL (`Python::with_gil`) — always acquire it before running Python code.
- **Do not** use `polars` lazy operations without calling `.collect()` before serialization.
- **Do not** edit the `vendor/` directory manually — it is managed by `scripts/setup_vendor.sh` and gitignored.

---

## Testing

There are currently no automated tests. When adding tests:

- **Rust unit tests**: Use `#[cfg(test)]` modules in `src/main.rs`. Test `build_*_dataframe()` and `serialize_df()` independently of Python.
- **Python tests**: Use `pytest` for `render.py` logic if refactored into functions.
- **Integration tests**: Run `cargo run --release` and validate that `output/*.html` files are produced and contain expected content.

---

## Git Workflow

- `main` is the stable branch.
- **Branch creation**: Always create a new branch before making changes. Use the naming convention `claude/<short-description>` (e.g., `claude/add-pie-chart`, `claude/fix-slider-range`). Keep the description concise and lowercase with hyphens. Do not commit directly to `main`.
- Commits are merged via pull requests.
- `output/*.html` files are committed intentionally as live preview artifacts.

---

## Common Issues

| Problem | Likely Cause | Fix |
|---------|-------------|-----|
| `PYO3: could not find python` | Vendored Python not set up | Run `bash scripts/setup_vendor.sh` |
| `ModuleNotFoundError: bokeh` | Python deps missing | Run `pip install -r requirements.txt` in vendored env |
| `IpcWriter` compile error | `ipc` feature missing | Ensure `features = ["ipc"]` in `Cargo.toml` |
| Blank/empty chart | Frames dict key mismatch | Match `source_key` in ChartSpec with key in `frame_data` |
| Template not updating | `include_str!()` uses compile-time copy | Recompile after editing `templates/chart.html` |
| Python DLLs not found (Windows) | `build.rs` didn't copy DLLs | Run `bash scripts/setup_vendor.sh`, then rebuild |
