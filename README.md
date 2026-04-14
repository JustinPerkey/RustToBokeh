# RustToBokeh

A Rust library for building interactive multi-page [Bokeh](https://bokeh.org/) dashboards. Data processing happens in Rust with [Polars](https://pola.rs/) DataFrames, then [PyO3](https://pyo3.rs/) bridges into Python where Bokeh renders the charts and [Jinja2](https://jinja.palletsprojects.com/) produces self-contained HTML files.

## How It Works

```
Rust (Polars DataFrames)
        │  serialize to Arrow IPC
        ▼
   ┌────┴─────┐
   │          │
Native      Python (PyO3 → Bokeh + Jinja2)
   │          │
   └────┬─────┘
        ▼
  output/*.html  (one file per page, with navigation)
```

1. **Build DataFrames** in Rust using Polars — one DataFrame per data source.
2. **Register data** with `Dashboard::add_df()`, which serializes each DataFrame to Arrow IPC bytes.
3. **Define pages** with `PageBuilder`, adding chart specs and optional interactive filters.
4. **Render** — pick a backend:
   - `Dashboard::render_native(BokehResources::Cdn)` — pure-Rust renderer, emits Bokeh's JSON document model directly. No Python at runtime.
   - `Dashboard::render()` (feature `python`) — PyO3 acquires the GIL, passes everything to the embedded `render.py`, writes one HTML file per page.

The Python script and HTML template are embedded into the binary at compile time using `include_str!()`, so when using the Python backend the executable has no runtime file dependencies beyond a Python interpreter and the required packages. The native backend needs nothing at runtime.

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
            .build()?,
    );
    dash.render()?;
    Ok(())
}
```

## Prerequisites

- Rust toolchain (1.75+)
- `curl` or `wget` (for downloading vendored Python)
- No system Python installation required when using the vendor setup

## Setup

### Vendored Python (recommended)

Run the setup script once after cloning. It downloads a standalone CPython 3.12 build from [python-build-standalone](https://github.com/indygreg/python-build-standalone), extracts it to `vendor/python/`, installs all required pip packages, and writes `.cargo/config.toml` so that PyO3 links against the vendored interpreter automatically.

```bash
bash scripts/setup_vendor.sh
```

Supported platforms: Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64.

To force a fresh download (e.g. after upgrading Python), delete `vendor/python/` and re-run the script:

```bash
rm -rf vendor/python
bash scripts/setup_vendor.sh
```

### Alternative: System Python

If you have a system or virtual-environment Python with the required packages:

```bash
pip install -r requirements.txt
```

Then set the `PYO3_PYTHON` environment variable to point at that interpreter before building:

```bash
PYO3_PYTHON=$(which python3) cargo build --release
```

Or add it permanently to `.cargo/config.toml`:

```toml
[env]
PYO3_PYTHON = { value = "/usr/bin/python3" }
```

### Offline builds

To make the project buildable with no internet access, comment out the `vendor/` line in `.gitignore` and commit the `vendor/python/` directory. This adds approximately 300 MB to the repository but allows cloning and building with zero downloads.

## Building and Running

```bash
cargo build --release
cargo run --bin example-dashboard --release
```

On success, HTML files are written to the `output/` directory in the current working directory — one file per dashboard page. Open any of them in a browser to explore the interactive charts and navigate between pages.

To run the unit tests (no Python required):

```bash
cargo test --lib
```

## Library Usage

Add `rust-to-bokeh` as a dependency in your `Cargo.toml`:

```toml
[dependencies]
rust-to-bokeh = { path = "..." }
polars = { version = "0.53", features = ["lazy"] }
```

Import the prelude to bring all types into scope:

```rust
use rust_to_bokeh::prelude::*;
use polars::prelude::*;
```

### Dashboard Builder

`Dashboard` is the top-level builder. Configure it with method chaining before calling `render()`:

```rust
let mut dash = Dashboard::new()
    .title("Q4 Financial Report")           // label shown in the nav bar
    .nav_style(NavStyle::Vertical)          // fixed left sidebar (default: Horizontal)
    .output_dir("reports/q4");              // output directory (default: "output")

