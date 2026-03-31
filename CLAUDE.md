# CLAUDE.md — AI Assistant Guide for RustToBokeh

## Project Overview

RustToBokeh is a demonstration of seamless Rust ↔ Python interoperability for data pipeline workflows. It uses Rust (with Polars) for data processing and Python (with Bokeh) for interactive visualization, bridged via PyO3.

**Core idea**: Rust builds DataFrames → serializes to Arrow IPC binary → passes to embedded Python → Python renders multi-page interactive Bokeh dashboards → outputs self-contained HTML files with cross-chart linking.

---

## Repository Structure

```
RustToBokeh/
├── src/
│   ├── lib.rs               # Library root: Dashboard builder, serialize_df(), NavStyle
│   ├── stats.rs             # compute_histogram(), compute_box_stats(), compute_box_outliers()
│   ├── python_config.rs     # configure_vendored_python() — vendored interpreter discovery
│   ├── render.rs            # PyO3 bridge: render_dashboard() (private module)
│   ├── error.rs             # ChartError enum
│   ├── prelude.rs           # Convenience re-exports (use rust_to_bokeh::prelude::*)
│   ├── pages.rs             # Page, PageBuilder
│   ├── modules.rs           # PageModule, ParagraphSpec, TableSpec, TableColumn
│   ├── charts/              # Chart configuration types, layout primitives, filter definitions
│   │   ├── mod.rs           # Re-exports everything from sub-modules
│   │   ├── charts/          # Chart types and their builders
│   │   │   ├── mod.rs       # ChartConfig enum, GridCell, ChartSpec + re-exports
│   │   │   ├── spec.rs      # ChartSpecBuilder
│   │   │   ├── grouped_bar.rs  # GroupedBarConfig + builder
│   │   │   ├── line.rs      # LineConfig + builder
│   │   │   ├── hbar.rs      # HBarConfig + builder
│   │   │   ├── scatter.rs   # ScatterConfig + builder
│   │   │   ├── pie.rs       # PieConfig + builder
│   │   │   ├── histogram.rs # HistogramConfig + HistogramDisplay + builder
│   │   │   ├── box_plot.rs  # BoxPlotConfig + builder
│   │   │   └── density.rs   # DensityConfig + builder
│   │   └── customization/   # Visual styling and interactive filter definitions
│   │       ├── mod.rs       # Re-exports all customization types
│   │       ├── palette.rs   # PaletteSpec enum
│   │       ├── time_scale.rs  # TimeScale enum, DateStep enum
│   │       ├── tooltip.rs   # TooltipFormat, TooltipField, TooltipSpec + builder
│   │       ├── axis.rs      # AxisConfig + builder
│   │       └── filters.rs   # FilterConfig enum (7 variants), FilterSpec + factory methods
│   └── bin/
│       └── example_dashboard/
│           ├── main.rs      # Dashboard setup: register DataFrames, add pages, render
│           ├── data.rs      # DataFrame builders for demo data
│           └── pages/       # 28-page demo dashboard, organized by category
│               ├── mod.rs       # Re-exports all page functions
│               ├── executive.rs # Executive summary page (no category)
│               ├── financial.rs # 6 Financial pages
│               ├── commercial.rs  # 3 Commercial pages
│               ├── digital.rs   # 3 Digital pages
│               ├── people.rs    # 3 People pages
│               ├── operations.rs  # 4 Operations pages
│               └── reference/   # Reference pages grouped by type
│                   ├── mod.rs
│                   ├── showcase.rs    # Module Showcase + Chart Customization
│                   ├── time_series.rs # RangeTool Navigator + Sensor Time Series
│                   └── statistical.rs # Pie & Donut, Histogram, Box Plot, Density
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
├── vendor/                  # Vendored standalone Python (gitignored, created by setup_vendor.sh)
├── README.md                # User-facing setup and usage documentation
└── LICENSE                  # MIT License
```

---

## Architecture and Data Flow

