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
#   has_filter=True  — RangeSlider CustomJS mutates source.data directly;
#                      both charts share the same CDS so both update together.
#                      No CDSView used (avoids shared-CDSView renderer issues).
#   has_filter=False — Charts share a CDS with box_select/lasso_select tools;
#                      Bokeh automatically links selection across figures.

import io
import os

import polars as pl
from bokeh.embed import components
from bokeh.models import (
    CDSView, ColumnDataSource, CustomJS, FactorRange,
    HoverTool, IndexFilter, Legend, LegendItem, RangeSlider,
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


def _make_view(indices):
    if indices is None:
        return None
    return CDSView(filter=IndexFilter(indices=list(indices)))


# ── Chart builders ───────────────────────────────────────────────────────────

_LINK_TOOLS = "pan,wheel_zoom,box_zoom,box_select,lasso_select,tap,reset,save"
_FILTER_TOOLS = "pan,wheel_zoom,box_zoom,reset,save"


def build_grouped_bar(spec, source, df, filter_mode=False):
    """Dodge-based grouped bar from a wide-format DataFrame."""
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
        tools=_FILTER_TOOLS if filter_mode else _LINK_TOOLS,
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


def build_line_multi(spec, source, df, filter_mode=False):
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
        tools=_FILTER_TOOLS if filter_mode else _LINK_TOOLS,
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


def build_hbar(spec, source, df, filter_mode=False):
    """Horizontal bar; x_col is the category column (rendered on y-axis)."""
    x_col = spec["x_col"]
    value_col = spec["value_cols"][0]
    categories = df[x_col].to_list()
    palette = _DEFAULT_PALETTE[:len(categories)]
    view = _make_view(spec["indices"])

    fig = figure(
        y_range=categories,
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools=_FILTER_TOOLS if filter_mode else _LINK_TOOLS,
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


def build_scatter_plot(spec, source, df, filter_mode=False):
    """Numeric x/y scatter; x_col is the x-axis column, value_cols[0] is y."""
    x_col = spec["x_col"]
    y_col = spec["value_cols"][0]
    view = _make_view(spec["indices"])

    hover = HoverTool(tooltips=[(col, f"@{{{col}}}") for col in df.columns])

    fig = figure(
        height=spec["height"],
        sizing_mode="stretch_width",
        title=spec["title"],
        toolbar_location="above",
        tools=[hover] + (_FILTER_TOOLS if filter_mode else _LINK_TOOLS).split(","),
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

bokeh_js_urls = CDN.js_files
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""
template = Template(html_template)

nav_pages = [{"label": p["nav_label"], "href": p["slug"] + ".html"} for p in pages]

os.makedirs(output_dir, exist_ok=True)

for page in pages:
    has_filter = page.get("has_filter", False)

    # Build fresh CDS per page. Charts sharing a source_key get the same
    # instance, so Bokeh links their selection/hover automatically.
    sources = _build_sources(page["specs"])

    if has_filter and page["specs"]:
        # ── Interactive filter via direct CDS data mutation ──────────────────
        # We do NOT use CDSView here. Sharing one CDSView across multiple
        # GlyphRenderers causes Bokeh to silently drop those charts.
        # Instead, CustomJS slices source.data directly; because both charts
        # reference the same ColumnDataSource object, both update together.
        primary_key = page["specs"][0]["source_key"]
        primary_df = _all_dfs[primary_key]
        n_rows = len(primary_df)
        x_col_name = page["specs"][0]["x_col"]
        labels = primary_df[x_col_name].to_list()

        # full_data is passed to CustomJS as a JS const — the canonical dataset.
        full_data = {col: primary_df[col].to_list() for col in primary_df.columns}

        # Replace the source for this key with one CustomJS can mutate.
        sources[primary_key] = ColumnDataSource({k: list(v) for k, v in full_data.items()})

        # Build figures first so their x_range objects exist for the callback.
        figures_list = [
            _BUILDERS[spec["chart_type"]](
                spec, sources[spec["source_key"]], _all_dfs[spec["source_key"]],
                filter_mode=True,
            )
            for spec in page["specs"]
        ]

        # Collect any FactorRange x_ranges from the figures so the callback
        # can shrink the categorical axis to match the filtered rows.
        factor_ranges = [
            f.x_range for f in figures_list
            if isinstance(f.x_range, FactorRange)
        ]

        slider = RangeSlider(
            start=0, end=n_rows - 1,
            value=(0, n_rows - 1),
            step=1,
            title=f"Filter: {x_col_name}  (0 = {labels[0]},  {n_rows - 1} = {labels[-1]})",
            sizing_mode="stretch_width",
        )

        cb_args = dict(source=sources[primary_key], full=full_data, x_col=x_col_name)
        # Pass each FactorRange so JS can update its factors list.
        for i, fr in enumerate(factor_ranges):
            cb_args[f"fr{i}"] = fr
        update_factors = "\n".join(
            f"            fr{i}.factors = sliced[x_col];"
            for i in range(len(factor_ranges))
        )
        callback = CustomJS(
            args=cb_args,
            code=f"""
            const lo = Math.round(cb_obj.value[0]);
            const hi = Math.round(cb_obj.value[1]);
            const sliced = {{}};
            for (const [key, val] of full) {{
                sliced[key] = val.slice(lo, hi + 1);
            }}
            source.data = sliced;
{update_factors}
        """,
        )
        slider.js_on_change("value", callback)

    else:
        # ── Linked selection via shared CDS (no filter widget) ───────────────
        # BoxSelectTool / LassoSelectTool highlight the same row indices in all
        # figures sharing a CDS — Bokeh does this automatically.
        slider = None
        figures_list = [
            _BUILDERS[spec["chart_type"]](
                spec, sources[spec["source_key"]], _all_dfs[spec["source_key"]],
                filter_mode=False,
            )
            for spec in page["specs"]
        ]

    # Include the slider in components() so its JS lives in the page script.
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
        bokeh_js_urls=bokeh_js_urls,
        bokeh_css_url=bokeh_css_url,
        plot_script=script,
        plots=plots,
    )

    out_path = os.path.join(output_dir, page["slug"] + ".html")
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(html)
    print(f"  wrote {out_path}")