// Register DataFrames — each is serialized to Arrow IPC immediately
dash.add_df("monthly", &mut monthly_df)?;
dash.add_df("by_region", &mut region_df)?;

// Add pages in display order
dash.add_page(overview_page()?);
dash.add_page(detail_page()?);

// Render all pages to HTML — pick a backend
dash.render_native(BokehResources::Cdn)?;   // pure Rust, no Python
// or
dash.render()?;                              // PyO3 → Python (feature `python`)
```

### Rendering Backends

| Method | Runtime | Feature flag | Notes |
|---|---|---|---|
| `render_native(BokehResources)` | Pure Rust | (default) | Emits Bokeh's JSON document model directly. `BokehResources::Cdn` loads JS/CSS from cdn.bokeh.org; `BokehResources::Inline` embeds them (requires `bokeh-inline` feature). |
| `render()` | PyO3 → embedded Python | `python` | Uses the embedded `render.py` and vendored Python interpreter. Equivalent output; useful when native gaps matter or for debugging. |

Both backends consume the same `Dashboard` state and produce one HTML file per page.

**Navigation styles:**

| `NavStyle` | Layout |
|---|---|
| `Horizontal` (default) | Sticky top bar; categories shown as inline labels before their group of links |
| `Vertical` | Fixed left sidebar; categories shown as section headings with page links stacked below |

### Grid Layout

Every page has a CSS grid with a configurable number of columns (1–6). Modules (charts, paragraphs, tables) are placed in the grid using `.at(row, col, span)` where `row` and `col` are zero-based and `span` is the number of columns the module occupies.

```
grid_cols = 3

row 0: [  chart A (span 2)  ] [  chart B (span 1)  ]
row 1: [          chart C (span 3)                  ]
```

```rust
PageBuilder::new("overview", "Overview", "Overview", 3)
    .chart(chart_a.at(0, 0, 2).build())   // row 0, starts at col 0, spans 2
    .chart(chart_b.at(0, 2, 1).build())   // row 0, starts at col 2, spans 1
    .chart(chart_c.at(1, 0, 3).build())   // row 1, spans all 3 columns
    .build()?
```

`PageBuilder::build()` validates that no two modules in the same row overlap and that no module overflows the grid boundaries, returning `ChartError::GridValidation` if violated.

### Page Categories and Navigation

Group pages under navigation headings by calling `.category()` on the `PageBuilder`. Pages sharing the same category string are grouped together. Use `"A/B"` syntax for hierarchical categories.

```rust
PageBuilder::new("revenue", "Revenue Overview", "Revenue", 2)
    .category("Financial")
    // ...

PageBuilder::new("expenses", "Expense Analysis", "Expenses", 2)
    .category("Financial")
    // ...

PageBuilder::new("timeseries", "Sensor Time Series", "Sensors", 2)
    .category("Reference/Time Series")
    // ...
```

Pages without a category are shown ungrouped at the top of the navigation.

### Supported Chart Types

| Type | Constructor | Key config fields |
|---|---|---|
| Grouped bar | `ChartSpecBuilder::bar(title, key, config)` | `x`, `group`, `value`, `y_label` |
| Multi-line | `ChartSpecBuilder::line(title, key, config)` | `x`, `y_cols`, `y_label` |
| Horizontal bar | `ChartSpecBuilder::hbar(title, key, config)` | `category`, `value`, `x_label` |
| Scatter plot | `ChartSpecBuilder::scatter(title, key, config)` | `x`, `y`, `x_label`, `y_label` |
| Pie / donut | `ChartSpecBuilder::pie(title, key, config)` | `label`, `value` |
| Histogram | `ChartSpecBuilder::histogram(title, key, config)` | `x_label` |
| Box plot | `ChartSpecBuilder::box_plot(title, key, config)` | `category`, `q1`–`q3`, `lower`, `upper`, `y_label` |
| Density | `ChartSpecBuilder::density(title, key, config)` | `category`, `value`, `y_label` |

Each chart config type has its own fluent builder accessed via `::builder()`:

```rust
// Grouped bar — one bar group per x value, coloured by the group column
let bar = ChartSpecBuilder::bar("Monthly Revenue", "monthly",
    GroupedBarConfig::builder()
        .x("month")
        .group("category")
        .value("amount")
        .y_label("USD (thousands)")
        .build()?
).at(0, 0, 2).build();

