# render.py — called from Rust via PyO3
# Variables injected by Rust:
#   frames: dict[str, bytes]  — Arrow IPC bytes keyed by source_key
#   chart_specs: list[dict]   — each dict has keys:
#       chart_type (str), title (str), source_key (str),
#       x_col (str), value_cols (list[str]), y_label (str),
#       width (int), height (int), indices (list[int] | None)
#   html_template: str        — Jinja2 HTML template source
#   output_path: str          — destination file path

import io

import polars as pl
from bokeh.embed import components
from bokeh.models import CDSView, ColumnDataSource, IndexFilter, Legend, LegendItem
from bokeh.plotting import figure
from bokeh.resources import CDN
from bokeh.transform import dodge
from jinja2 import Template

_DEFAULT_PALETTE = [
    "#4C72B0", "#DD8452", "#2ca02c",
    "#9467bd", "#e377c2", "#8c564b",
    "#17becf", "#bcbd22",
]

# ── Build one ColumnDataSource per source_key ────────────────────────────────
# Charts sharing a source_key reference the same CDS instance, so
# hover and selection events are automatically linked across those panels.

sources = {}
dfs = {}
for _key, _raw in frames.items():
    _df = pl.read_ipc(io.BytesIO(_raw))
    dfs[_key] = _df
    sources[_key] = ColumnDataSource({col: _df[col].to_list() for col in _df.columns})


def _make_view(indices):
    """Return a CDSView with an IndexFilter when indices are specified, else None."""
    if indices is None:
        return None
    return CDSView(filter=IndexFilter(indices=list(indices)))


def build_grouped_bar(spec, source, df):
    """Dodge-based grouped bar chart from a wide-format DataFrame."""
    x_col = spec["x_col"]
    value_cols = spec["value_cols"]
    x_vals = df[x_col].to_list()
    n = len(value_cols)
    bar_width = 0.8 / n
    offsets = [(i - (n - 1) / 2) * bar_width for i in range(n)]
    palette = _DEFAULT_PALETTE[:n]
    view = _make_view(spec["indices"])

    fig = figure(
        x_range=x_vals,
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save",
    )

    legend_items = []
    for col, offset, color in zip(value_cols, offsets, palette):
        kw = dict(
            x=dodge(x_col, offset, range=fig.x_range),
            top=col,
            width=bar_width * 0.9,
            source=source,
            fill_color=color,
            line_color="white",
        )
        if view is not None:
            kw["view"] = view
        r = fig.vbar(**kw)
        legend_items.append(LegendItem(label=col, renderers=[r]))

    fig.add_layout(Legend(items=legend_items), "right")
    fig.xaxis.major_label_orientation = 1.0
    fig.yaxis.axis_label = spec["y_label"]
    fig.xgrid.grid_line_color = None
    return fig


def build_line_multi(spec, source, df):
    """One line per value column, sharing the same ColumnDataSource.

    CDSView/IndexFilter is incompatible with connected glyphs (E-1024), so
    index filtering is handled differently per glyph type:
      - Line:    restrict figure x_range to the filtered x values; Bokeh only
                 renders segments whose endpoints are within the categorical range.
      - Scatter: apply CDSView+IndexFilter normally (discrete glyph, no issue).
    """
    x_col = spec["x_col"]
    value_cols = spec["value_cols"]
    x_vals = df[x_col].to_list()
    palette = _DEFAULT_PALETTE[:len(value_cols)]
    indices = spec["indices"]

    if indices is not None:
        display_x = [x_vals[i] for i in indices]
        scatter_view = CDSView(filter=IndexFilter(indices=list(indices)))
    else:
        display_x = x_vals
        scatter_view = None

    fig = figure(
        x_range=display_x,
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools="pan,wheel_zoom,box_zoom,reset,save",
    )

    legend_items = []
    for col, color in zip(value_cols, palette):
        r = fig.line(x=x_col, y=col, source=source, line_color=color, line_width=2)
        scatter_kw = dict(x=x_col, y=col, source=source, fill_color=color, size=6, line_color="white")
        if scatter_view is not None:
            scatter_kw["view"] = scatter_view
        fig.scatter(**scatter_kw)
        legend_items.append(LegendItem(label=col, renderers=[r]))

    fig.add_layout(Legend(items=legend_items), "right")
    fig.xaxis.major_label_orientation = 0.8
    fig.yaxis.axis_label = spec["y_label"]
    return fig


# ── Dispatch table ───────────────────────────────────────────────────────────

_BUILDERS = {
    "grouped_bar": build_grouped_bar,
    "line_multi": build_line_multi,
}

# ── Build all figures ────────────────────────────────────────────────────────

figures_list = []
for spec in chart_specs:
    builder = _BUILDERS.get(spec["chart_type"])
    if builder is None:
        raise ValueError(f"Unknown chart_type: {spec['chart_type']!r}")
    key = spec["source_key"]
    figures_list.append(builder(spec, sources[key], dfs[key]))

# ── Combine all figures into a single Bokeh script ───────────────────────────

script, divs = components(figures_list)
bokeh_js_url = CDN.js_files[0]
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""

# width is passed through as a layout hint for the Jinja template.
plots = [
    {"title": spec["title"], "div": div, "width": spec["width"]}
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
