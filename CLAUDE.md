# CLAUDE.md — RustToBokeh

Rust+Polars → Arrow IPC → embedded Python+Bokeh → self-contained HTML via PyO3.

## Primary use

Render Bokeh dashboards **inline with vendored resources**. Bokeh JS/CSS injected via `bokeh.resources.INLINE` (`render.py` L1399-1400) into `templates/chart.html`. Python runtime + deps come from `vendor/` (not system Python). Output HTML has zero CDN/runtime deps — ship single file, opens offline. `scripts/setup_vendor.sh` populates `vendor/`; `src/python_config.rs` points PyO3 at it.

## Flow

Binary builds Polars DFs, registers via `Dashboard::add_df()`, defines `Page`s with `ChartSpec`/`FilterSpec`, calls `render()`. `src/render.rs` serializes to Arrow IPC, takes GIL, runs embedded `python/render.py` (via `include_str!`). Python deserializes, builds Bokeh, applies `CDSView`+`IntersectionFilter`, emits one HTML per page with inline Bokeh JS/CSS.

## Layout

- `src/lib.rs` — `Dashboard`, `serialize_df`, `NavStyle`
- `src/stats.rs` — `compute_histogram`/`compute_box_stats`/`compute_box_outliers` (call before `add_df`)
- `src/render.rs` — PyO3 bridge
- `src/python_config.rs` — vendored Python discovery
- `src/pages.rs`, `src/modules.rs` — Page, ParagraphSpec, TableSpec
- `src/charts/charts/` — `ChartConfig`: GroupedBar, Line, HBar, Scatter, Pie, Histogram, BoxPlot, Density. `ChartSpecBuilder` with `.at()`, `.filtered()`, `.dimensions()`
- `src/charts/customization/` — PaletteSpec, TimeScale, TooltipSpec, AxisConfig, `FilterConfig` (Range, Select, Group, Threshold, TopN, DateRange, RangeTool)
- `src/bin/example_dashboard/` — demo
- `python/render.py`, `templates/chart.html` — embedded compile-time
- `scripts/setup_vendor.sh` → `vendor/` (Python + bokeh + polars + jinja2)

## Build

```bash
bash scripts/setup_vendor.sh
cargo build --release
cargo run --bin example-dashboard --release
```

`.cargo/config.toml` sets `PYO3_PYTHON` → vendored interpreter. `build.rs` copies DLLs on Windows. Output: `output/`.

Deps: pyo3 0.23, polars 0.53 (lazy/ipc/parquet), bokeh 3.9.0, python-polars 1.39.3, jinja2 3.1.6.

## Patterns

- New chart: config in `charts/charts/`, `ChartConfig` variant, `ChartSpecBuilder` method, handler in `render.py`.
- New page: fn under `example_dashboard/pages/`, re-export, call from `main.rs`. Nav auto.
- New filter: `FilterConfig` variant + factory in `filters.rs`, serialize branch in `render.rs`, handler in `build_filter_objects()` in `render.py`.
- Shared `source_key` on one page = shared CDS = linked selection. Multi filters on same source = `IntersectionFilter` auto.

## Rules

- No manual `Cargo.lock` / `vendor/` edits.
- Recompile after `render.py` or `chart.html` change (`include_str!`).
- `.collect()` lazy Polars before serialize.
- Always `Python::with_gil`.
- Python deps → `requirements.txt`; re-run `setup_vendor.sh` after change.
- Keep Bokeh resource mode `INLINE` — output must stay self-contained, no CDN fallback.

## Test / Git

`cargo nextest run`. Branch `claude/<desc>`, PR to `main`, never commit direct.

## Design

Technical · industrial · utilitarian. Lab-instrument aesthetic. Dual light+dark (OKLCH, `light-dark()`, no hex). Humanist body + tabular numeric face (avoid Inter/Plex/Space Grotesk/DM Sans). Flat cards, dense rhythmic spacing, one sharp accent. No gradients/glow/glassmorphism/decorative motion. Data > chrome.