// Multi-line — one line per entry in y_cols
let line = ChartSpecBuilder::line("Trend", "monthly",
    LineConfig::builder()
        .x("month")
        .y_cols(&["revenue", "expenses"])
        .y_label("USD")
        .build()?
).at(1, 0, 2).build();

// Horizontal bar — ranked categories
let hbar = ChartSpecBuilder::hbar("Top Products", "products",
    HBarConfig::builder()
        .category("product")
        .value("sales")
        .x_label("Units Sold")
        .build()?
).at(0, 0, 1).build();

// Scatter plot
let scatter = ChartSpecBuilder::scatter("Price vs Volume", "trades",
    ScatterConfig::builder()
        .x("price")
        .y("volume")
        .x_label("Price")
        .y_label("Volume")
        .build()?
).at(0, 1, 1).build();

// Pie chart (set inner_radius for a donut)
let pie = ChartSpecBuilder::pie("Market Share", "market_share",
    PieConfig::builder()
        .label("company")
        .value("share")
        .inner_radius(0.45)   // omit for a solid pie
        .build()?
).at(0, 0, 1).dimensions(380, 380).build();
```

**Histogram and box plot** require pre-computed DataFrames. Use the helpers from
`rust_to_bokeh::stats` (re-exported via the prelude):

```rust
// Histogram: call compute_histogram() before registering data
let raw = df!["salary" => [42.0f64, 65.0, 80.0, 95.0]]?;
let mut hist = compute_histogram(&raw, "salary", 12)?;
dash.add_df("salary_hist", &mut hist)?;

let histogram = ChartSpecBuilder::histogram("Salary Distribution", "salary_hist",
    HistogramConfig::builder()
        .x_label("Salary (k)")
        .display(HistogramDisplay::Pdf)
        .build()?
).at(0, 0, 2).build();

// Box plot: call compute_box_stats() and optionally compute_box_outliers()
let mut stats = compute_box_stats(&raw, "department", "salary")?;
dash.add_df("salary_box", &mut stats)?;
let mut outliers = compute_box_outliers(&raw, "department", "salary")?;
dash.add_df("salary_outliers", &mut outliers)?;

let box_plot = ChartSpecBuilder::box_plot("Salary by Dept", "salary_box",
    BoxPlotConfig::builder()
        .category("category").q1("q1").q2("q2").q3("q3")
        .lower("lower").upper("upper")
        .y_label("Salary (k)")
        .outlier_source("salary_outliers")
        .outlier_value_col("salary")
        .build()?
).at(1, 0, 2).build();
```

**Density plots** use the raw long-format DataFrame directly (one row per observation):

```rust
// density_scores has columns "dept" (category) and "score" (numeric)
dash.add_df("density_scores", &mut density_scores_df)?;

let density = ChartSpecBuilder::density("Score Distribution", "density_scores",
    DensityConfig::builder()
        .category("dept")
        .value("score")
        .y_label("Performance Score")
        .palette(PaletteSpec::Named("Set2".into()))
        .build()?
).at(0, 0, 2).build();
```

The renderer automatically selects **sina** (jittered scatter) for sparsely
populated categories (≤ 50 points) and **violin** (KDE polygon) for denser ones.
In sina mode each point is jittered uniformly within the local KDE density
envelope so that points fill the interior of the distribution rather than
clustering on the boundary.
Override the threshold with `.point_threshold(n)` on `DensityConfig`.

**Chart dimensions:** override the default responsive width by calling `.dimensions(width, height)` on `ChartSpecBuilder`:

```rust
ChartSpecBuilder::scatter("Correlation", "data", cfg)
    .at(0, 0, 1)
    .dimensions(500, 400)   // fixed pixel size
    .build()