```
[Rust Library - src/lib.rs + src/render.rs]
  │  Provides Dashboard builder, serialize_df(), render_dashboard()
  │  Embeds python/render.py and templates/chart.html at compile time via include_str!()
  │
[User Binary - e.g. src/bin/example_dashboard/main.rs]
  │  Build Polars DataFrames (wide format: one row per category, one column per series)
  │  Call compute_histogram() / compute_box_stats() / compute_box_outliers() from src/stats.rs
  │  Define Pages and ChartSpecs using builder API
  │  Call Dashboard::render() or render_dashboard()
  ↓
[PyO3 Bridge - src/render.rs]
  │  Serialize DataFrames to Arrow IPC binary (Vec<u8>)
  │  Acquire Python GIL (Python::with_gil)
  │  Inject: { frames, pages, html_template, output_dir }
  │  Execute render.py in that context
  ↓
[Python - python/render.py]
  │  Deserialize Arrow IPC bytes → Polars DataFrames
  │  Build charts from ChartSpec dicts (grouped_bar, line_multi, hbar, scatter, pie, histogram, box_plot, density)
  │  Line/scatter charts sharing the same source_key share one ColumnDataSource (linked selection)
  │  Build Bokeh filter objects (BooleanFilter, GroupFilter, IndexFilter) from FilterSpecs
  │  Combine filters via IntersectionFilter → CDSView on filtered chart renderers
  │  Create widgets (RangeSlider, Select, Switch, Slider, DateRangeSlider) with CustomJS callbacks
  │  RangeTool: attaches a Bokeh RangeTool to an auto-generated overview chart, syncing x-axis Range1d
  │  Render each Page to its own HTML file via Jinja2
  │  Write output/<slug>.html files with inter-page navigation
```

Key architectural concepts:
- `include_str!()` embeds `render.py` and `chart.html` as string literals at **compile time** — no file I/O needed at runtime for these resources.
- **`stats.rs`**: pure-Rust functions (`compute_histogram`, `compute_box_stats`, `compute_box_outliers`) that pre-compute statistical summaries before data is passed to Python.
- **`python_config.rs`**: searches for a vendored Python interpreter at startup and sets `PYTHONHOME`, `PYTHONPATH`, and `PATH` accordingly. Called automatically by `render_dashboard()`.
- **ChartSpec**: declarative chart definition (type, data source key, columns, dimensions, `filtered` flag). Defined in Rust, consumed by Python.
- **FilterSpec**: declarative filter definition (source_key, column, label, `FilterConfig` variant). Defined in Rust per-page, consumed by Python to build Bokeh filter objects and widgets.
- **FilterConfig** enum: `Range` (RangeSlider → BooleanFilter), `Select` (dropdown with "All" → BooleanFilter), `Group` (dropdown → GroupFilter), `Threshold` (Switch toggle → BooleanFilter), `TopN` (Slider → IndexFilter), `DateRange` (DateRangeSlider → BooleanFilter on epoch-ms column), `RangeTool` (overview chart with draggable range selector → x-axis Range1d sync).
- **Page**: groups ChartSpecs + FilterSpecs into a single HTML file. Each page embeds only the data it needs.
- **Shared ColumnDataSource**: all charts on the same page that reference the same `source_key` share one flat CDS, enabling linked hover/selection across all chart types.
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
cargo run --bin example-dashboard --release
```

This produces HTML files in the `output/` directory (one per page).

To use as a library in your own binary, add `rust-to-bokeh` as a dependency and use the `Dashboard` builder API (see `src/bin/example_dashboard/main.rs` for a full example).

---

## Key Dependencies

| Language | Crate/Package | Version | Purpose |
|----------|--------------|---------|---------|
| Rust | `pyo3` | 0.23 | Rust ↔ Python FFI, GIL management |
| Rust | `polars` | 0.53 | DataFrame creation, Arrow IPC serialization |
| Python | `bokeh` | 3.9.0 | Interactive chart generation |
| Python | `polars` | 1.39.3 | Arrow IPC deserialization |
| Python | `jinja2` | 3.1.6 | HTML template rendering |

Polars features enabled in `Cargo.toml`: `lazy`, `ipc`, `parquet`.
PyO3 feature: `auto-initialize` (Python interpreter auto-initialized by Rust).
Python versions are pinned in `requirements.txt`.

---

## Code Conventions

### Rust Library (`src/lib.rs`, `src/stats.rs`, `src/charts/`, `src/pages.rs`, `src/render.rs`)

- **`Dashboard`** builder: high-level API that collects DataFrames via `add_df()` and pages via `add_page()`, then calls `render()`.
- **`serialize_df()`**: standalone function to serialize a Polars DataFrame to Arrow IPC bytes.
- **`render_dashboard()`**: lower-level function taking pre-serialized frame data and page definitions.
- **`compute_histogram()`**, **`compute_box_stats()`**, **`compute_box_outliers()`**: pure-Rust statistics helpers in `src/stats.rs`. Always call these before `add_df()` when using histogram or box plot charts.
- **`ChartSpecBuilder`**: fluent builder with `bar()`, `line()`, `hbar()`, `scatter()`, `pie()`, `histogram()`, `box_plot()`, `density()` constructors, chained with `.at(row, col, span)`, `.filtered()`, and `.dimensions(width, height)`.
- **`PageBuilder`**: fluent builder with `.chart()`, `.paragraph()`, `.table()`, and `.filter()` methods.
- **`FilterSpec`** factory methods: `range()`, `select()`, `group()`, `threshold()`, `top_n()`, `date_range()`, `range_tool()`.
- Use `.expect()` for error handling (acceptable for this demo; update to `?` propagation if error handling is needed in production extensions).

**Supported chart types** (`ChartConfig` enum in `src/charts/charts/mod.rs`): `GroupedBar`, `Line`, `HBar`, `Scatter`, `Pie`, `Histogram`, `BoxPlot`, `Density`.

**Chart module layout** (`src/charts/`):
- `charts/` — chart type definitions and config builders (one file per chart type)
- `customization/` — palette, time scale, tooltip, axis config, and filter definitions

**Pattern for adding a new chart to an existing page:**
1. If needed, build a DataFrame and register it with `dash.add_df("key", &mut df)`.
2. Add a `ChartSpec` via the builder to the relevant page.
3. Python's `render.py` handles the rest generically — no Python changes needed unless adding a new chart type.

**Pattern for adding a new page:**
1. Add a new function to the appropriate file under `src/bin/example_dashboard/pages/`.
2. Re-export it in `pages/mod.rs` and call it in `main.rs`.
3. Ensure referenced `source_key`s have been registered with `add_df()`.
4. Navigation is generated automatically by the template.

### Python (`python/render.py`)

- Script-style execution with helper functions — data arrives via injected local variables.
- Available variables at runtime: `frames` (dict of `str → bytes`), `pages` (list of page dicts), `nav_links` (list), `html_template` (str), `output_dir` (str).
- Deserialize frames: `polars.read_ipc(io.BytesIO(frames["key"]))`.
- Chart rendering is driven by ChartSpec dicts — the Python code is generic, not per-chart.
- Uses Bokeh's `ColumnDataSource`, `CDSView`, `FactorRange`, `factor_cmap()`, and `CustomJS` for interactivity.
- Filtering uses Bokeh native filter models: `BooleanFilter`, `GroupFilter`, `IndexFilter`, combined via `IntersectionFilter`.
- `RangeTool`: attaches a Bokeh `RangeTool` to an overview chart; syncs the `Range1d` shared by all detail charts on the page.
- Use `bokeh.embed.components()` for embedding and Bokeh CDN for JS/CSS resources.

### HTML Template (`templates/chart.html`)

- Jinja2 template with responsive CSS grid layout and inter-page navigation bar.
- Receives: `bokeh_js_urls` (list of CDN URLs), `bokeh_css` (CDN URL), `plots` (list of dicts with `script` and `div`), `plot_script` (inline JS), `nav_links` (list of page links), `page_title`.
- Charts wider than 700px span the full grid row automatically.
- Styling: system font stack, `#4C72B0` primary color, `#2c3e50` dark text.

