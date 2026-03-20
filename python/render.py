# render.py — called from Rust via PyO3
# Variables injected by Rust:
#   frames: dict[str, bytes]  — Arrow IPC bytes keyed by source_key (all pages)
#   pages: list[dict]         — each dict has keys:
#       title (str), nav_label (str), slug (str), has_filter (bool),
#       specs: list[dict] where each spec has:
#           chart_type (str), title (str), source_key (str),
#           x_col (str), value_cols (list[str]), y_label (str),
#           width (int), height (int), indices (list[int] | None)
#   html_template: str        — Jinja2 HTML template source
#   output_dir: str           — directory to write <slug>.html files into
#
# Shared CDS linking strategy (per page type):
#   has_filter=True  — RangeSlider drives a shared IndexFilter on one CDS;
#                      all charts using that source_key filter together.
#   has_filter=False — Charts share a CDS with box_select/lasso_select tools;
#                      selecting rows in one figure highlights the same rows
#                      in the other automatically (no CustomJS needed).

import io
import os

import polars as pl
from bokeh.embed import components
from bokeh.models import (
    CDSView, ColumnDataSource, CustomJS, HoverTool,
    IndexFilter, Legend, LegendItem, RangeSlider,
)
from bokeh.plotting import figure
from bokeh.resources import CDN
from bokeh.transform import dodge, factor_cmap
from jinja2 import Template

_DEFAULT_PALETTE = [
    "#4C72B0", "#DD8452", "#2ca02c",
    "#9467bd", "#e377c2", "#8c564b",
    "#17becf", "#bcbd22",
]

# ── Pre-parse all DataFrames once ────────────────────────────────────────────

_all_dfs = {}
for _key, _raw in frames.items():
    _all_dfs[_key] = pl.read_ipc(io.BytesIO(_raw))


def _build_sources(page_specs):
    """Build fresh ColumnDataSource objects scoped to this page's source_keys.

    Charts that share a source_key get the SAME CDS instance, enabling
    Bokeh's automatic linked selection and hover across those figures.
    components() will only serialize data reachable from this page's figures.
    """
    sources = {}
    for spec in page_specs:
        key = spec["source_key"]
        if key not in sources:
            df = _all_dfs[key]
            sources[key] = ColumnDataSource({col: df[col].to_list() for col in df.columns})
    return sources


# ── Chart builders ───────────────────────────────────────────────────────────
# Each builder accepts an optional `shared_view` keyword argument.
# When provided (has_filter=True pages), it overrides spec["indices"] so all
# charts on the page respond to the same interactive IndexFilter.

_LINK_TOOLS = "pan,wheel_zoom,box_zoom,box_select,lasso_select,tap,reset,save"
_FILTER_TOOLS = "pan,wheel_zoom,box_zoom,reset,save"


def build_grouped_bar(spec, source, df, shared_view=None):
    """Dodge-based grouped bar from a wide-format DataFrame."""
    x_col = spec["x_col"]
    value_cols = spec["value_cols"]
    x_vals = df[x_col].to_list()
    n = len(value_cols)
    bar_width = 0.8 / n
    offsets = [(i - (n - 1) / 2) * bar_width for i in range(n)]
    palette = _DEFAULT_PALETTE[:n]
    view = shared_view or (
        CDSView(filter=IndexFilter(indices=list(spec["indices"])))
        if spec["indices"] is not None else None
    )
    tools = _FILTER_TOOLS if shared_view is not None else _LINK_TOOLS

    fig = figure(
        x_range=x_vals,
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools=tools,
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
            nonselection_fill_alpha=0.2,
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


def build_line_multi(spec, source, df, shared_view=None):
    """One line per value column, sharing the same ColumnDataSource.

    CDSView/IndexFilter is incompatible with connected glyphs (E-1024), so
    index filtering is handled differently per glyph type:
      - Line:    restrict figure x_range to the filtered x values.
      - Scatter: apply CDSView+IndexFilter (discrete glyph, no issue).
    """
    x_col = spec["x_col"]
    value_cols = spec["value_cols"]
    x_vals = df[x_col].to_list()
    palette = _DEFAULT_PALETTE[:len(value_cols)]
    indices = spec["indices"]
    tools = _FILTER_TOOLS if shared_view is not None else _LINK_TOOLS

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
        tools=tools,
    )

    legend_items = []
    for col, color in zip(value_cols, palette):
        r = fig.line(x=x_col, y=col, source=source, line_color=color, line_width=2)
        scatter_kw = dict(
            x=x_col, y=col, source=source,
            fill_color=color, size=6, line_color="white",
            nonselection_fill_alpha=0.2,
        )
        if scatter_view is not None:
            scatter_kw["view"] = scatter_view
        fig.scatter(**scatter_kw)
        legend_items.append(LegendItem(label=col, renderers=[r]))

    fig.add_layout(Legend(items=legend_items), "right")
    fig.xaxis.major_label_orientation = 0.8
    fig.yaxis.axis_label = spec["y_label"]
    return fig