```

**Shared data sources:** multiple charts on the same page that reference the same `source_key` share one Bokeh `ColumnDataSource`, enabling linked hover and selection across all those charts automatically.

### Interactive Filters

Add filters to a page via `.filter()` on `PageBuilder`. Charts opt in by calling `.filtered()` on `ChartSpecBuilder` — only charts that share the filter's `source_key` and are marked as filtered will respond.

```rust
PageBuilder::new("analysis", "Product Analysis", "Products", 2)
    .chart(
        ChartSpecBuilder::bar("Sales by Product", "products", bar_cfg)
            .at(0, 0, 2)
            .filtered()        // this chart responds to filters on "products"
            .build()
    )
    .filter(FilterSpec::range("products", "sales", "Sales Range", 0.0, 500.0, 10.0))
    .filter(FilterSpec::select("products", "category", "Category",
        vec!["Electronics", "Clothing", "Food"]))
    .build()?
```

Multiple filters on the same `source_key` combine automatically via Bokeh's `IntersectionFilter`.

| Filter | Factory method | Widget | Behavior |
|---|---|---|---|
| Range | `FilterSpec::range(src, col, label, min, max, step)` | `RangeSlider` | Keeps rows where `col` is within `[min, max]` |
| Select | `FilterSpec::select(src, col, label, options)` | Dropdown | Exact value match; "All" shows everything |
| Group | `FilterSpec::group(src, col, label, options)` | Dropdown | Bokeh `GroupFilter`; no "All" option |
| Threshold | `FilterSpec::threshold(src, col, label, value, above)` | Toggle switch | Keeps rows above (or below) `value` |
| Top N | `FilterSpec::top_n(src, col, label, max_n, descending)` | Slider | Limits to top/bottom N rows sorted by `col` |
| Date range | `FilterSpec::date_range(src, col, label, min_ms, max_ms, step, scale)` | `DateRangeSlider` | Keeps rows where `col` (epoch-ms) is within the selected window |
| Range tool | `FilterSpec::range_tool(src, x_col, y_col, label, start, end, time_scale)` | Overview chart | Zooms the x-axis window of all line/scatter charts sharing `src` |

**Date range filter note:** the column must contain datetime values stored as milliseconds since the Unix epoch.

**Range tool note:** unlike the other filters, `RangeTool` does not hide rows via `CDSView`. It synchronises the visible x-axis window across charts sharing the same `source_key`. Charts do **not** need `.filtered()` to participate.

### Content Modules: Paragraphs and Tables

Pages can mix charts with styled text blocks and data tables.

**Paragraph:**

```rust
PageBuilder::new("about", "About This Report", "About", 2)
    .paragraph(
        ParagraphSpec::new(
            "This report covers Q4 2024 financial performance.\n\n\
             Data is sourced from the internal finance system."
        )
        .title("Report Overview")   // optional heading
        .at(0, 0, 2)
        .build()
    )
    .chart(/* ... */)
    .build()?
```

Separate multiple paragraphs with `"\n\n"` — each becomes its own `<p>` element.

**Table:**

```rust
PageBuilder::new("data-table", "Data Table", "Table", 2)
    .table(
        TableSpec::new("Monthly Summary", "monthly")
            .column(TableColumn::text("month", "Month"))
            .column(TableColumn::currency("revenue", "Revenue", "$", 0))
            .column(TableColumn::number("units", "Units", 0))
            .column(TableColumn::percent("margin", "Margin", 1))
            .at(0, 0, 2)
            .build()
    )
    .build()?