---

## Example Dashboard Feature Coverage

The 28-page example dashboard in `src/bin/example_dashboard/` demonstrates every available feature:

| Feature | Where demonstrated |
|---------|-------------------|
| Grouped bar chart | Revenue Overview, Quarterly Performance, and others |
| Multi-line chart | Module Showcase, Chart Customization, Time Series |
| Horizontal bar chart | Market Position, Chart Customization |
| Scatter plot | Chart Customization, RangeTool Navigator, Time Series |
| Pie and donut charts | Pie & Donut Charts (`reference/statistical.rs`) |
| Histogram (count/PDF/CDF) | Histogram Demo (`reference/statistical.rs`) |
| Box plot with outliers | Box Plot Demo (`reference/statistical.rs`) |
| Density plot (sina/violin) | Density Plots (`reference/statistical.rs`) |
| `FilterConfig::Range` — RangeSlider | Executive Summary, Product Analysis |
| `FilterConfig::Select` — dropdown with "All" | Product Analysis, Financial Health, Time Series |
| `FilterConfig::Group` — Bokeh GroupFilter | Customer Insights |
| `FilterConfig::Threshold` — toggle switch | Team Metrics, Cost Optimization, Workforce Planning |
| `FilterConfig::TopN` — slider for top-N rows | Project Portfolio, Workforce Planning |
| `FilterConfig::DateRange` — DateRangeSlider | Sensor Time Series (`reference/time_series.rs`) |
| `FilterConfig::RangeTool` — overview navigator | RangeTool Navigator (`reference/time_series.rs`) |
| Multiple filters on one source (IntersectionFilter) | Product Analysis, Workforce Planning, Time Series |
| `ParagraphSpec` — text content module | Module Showcase, Time Series, Density Plots |
| `TableSpec` — data table with column formats | Module Showcase |
| `NavStyle::Vertical` — fixed left sidebar | Whole example dashboard |
| `NavStyle::Horizontal` — sticky top bar | Default; tested in `tests/dashboard_output.rs` |
| Page categories (grouped nav) | Financial, Commercial, Digital, People, Operations |
| Hierarchical nav categories (`"A/B"` syntax) | Reference/Time Series |
| `ChartSpec::dimensions(w, h)` — fixed-size chart | Chart Customisation (scatter), Pie & Donut |
| Custom colors, markers, palettes, line widths | Chart Customisation |
| `TooltipSpec` — multi-field custom tooltips | Chart Customisation, Time Series |
| `AxisConfig` — ranges, bounds, tick format, grid | Chart Customisation |
| `TimeScale` — datetime axis formatting | Time Series (line chart) |
| `TooltipFormat::DateTime` — datetime tooltips | Time Series |

