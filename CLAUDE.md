# CLAUDE.md — AI Assistant Guide for RustToBokeh

## Project Overview

RustToBokeh is a demonstration of seamless Rust ↔ Python interoperability for data pipeline workflows. It uses Rust (with Polars) for data processing and Python (with Bokeh) for interactive visualization, bridged via PyO3.

**Core idea**: Rust builds DataFrames → serializes to Arrow IPC binary → passes to embedded Python → Python renders interactive Bokeh charts → outputs a self-contained HTML file.

---

## Repository Structure

```
RustToBokeh/
├── src/
│   └── main.rs              # Rust entry point; builds DataFrames, calls Python via PyO3
├── python/
│   └── render.py            # Python script; deserializes data, renders Bokeh charts
├── templates/
│   └── chart.html           # Jinja2 HTML template for final output
├── Cargo.toml               # Rust package manifest
├── Cargo.lock               # Pinned dependency versions (do not edit manually)
├── output.html              # Sample generated output (committed for preview purposes)
├── README.md                # User-facing setup and usage documentation
└── LICENSE                  # MIT License
```

---

## Architecture and Data Flow

```
[Rust - src/main.rs]
  │  Build Polars DataFrames
  │  Serialize to Arrow IPC binary (Vec<u8>)
  │  Embed python/render.py and templates/chart.html at compile time via include_str!()
  ↓
[PyO3 Bridge]
  │  Acquire Python GIL (Python::with_gil)
  │  Inject data dict: { frames, html_template, output_path }
  │  Execute render.py in that context
  ↓
[Python - python/render.py]
  │  Deserialize Arrow IPC bytes → Polars DataFrames
  │  Build grouped bar charts with Bokeh
  │  Extract JS + HTML components via bokeh.embed.components()
  │  Render Jinja2 template
  │  Write output.html to disk
```

Key insight: `include_str!()` embeds `render.py` and `chart.html` as string literals at **compile time** — no file I/O needed at runtime for these resources.

---

## Build and Run

### Prerequisites

- Rust toolchain (edition 2021, Rust 1.75+)
- Python 3.8+ with these packages:
  ```bash
  pip install bokeh jinja2 polars
  ```

### Build and Run

```bash
cargo build --release
cargo run --release
```

This produces `output.html` in the working directory.

### Optional: Specify Python Interpreter

```bash
export PYO3_PYTHON=$(which python3)
cargo build --release
```

---

## Key Dependencies

| Language | Crate/Package | Version | Purpose |
|----------|--------------|---------|---------|
| Rust | `pyo3` | 0.23 | Rust ↔ Python FFI, GIL management |
| Rust | `polars` | 0.53 | DataFrame creation, Arrow IPC serialization |
| Python | `bokeh` | latest | Interactive chart generation |
| Python | `polars` | latest | Arrow IPC deserialization |
| Python | `jinja2` | latest | HTML template rendering |

Polars features enabled in `Cargo.toml`: `lazy`, `ipc`, `parquet`.
PyO3 feature: `auto-initialize` (Python interpreter auto-initialized by Rust).

---

## Code Conventions

### Rust (`src/main.rs`)

- Use Polars `df!` macro for DataFrame construction.
- Serialize DataFrames with `IpcWriter` writing into a `std::io::Cursor`.
- Pass data to Python as a `HashMap<&str, PyObject>` dict.
- Use `.expect()` for error handling (acceptable for this demo; update to `?` propagation if error handling is needed in production extensions).
- Imports grouped by crate: `polars`, then `pyo3`, then `std`.

**Pattern for adding a new DataFrame:**
1. Add a `build_*_dataframe()` function returning `DataFrame`.
2. Serialize it with `serialize_df()`.
3. Insert the bytes into the `frames` dict passed to Python.
4. Consume it in `render.py`.

### Python (`python/render.py`)

- Script-style execution (no classes or top-level functions) — data arrives via injected local variables.
- Available variables at runtime: `frames` (dict of `str → bytes`), `html_template` (str), `output_path` (str).
- Deserialize frames: `polars.read_ipc(io.BytesIO(frames["key"]))`.
- Build charts with Bokeh's `ColumnDataSource`, `FactorRange`, and `factor_cmap()`.
- Combine figures using `bokeh.embed.components()` and pass to Jinja2.
- Keep chart construction modular: one section per chart.

### HTML Template (`templates/chart.html`)

- Jinja2 template; receives: `bokeh_js` (CDN URL), `bokeh_css` (CDN URL), `plots` (list of dicts with `script` and `div`), `plot_script` (inline JS).
- Use `{% for plot in plots %}` to render multiple charts.
- Styling: system font stack, `#4C72B0` primary color, `#2c3e50` dark text.

---

## How to Extend the Project

### Add a New Chart

1. **In `src/main.rs`**: Add a new `build_*_dataframe()` function and serialize it:
   ```rust
   let my_df = build_my_dataframe().unwrap();
   let my_bytes = serialize_df(my_df);
   frames.set_item("my_key", PyBytes::new(py, &my_bytes))?;
   ```

2. **In `python/render.py`**: Deserialize and build a Bokeh figure:
   ```python
   df_my = pl.read_ipc(io.BytesIO(frames["my_key"]))
   # ... build figure p_my ...
   plots.append({"script": script, "div": div})
   ```
   (Add to the list passed to `components()`)

3. **In `templates/chart.html`**: The `{% for plot in plots %}` loop handles new charts automatically.

---

## What NOT to Do

- **Do not** edit `Cargo.lock` manually — it is auto-managed by Cargo.
- **Do not** assume `render.py` is loaded from disk at runtime — it is compiled into the binary via `include_str!()`. Changes to `render.py` require a recompile.
- **Do not** add Python dependencies without documenting them in the README prerequisites section.
- **Do not** bypass PyO3's GIL (`Python::with_gil`) — always acquire it before running Python code.
- **Do not** use `polars` lazy operations without calling `.collect()` before serialization.

---

## Testing

There are currently no automated tests. When adding tests:

- **Rust unit tests**: Use `#[cfg(test)]` modules in `src/main.rs`. Test `build_*_dataframe()` and `serialize_df()` independently of Python.
- **Python tests**: Use `pytest` for `render.py` logic if refactored into functions.
- **Integration tests**: Run `cargo run --release` and validate that `output.html` is produced and contains expected content.

---

## Git Workflow

- `master` is the stable branch.
- Development branches follow `claude/<feature>-<id>` naming.
- Commits are merged via pull requests (see git history pattern).
- `output.html` is committed intentionally as a live preview artifact.

---

## Common Issues

| Problem | Likely Cause | Fix |
|---------|-------------|-----|
| `PYO3: could not find python` | Python not in PATH | Set `PYO3_PYTHON=$(which python3)` |
| `ModuleNotFoundError: bokeh` | Python deps missing | `pip install bokeh jinja2 polars` |
| `IpcWriter` compile error | `ipc` feature missing | Ensure `features = ["ipc"]` in `Cargo.toml` |
| Blank/empty chart | Frames dict key mismatch | Match key names between `main.rs` and `render.py` |
| Template not updating | `include_str!()` uses compile-time copy | Recompile after editing `templates/chart.html` |