def build_hbar(spec, source, df, shared_view=None):
    """Horizontal bar; x_col is the category column (rendered on y-axis)."""
    x_col = spec["x_col"]
    value_col = spec["value_cols"][0]
    categories = df[x_col].to_list()
    palette = _DEFAULT_PALETTE[:len(categories)]
    view = shared_view or (
        CDSView(filter=IndexFilter(indices=list(spec["indices"])))
        if spec["indices"] is not None else None
    )
    tools = _FILTER_TOOLS if shared_view is not None else _LINK_TOOLS

    fig = figure(
        y_range=categories,
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools=tools,
    )

    kw = dict(
        y=x_col,
        right=value_col,
        height=0.6,
        source=source,
        fill_color=factor_cmap(x_col, palette=palette, factors=categories),
        line_color="white",
        nonselection_fill_alpha=0.2,
    )
    if view is not None:
        kw["view"] = view
    fig.hbar(**kw)

    fig.xaxis.axis_label = spec["y_label"]
    fig.ygrid.grid_line_color = None
    return fig


def build_scatter_plot(spec, source, df, shared_view=None):
    """Numeric x/y scatter; x_col is the x-axis column, value_cols[0] is y.

    HoverTool shows all CDS columns (including label columns like 'month').
    """
    x_col = spec["x_col"]
    y_col = spec["value_cols"][0]
    view = shared_view or (
        CDSView(filter=IndexFilter(indices=list(spec["indices"])))
        if spec["indices"] is not None else None
    )
    tools = _FILTER_TOOLS if shared_view is not None else _LINK_TOOLS

    hover = HoverTool(tooltips=[(col, f"@{{{col}}}") for col in df.columns])

    fig = figure(
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools=[hover] + tools.split(","),
    )

    kw = dict(
        x=x_col,
        y=y_col,
        source=source,
        size=10,
        fill_color=_DEFAULT_PALETTE[0],
        fill_alpha=0.8,
        line_color="white",
        nonselection_fill_alpha=0.15,
    )
    if view is not None:
        kw["view"] = view
    fig.scatter(**kw)

    fig.xaxis.axis_label = x_col
    fig.yaxis.axis_label = spec["y_label"]
    return fig


# ── Dispatch table ───────────────────────────────────────────────────────────

_BUILDERS = {
    "grouped_bar":  build_grouped_bar,
    "line_multi":   build_line_multi,
    "hbar":         build_hbar,
    "scatter_plot": build_scatter_plot,
}

# ── Render one HTML file per page ────────────────────────────────────────────

bokeh_js_url = CDN.js_files[0]
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""
template = Template(html_template)

nav_pages = [{"label": p["nav_label"], "href": p["slug"] + ".html"} for p in pages]

os.makedirs(output_dir, exist_ok=True)

for page in pages:
    # Fresh CDS per page — only the source_keys this page references.
    # Charts sharing a source_key get the same CDS instance within a page,
    # so Bokeh's selection/hover linking is active between them.
    sources = _build_sources(page["specs"])

    has_filter = page.get("has_filter", False)
    slider = None
    shared_view = None

    if has_filter and page["specs"]:
        # Build a shared IndexFilter + RangeSlider for the primary source_key.
        # Both charts on this page use the same CDSView, so moving the slider
        # filters them simultaneously from a single ColumnDataSource.
        primary_key = page["specs"][0]["source_key"]
        primary_df = _all_dfs[primary_key]
        n_rows = len(primary_df)
        x_col_name = page["specs"][0]["x_col"]
        labels = primary_df[x_col_name].to_list()

        shared_filter = IndexFilter(indices=list(range(n_rows)))
        shared_view = CDSView(filter=shared_filter)

        slider = RangeSlider(
            start=0, end=n_rows - 1,
            value=(0, n_rows - 1),
            step=1,
            title=f"Filter by {x_col_name} (0 = {labels[0]}, {n_rows - 1} = {labels[-1]})",
            sizing_mode="stretch_width",
        )
        # CustomJS: update the shared IndexFilter when the slider moves.
        # Because shared_view references shared_filter, and both charts use
        # shared_view, both update immediately from one CDS.
        callback = CustomJS(args=dict(f=shared_filter), code="""
            const lo = Math.round(cb_obj.value[0]);
            const hi = Math.round(cb_obj.value[1]);
            f.indices = Array.from({length: hi - lo + 1}, (_, i) => lo + i);
        """)
        slider.js_on_change("value", callback)

    figures_list = []
    for spec in page["specs"]:
        builder = _BUILDERS.get(spec["chart_type"])
        if builder is None:
            raise ValueError(f"Unknown chart_type: {spec['chart_type']!r}")
        key = spec["source_key"]
        # Pass shared_view only when this spec's source_key is the filtered one.
        view_arg = (
            shared_view
            if has_filter and key == page["specs"][0]["source_key"]
            else None
        )
        figures_list.append(builder(spec, sources[key], _all_dfs[key], shared_view=view_arg))

    # Include the slider in components() so its JS is part of the page script.
    all_models = ([slider] if slider else []) + figures_list
    script, all_divs = components(all_models)

    plots = []
    if slider:
        plots.append({"title": "", "div": all_divs[0], "width": 1000, "kind": "widget"})
        chart_divs = all_divs[1:]
    else:
        chart_divs = all_divs

    plots += [
        {"title": spec["title"], "div": div, "width": spec["width"], "kind": "chart"}
        for spec, div in zip(page["specs"], chart_divs)
    ]

    this_nav = [
        {**entry, "active": entry["href"] == page["slug"] + ".html"}
        for entry in nav_pages
    ]

    html = template.render(
        title=page["title"],
        nav_pages=this_nav,
        bokeh_js_url=bokeh_js_url,
        bokeh_css_url=bokeh_css_url,
        plot_script=script,
        plots=plots,
    )

    out_path = os.path.join(output_dir, page["slug"] + ".html")
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(html)
    print(f"  wrote {out_path}")