---

## How to Extend the Project

### Add a New Chart Type

1. **In `src/charts/charts/`**: Create a new config file (e.g., `pie.rs`) with the config struct and builder. Add the variant to `ChartConfig` in `mod.rs` and re-export from `mod.rs`. Add a builder method to `ChartSpecBuilder` in `spec.rs`.
2. **In `python/render.py`**: Add a rendering branch for the new chart type string in the chart-building loop.
3. Use the new type in a `ChartSpec` via the builder.

### Add a New Page

1. Add a new function to the appropriate file under `src/bin/example_dashboard/pages/` (or create a new file).
2. Re-export it in `pages/mod.rs` and call it in `main.rs`.
3. Ensure referenced `source_key`s have been registered with `dash.add_df()`.
4. Navigation updates automatically — no template changes needed.

### Add a Filter to a Page

1. Add a `FilterSpec` via its factory method (e.g. `FilterSpec::range(...)`) to the `PageBuilder` chain.
2. Mark charts with `.filtered()` to opt them into CDSView-based filtering (must share the same `source_key`). Charts using `RangeTool` do **not** need `.filtered()`.
3. Available filter types: `Range` (RangeSlider), `Select` (dropdown with "All"), `Group` (dropdown, single group — uses `GroupFilter`), `Threshold` (toggle switch), `TopN` (slider for top N rows — uses `IndexFilter`), `DateRange` (DateRangeSlider for epoch-ms columns), `RangeTool` (overview chart with draggable range selector, syncs x-axis).
4. Multiple filters on the same `source_key` combine via `IntersectionFilter` automatically.
5. Python handles widget creation and `CustomJS` callbacks generically — no Python changes needed for existing filter types.

### Add a New Filter Type

1. **In `src/charts/customization/filters.rs`**: Add a variant to `FilterConfig` with its parameters. Add a factory method to `FilterSpec`.
2. **In `src/render.rs`**: Add serialization for the new variant in the PyO3 bridge `match` block.
3. **In `python/render.py`**: Add a handler in `build_filter_objects()` that creates the Bokeh filter model, widget, and `CustomJS` callback.

### Add Statistical Charts (Histogram / Box Plot)

Histogram and box plot charts require pre-computed DataFrames. In `main.rs`:

```rust
let raw = data::build_salary_distribution();
let mut hist = compute_histogram(&raw, "salary", 12)?;
dash.add_df("salary_hist", &mut hist)?;

let raw2 = data::build_salary_raw();
let mut box_stats = compute_box_stats(&raw2, "department", "salary_k")?;
dash.add_df("salary_box", &mut box_stats)?;
let mut outliers = compute_box_outliers(&raw2, "department", "salary_k")?;
dash.add_df("salary_outliers", &mut outliers)?;
```

---

## What NOT to Do

- **Do not** edit `Cargo.lock` manually — it is auto-managed by Cargo.
- **Do not** assume `render.py` is loaded from disk at runtime — it is compiled into the binary via `include_str!()`. Changes to `render.py` require a recompile.
- **Do not** add Python dependencies without adding them to `requirements.txt` and documenting in the README.
- **Do not** bypass PyO3's GIL (`Python::with_gil`) — always acquire it before running Python code.
- **Do not** use `polars` lazy operations without calling `.collect()` before serialization.
- **Do not** edit the `vendor/` directory manually — it is managed by `scripts/setup_vendor.sh` and gitignored.
- **Do not** use histogram or box plot charts without first calling `compute_histogram()` / `compute_box_stats()` and registering the result via `add_df()`.

---

## Testing

The library has unit tests across all `src/charts/` sub-modules, `src/pages.rs`, `src/modules.rs`, `src/lib.rs`, and `src/stats.rs`. Run them with:

```bash
cargo test --lib
```

Integration tests are in `tests/dashboard_output.rs`. They require a Python interpreter with the required packages installed.

- **Rust unit tests**: `#[cfg(test)]` modules in each source file test the builders and validators independently of Python.
- **Integration tests**: Run `cargo run --bin example-dashboard --release` and validate that `output/*.html` files are produced and contain expected content.

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
| Histogram/box plot shows nothing | Missing pre-computation step | Call `compute_histogram()` / `compute_box_stats()` before `add_df()` |
