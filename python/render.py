# render.py — called from Rust via PyO3
# Variables injected by Rust:
#   frames: dict[str, bytes]   — Arrow IPC bytes keyed by source name
#   pages: list[dict]          — each page has slug, title, grid_cols, specs
#   nav_links: list[dict]      — slug + label for every page (navigation)
#   html_template: str         — Jinja2 HTML template source
#   output_dir: str            — output directory path

import io
import os

import polars as pl
from bokeh.embed import components
from bokeh.models import ColumnDataSource, FactorRange
from bokeh.plotting import figure
from bokeh.resources import CDN
from bokeh.transform import factor_cmap
from jinja2 import Template

_PALETTE = [
    "#4C72B0", "#DD8452", "#55A868", "#C44E52",
    "#8172B3", "#937860", "#DA8BC3", "#8C8C8C",
    "#CCB974", "#64B5CD",
]

# ── Deserialize all frames once ─────────────────────────────────────────────

dataframes = {}
for key, raw in frames.items():
    dataframes[key] = pl.read_ipc(io.BytesIO(raw))

# ── Chart builders ──────────────────────────────────────────────────────────
# Each builder receives (spec_dict, source_cache) and returns a Bokeh figure.
# Charts with the same source_key on the same page share a ColumnDataSource,
# which gives them linked selection and hover for free.


def build_grouped_bar(spec, source_cache):
    key = spec["source_key"]
    df = dataframes[key]
    x_col, group_col, value_col = spec["x_col"], spec["group_col"], spec["value_col"]

    groups = df[group_col].unique(maintain_order=True).to_list()
    x_factors = [
        (str(x), str(g))
        for x, g in zip(df[x_col].to_list(), df[group_col].to_list())
    ]

    cache_key = key + "__grouped_bar"
    if cache_key in source_cache:
        source = source_cache[cache_key]
    else:
        source = ColumnDataSource(dict(x=x_factors, counts=df[value_col].to_list()))
        source_cache[cache_key] = source

    palette = _PALETTE[: len(groups)]
    fig = figure(
        x_range=FactorRange(*x_factors),
        height=400,
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save,hover",
        sizing_mode="stretch_width",
    )
    fig.vbar(
        x="x", top="counts", width=0.9, source=source,
        line_color="white",
        fill_color=factor_cmap("x", palette=palette, factors=groups, start=1, end=2),
    )
    fig.x_range.range_padding = 0.1
    fig.xaxis.major_label_orientation = 1.0
    fig.xaxis.group_label_orientation = 0.5
    fig.yaxis.axis_label = spec.get("y_label", "")
    fig.xgrid.grid_line_color = None
    return fig


def build_line_multi(spec, source_cache):
    key = spec["source_key"]
    df = dataframes[key]
    x_col = spec["x_col"]
    y_cols = [c.strip() for c in spec["y_cols"].split(",")]

    cache_key = key + "__line"
    if cache_key in source_cache:
        source = source_cache[cache_key]
    else:
        data = {col: df[col].to_list() for col in df.columns}
        source = ColumnDataSource(data)
        source_cache[cache_key] = source

    fig = figure(
        height=400,
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save,hover",
        sizing_mode="stretch_width",
        x_range=df[x_col].to_list(),
    )
    for i, col in enumerate(y_cols):
        color = _PALETTE[i % len(_PALETTE)]
        fig.line(x=x_col, y=col, source=source, line_width=2.5,
                 color=color, legend_label=col)
        fig.scatter(x=x_col, y=col, source=source, size=7,
                    color=color, legend_label=col)
    fig.yaxis.axis_label = spec.get("y_label", "")
    fig.legend.location = "top_left"
    fig.legend.click_policy = "hide"
    return fig


def build_hbar(spec, source_cache):
    key = spec["source_key"]
    df = dataframes[key]
    cat_col = spec["category_col"]
    val_col = spec["value_col"]

    cache_key = key + "__hbar"
    if cache_key in source_cache:
        source = source_cache[cache_key]
    else:
        cats = df[cat_col].to_list()
        vals = df[val_col].to_list()
        source = ColumnDataSource(dict(categories=cats, values=vals))
        source_cache[cache_key] = source

    cats = source.data["categories"]
    fig = figure(
        y_range=list(reversed(cats)),
        height=max(300, len(cats) * 40 + 80),
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save,hover",
        sizing_mode="stretch_width",
    )
    fig.hbar(
        y="categories", right="values", height=0.7, source=source,
        line_color="white", fill_color="#4C72B0",
    )
    fig.xaxis.axis_label = spec.get("x_label", "")
    fig.ygrid.grid_line_color = None
    return fig


def build_scatter(spec, source_cache):
    key = spec["source_key"]
    df = dataframes[key]
    x_col = spec["x_col"]
    y_col = spec["y_col"]

    cache_key = key + "__scatter"
    if cache_key in source_cache:
        source = source_cache[cache_key]
    else:
        data = {col: df[col].to_list() for col in df.columns}
        source = ColumnDataSource(data)
        source_cache[cache_key] = source

    fig = figure(
        height=400,
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save,hover",
        sizing_mode="stretch_width",
    )
    fig.scatter(
        x=x_col, y=y_col, source=source,
        size=10, color="#4C72B0", alpha=0.7,
    )
    fig.xaxis.axis_label = spec.get("x_label", "")
    fig.yaxis.axis_label = spec.get("y_label", "")
    return fig


_BUILDERS = {
    "grouped_bar": build_grouped_bar,
    "line_multi": build_line_multi,
    "hbar": build_hbar,
    "scatter": build_scatter,
}

# ── Render all pages ────────────────────────────────────────────────────────

os.makedirs(output_dir, exist_ok=True)
template = Template(html_template)
bokeh_js_urls = CDN.js_files
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""

for page in pages:
    source_cache = {}  # per-page CDS cache for linking
    figs = []
    grid_items = []

    for spec in page["specs"]:
        builder = _BUILDERS.get(spec["chart_type"])
        if builder is None:
            raise ValueError(f"Unknown chart_type: {spec['chart_type']!r}")
        fig = builder(spec, source_cache)
        figs.append(fig)
        grid_items.append({
            "title": spec["title"],
            "grid_row": spec["grid_row"] + 1,
            "grid_col": spec["grid_col"] + 1,
            "grid_col_span": spec["grid_col_span"],
        })

    script, divs = components(figs)
    plots = [{**item, "div": div} for item, div in zip(grid_items, divs)]

    html = template.render(
        title=page["title"],
        bokeh_js_urls=bokeh_js_urls,
        bokeh_css_url=bokeh_css_url,
        plot_script=script,
        plots=plots,
        grid_cols=page["grid_cols"],
        nav_links=nav_links,
        current_slug=page["slug"],
    )

    path = os.path.join(output_dir, f"{page['slug']}.html")
    with open(path, "w") as f:
        f.write(html)
