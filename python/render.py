# render.py — called from Rust via PyO3
# Variables injected by Rust:
#   chart_specs: list[dict]  — each dict has keys:
#       bytes (bytes), chart_type (str), title (str),
#       x_col (str), group_col (str), value_col (str), y_label (str)
#   html_template: str       — Jinja2 HTML template source
#   output_path: str         — destination file path

import io

import polars as pl
from bokeh.embed import components
from bokeh.models import ColumnDataSource, FactorRange
from bokeh.plotting import figure
from bokeh.resources import CDN
from bokeh.transform import factor_cmap
from jinja2 import Template

_DEFAULT_PALETTE = [
    "#4C72B0", "#DD8452", "#2ca02c",
    "#9467bd", "#e377c2", "#8c564b",
    "#17becf", "#bcbd22",
]


def build_grouped_bar(spec):
    df = pl.read_ipc(io.BytesIO(spec["bytes"]))
    x_col = spec["x_col"]
    group_col = spec["group_col"]
    value_col = spec["value_col"]

    groups = df[group_col].unique(maintain_order=True).to_list()
    x_factors = [(str(x), str(g)) for x, g in zip(df[x_col].to_list(), df[group_col].to_list())]
    source = ColumnDataSource(dict(x=x_factors, counts=df[value_col].to_list()))
    palette = _DEFAULT_PALETTE[: len(groups)]

    fig = figure(
        x_range=FactorRange(*x_factors),
        height=450,
        width=900,
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save",
    )
    fig.vbar(
        x="x",
        top="counts",
        width=0.9,
        source=source,
        line_color="white",
        fill_color=factor_cmap("x", palette=palette, factors=groups, start=1, end=2),
    )
    fig.x_range.range_padding = 0.1
    fig.xaxis.major_label_orientation = 1.0
    fig.xaxis.group_label_orientation = 0.5
    fig.yaxis.axis_label = spec["y_label"]
    fig.xgrid.grid_line_color = None
    return fig


# ── Dispatch table: chart_type string -> builder function ────────────────────

_BUILDERS = {
    "grouped_bar": build_grouped_bar,
}

# ── Build all figures ────────────────────────────────────────────────────────

figures = []
for spec in chart_specs:
    builder = _BUILDERS.get(spec["chart_type"])
    if builder is None:
        raise ValueError(f"Unknown chart_type: {spec['chart_type']!r}")
    figures.append(builder(spec))

# ── Combine all figures into a single Bokeh script ───────────────────────────

script, divs = components(figures)
bokeh_js_url = CDN.js_files[0]
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""

plots = [
    {"title": spec["title"], "div": div}
    for spec, div in zip(chart_specs, divs)
]

# ── Render Jinja2 template ───────────────────────────────────────────────────

template = Template(html_template)
html = template.render(
    title="RustToBokeh Dashboard",
    bokeh_js_url=bokeh_js_url,
    bokeh_css_url=bokeh_css_url,
    plot_script=script,
    plots=plots,
)

with open(output_path, "w") as f:
    f.write(html)