```

**Column formats:**

| Factory method | Example output |
|---|---|
| `TableColumn::text(key, label)` | `"Widget A"` |
| `TableColumn::number(key, label, decimals)` | `"3.14"` |
| `TableColumn::currency(key, label, symbol, decimals)` | `"$1,234.50"` |
| `TableColumn::percent(key, label, decimals)` | `"28.5%"` |

### Error Handling

All fallible operations return `Result<T, ChartError>`. The error type covers:

| Variant | Cause |
|---|---|
| `MissingField` | A required builder field was not set |
| `GridValidation` | Grid layout rules violated (overlap, out of bounds) |
| `Serialization` | Polars failed to serialize a DataFrame |
| `Python` | Python raised an exception during rendering |
| `InvalidScript` | The embedded script is malformed (should not occur in practice) |

`ChartError` implements `From<PolarsError>` and `From<PyErr>`, so it composes naturally with `?` in functions returning `Result<T, ChartError>` or `Result<T, Box<dyn Error>>`.

## Project Structure

```
RustToBokeh/
├── src/
│   ├── lib.rs                # Library root: NavStyle, serialize_df(), module decls + re-exports
│   ├── dashboard.rs          # Dashboard builder (add_df, add_page, render, render_native)
│   ├── stats.rs              # Statistical helpers: compute_histogram(), compute_box_stats(), compute_box_outliers()
│   ├── python_config.rs      # Vendored Python interpreter discovery
│   ├── render/               # PyO3 bridge to Python (private, feature `python`)
│   │   ├── mod.rs            # render_dashboard() entry + frame/nav/page builders
│   │   ├── chart_config.rs   # PyO3 serialisation for ChartConfig + palette/tooltip/axis
│   │   └── module.rs         # PyO3 serialisation for PageModule + ColumnFormat + FilterConfig
│   ├── error.rs              # ChartError enum
│   ├── prelude.rs            # Convenience re-exports
│   ├── pages.rs              # Page and PageBuilder
│   ├── modules.rs            # ParagraphSpec, TableSpec, TableColumn
│   ├── charts/               # Chart types and visual customisation
│   │   ├── mod.rs            # Re-exports all chart types
│   │   ├── charts/           # Per-chart config structs and builders
│   │   │   ├── mod.rs        # ChartConfig enum, GridCell, ChartSpec
│   │   │   ├── spec.rs       # ChartSpecBuilder
│   │   │   ├── grouped_bar.rs
│   │   │   ├── line.rs
│   │   │   ├── hbar.rs
│   │   │   ├── scatter.rs
│   │   │   ├── pie.rs
│   │   │   ├── histogram.rs
│   │   │   ├── box_plot.rs
│   │   │   └── density.rs
│   │   └── customization/    # Palette, tooltip, axis, filters
│   │       ├── mod.rs
│   │       ├── palette.rs
│   │       ├── time_scale.rs
│   │       ├── tooltip.rs
│   │       ├── axis.rs
│   │       └── filters.rs
│   ├── bokeh_native/         # Pure-Rust Bokeh HTML renderer (no Python)
│   │   ├── mod.rs            # BokehResources, render_native_dashboard() entry
│   │   ├── page.rs           # Per-page assembly (charts + filters + modules → HTML)
│   │   ├── placeholder.rs    # CDS-ID placeholder rewriting in filter widget callbacks
│   │   ├── modules_html.rs   # HTML rendering for paragraph and table modules
│   │   ├── document.rs       # BokehDocument root collection + embed-script emission
│   │   ├── model.rs          # BokehObject/BokehValue JSON representation
│   │   ├── id_gen.rs         # UUID generation
│   │   ├── html.rs           # Jinja-style page template + escape helpers
│   │   ├── nav.rs            # Horizontal/vertical navigation HTML
│   │   ├── palette.rs        # Named Bokeh palette lookup
│   │   ├── source.rs         # ColumnDataSource builder from Polars DataFrame
│   │   ├── axis.rs           # AxisBuilder for LinearAxis / CategoricalAxis / DatetimeAxis
│   │   ├── figure/           # Figure builder (axes, toolbar, glyph renderer)
│   │   │   ├── mod.rs        # build_figure() + XRangeKind/YRangeKind
│   │   │   ├── ranges.rs     # Range1d / DataRange1d / FactorRange builders
│   │   │   ├── tools.rs      # Toolbar tool builders (pan, zoom, hover, …)
│   │   │   └── glyph.rs      # GlyphRenderer + CDSView helper
│   │   ├── filters/          # Filter widget + Bokeh filter model builders
│   │   │   ├── mod.rs        # FilterOutput, build_filter_widgets(), combine_filters()
│   │   │   ├── range.rs      # RangeSlider → BooleanFilter
│   │   │   ├── select.rs     # Select dropdown with "(All)" → BooleanFilter
│   │   │   ├── group.rs      # Select dropdown → Bokeh GroupFilter
│   │   │   ├── threshold.rs  # Switch toggle → BooleanFilter
│   │   │   ├── top_n.rs      # Slider → IndexFilter (top/bottom N)
│   │   │   ├── date_range.rs # DatetimeRangeSlider → BooleanFilter
│   │   │   └── range_tool.rs # Overview chart + RangeTool → shared Range1d
│   │   └── charts/           # Per-chart-type native renderers
│   │       ├── mod.rs, grouped_bar.rs, line.rs, hbar.rs, scatter.rs,
│   │       ├── pie.rs, histogram.rs, box_plot.rs, density.rs
│   └── bin/
│       └── example_dashboard/
│           ├── main.rs       # Dashboard setup (register data, add pages, render)
│           ├── data.rs       # DataFrame builders for demo data
│           └── pages/        # 28-page demo, split by category
│               ├── executive.rs
│               ├── financial.rs
│               ├── commercial.rs
│               ├── digital.rs
│               ├── people.rs
│               ├── operations.rs
│               └── reference/
│                   ├── showcase.rs    # Module showcase, chart customisation
│                   ├── time_series.rs # RangeTool and DateRange demos
│                   └── statistical.rs # Pie, histogram, box plot, density demos
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
| Python | [bokeh](https://pypi.org/project/bokeh/) | 3.9.0 | Interactive chart rendering |
| Python | [polars](https://pypi.org/project/polars/) | 1.39.3 | Arrow IPC deserialization |
| Python | [jinja2](https://pypi.org/project/Jinja2/) | 3.1.6 | HTML template rendering |

## Troubleshooting

| Problem | Likely cause | Fix |
|---|---|---|
| `could not find python` at build time | PyO3 cannot locate the interpreter | Run `bash scripts/setup_vendor.sh`; or set `PYO3_PYTHON` explicitly |
| `ModuleNotFoundError: bokeh` at runtime | Python packages not installed | Re-run `bash scripts/setup_vendor.sh` or `pip install -r requirements.txt` |
| `IpcWriter` compile error | `ipc` feature not enabled | Ensure `features = ["ipc"]` in the `polars` dependency in `Cargo.toml` |
| Blank or empty chart | `source_key` mismatch | Match the `source_key` in `ChartSpec` with the key passed to `add_df()` |
| Template changes not reflected | `include_str!()` embeds at compile time | Recompile after editing `templates/chart.html` or `python/render.py` |
| Python DLLs not found on Windows | `build.rs` copy step failed | Run `bash scripts/setup_vendor.sh`, then do a clean rebuild |
| `GridValidation` error | Module overflows grid or modules overlap | Check `.at(row, col, span)` — `col + span` must not exceed `grid_cols`, and no two modules in the same row may overlap |
| Charts not responding to filters | Chart not marked `.filtered()` | Call `.filtered()` on `ChartSpecBuilder` for every chart that should respond |
| Histogram chart shows no data | Pre-computation not done | Call `compute_histogram()` before `add_df()` and pass the result |
| Box plot chart shows no data | Pre-computation not done | Call `compute_box_stats()` before `add_df()` and pass the result |

## License

MIT — see [LICENSE](LICENSE).
