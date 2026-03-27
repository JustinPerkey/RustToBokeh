# render.py — called from Rust via PyO3
# Variables injected by Rust:
#   frames: dict[str, bytes]   — Arrow IPC bytes keyed by source name
#   pages: list[dict]          — each page has slug, title, grid_cols, modules, filters
#   nav_links: list[dict]      — slug + label for every page (navigation)
#   html_template: str         — Jinja2 HTML template source
#   output_dir: str            — output directory path

import io
import os

import polars as pl
from bokeh.embed import components
import math

from bokeh.models import (
    AllIndices,
    BooleanFilter,
    CDSView,
    ColumnDataSource,
    CustomJS,
    DatetimeRangeSlider,
    DatetimeTickFormatter,
    FactorRange,
    GroupFilter,
    HoverTool,
    IndexFilter,
    IntersectionFilter,
    Legend,
    LegendItem,
    NumeralTickFormatter,
    Range1d,
    RangeSlider,
    RangeTool,
    Select,
    Slider,
    Switch,
)
from bokeh.plotting import figure
from bokeh.resources import CDN
from bokeh.transform import factor_cmap
from jinja2 import Template

_PALETTE = [
    "#4C72B0", "#DD8452", "#55A868", "#C44E52",
    "#8172B3", "#937860", "#DA8BC3", "#8C8C8C",
    "#CCB974", "#64B5CD",
]

# ── Time scale helpers ───────────────────────────────────────────────────────

_TIME_SCALE_FMT = {
    "milliseconds": "%H:%M:%S.%3N",
    "seconds":      "%H:%M:%S",
    "minutes":      "%H:%M",
    "hours":        "%m/%d %H:%M",
    "days":         "%Y-%m-%d",
    "months":       "%b %Y",
    "years":        "%Y",
}


def _datetime_formatter(time_scale):
    """Return a DatetimeTickFormatter appropriate for the given time_scale string."""
    fmt = _TIME_SCALE_FMT.get(time_scale, "%Y-%m-%d")
    return DatetimeTickFormatter(
        milliseconds=fmt,
        seconds=fmt,
        minsec=fmt,
        minutes=fmt,
        hourmin=fmt,
        hours=fmt,
        days=fmt,
        months=fmt,
        years=fmt,
    )

# ── Deserialize all frames once ─────────────────────────────────────────────

dataframes = {}
for key, raw in frames.items():
    dataframes[key] = pl.read_ipc(io.BytesIO(raw))

# ── Shared flat source helper ───────────────────────────────────────────────
# Line and scatter charts use flat CDS ({col: list}) and can share a source
# when they reference the same source_key.  Grouped bar and hbar use different
# CDS shapes so they keep their own cache keys.


def _get_flat_source(key, source_cache):
    if key in source_cache:
        return source_cache[key]
    df = dataframes[key]
    data = {col: df[col].to_list() for col in df.columns}
    source = ColumnDataSource(data)
    source_cache[key] = source
    return source


# ── Visual customisation helpers ─────────────────────────────────────────────


def _resolve_palette(palette_spec, n):
    """Return a list of exactly n colors from a palette spec dict (or None)."""
    if palette_spec is None:
        base = _PALETTE
        return (base * (n // len(base) + 1))[:n] if n > len(base) else base[:n]
    kind = palette_spec["kind"]
    if kind == "named":
        import bokeh.palettes as _bp
        name = palette_spec["value"]
        if name in _bp.all_palettes:
            sizes = sorted(_bp.all_palettes[name].keys())
            best = next((s for s in sizes if s >= n), sizes[-1])
            colors = list(_bp.all_palettes[name][best])
            if len(colors) > n:
                step = max(1, len(colors) // n)
                colors = [colors[i * step] for i in range(n)]
            return colors[:n]
    if kind == "custom":
        colors = palette_spec["value"]
        return (colors * (n // len(colors) + 1))[:n] if n > len(colors) else colors[:n]
    return _PALETTE[:n]


def _build_hover_tool(spec):
    """Build a HoverTool from the spec's tooltips list, or return None."""
    tt_spec = spec.get("tooltips")
    if not tt_spec:
        return None
    tooltips = []
    formatters = {}
    for field in tt_spec:
        col = field["column"]
        label = field["label"]
        fmt = field["format"]
        dec = field.get("decimals")
        if fmt == "text":
            fmt_str = f"@{{{col}}}"
        elif fmt == "number":
            d = dec if dec is not None else 2
            fmt_str = f"@{{{col}}}{{0.{'0' * d}}}"
        elif fmt == "percent":
            d = dec if dec is not None else 1
            fmt_str = f"@{{{col}}}{{0.{'0' * d}%}}"
        elif fmt == "currency":
            fmt_str = f"@{{{col}}}{{$0,0}}"
        elif fmt == "datetime":
            ts = field.get("time_scale", "days")
            strftime_fmt = _TIME_SCALE_FMT.get(ts, "%Y-%m-%d")
            fmt_str = f"@{{{col}}}{{custom}}"
            formatters[f"@{{{col}}}"] = "datetime"
            # Override with the specific strftime format inside the tooltip string
            fmt_str = f"@{{{col}}}{{{strftime_fmt}}}"
            formatters[f"@{{{col}}}"] = "datetime"
        else:
            fmt_str = f"@{{{col}}}"
        tooltips.append((label, fmt_str))
    return HoverTool(tooltips=tooltips, formatters=formatters)


def _apply_axis_config(axis_dict, bokeh_axis, range_obj, grid_obj):
    """Apply an axis config dict to a Bokeh axis, range, and grid object."""
    if axis_dict is None:
        return
    # Datetime tick formatter takes priority over numeral tick format
    if axis_dict.get("time_scale") is not None:
        bokeh_axis.formatter = _datetime_formatter(axis_dict["time_scale"])
    elif axis_dict.get("tick_format") is not None:
        bokeh_axis.formatter = NumeralTickFormatter(format=axis_dict["tick_format"])
    # Label rotation (degrees → radians)
    if axis_dict.get("label_rotation") is not None:
        bokeh_axis.major_label_orientation = math.radians(axis_dict["label_rotation"])
    # Grid visibility
    if not axis_dict.get("show_grid", True):
        grid_obj.grid_line_color = None
    # Range start/end and pan bounds — only for numeric (non-FactorRange) axes
    if not isinstance(range_obj, FactorRange):
        if axis_dict.get("start") is not None:
            range_obj.start = axis_dict["start"]
        if axis_dict.get("end") is not None:
            range_obj.end = axis_dict["end"]
        bmin = axis_dict.get("bounds_min")
        bmax = axis_dict.get("bounds_max")
        if bmin is not None and bmax is not None:
            range_obj.bounds = (bmin, bmax)


def _figure_kw(spec, default_height=400, x_axis_type=None):
    """Return keyword arguments for figure() derived from a chart spec."""
    kw = {
        "title": spec["title"],
        "toolbar_location": "above",
        "height": spec["height"] if spec.get("height") else default_height,
    }
    if spec.get("width"):
        kw["width"] = spec["width"]
        kw["sizing_mode"] = "fixed"
    else:
        kw["sizing_mode"] = "stretch_width"
    if x_axis_type is not None:
        kw["x_axis_type"] = x_axis_type
    return kw


def _x_axis_time_scale(spec):
    """Return the time_scale string from the x_axis config, or None."""
    x_axis = spec.get("x_axis")
    if x_axis and x_axis.get("time_scale"):
        return x_axis["time_scale"]
    return None


# ── Chart builders ──────────────────────────────────────────────────────────
# Each builder receives (spec_dict, source_cache, view) and returns a figure.
# If view is not None, renderers attach it for CDSView-based filtering.


def build_grouped_bar(spec, source_cache, view=None):
    key = spec["source_key"]
    df = dataframes[key]
    x_col, group_col, value_col = spec["x_col"], spec["group_col"], spec["value_col"]

    source = _get_flat_source(key, source_cache)
    vkw = dict(view=view) if view else {}

    # Add factor tuples column for FactorRange if not already present
    factor_col = "_factors_" + x_col + "_" + group_col
    if factor_col not in source.data:
        source.data[factor_col] = [
            (str(x), str(g))
            for x, g in zip(source.data[x_col], source.data[group_col])
        ]

    groups = df[group_col].unique(maintain_order=True).to_list()
    palette = _resolve_palette(spec.get("palette"), len(groups))

    hover = _build_hover_tool(spec)
    tools = "pan,wheel_zoom,box_zoom,reset,save,box_select,tap"
    if hover is None:
        tools = "pan,wheel_zoom,box_zoom,reset,save,hover,box_select,tap"

    kw = _figure_kw(spec)
    kw["x_range"] = FactorRange(*source.data[factor_col])
    kw["tools"] = tools
    fig = figure(**kw)
    if hover:
        fig.add_tools(hover)

    fig.vbar(
        x=factor_col, top=value_col,
        width=spec.get("bar_width", 0.9),
        source=source,
        line_color="white",
        fill_color=factor_cmap(factor_col, palette=palette, factors=groups, start=1, end=2),
        selection_fill_color="firebrick",
        nonselection_fill_alpha=0.2,
        **vkw,
    )
    fig.x_range.range_padding = 0.1
    fig.xaxis.major_label_orientation = 1.0
    fig.xaxis.group_label_orientation = 0.5
    fig.yaxis.axis_label = spec.get("y_label", "")
    fig.xgrid.grid_line_color = None

    _apply_axis_config(spec.get("x_axis"), fig.xaxis[0], fig.x_range, fig.xgrid[0])
    _apply_axis_config(spec.get("y_axis"), fig.yaxis[0], fig.y_range, fig.ygrid[0])
    return fig


def build_line_multi(spec, source_cache, view=None, x_range=None):
    key = spec["source_key"]
    df = dataframes[key]
    x_col = spec["x_col"]
    y_cols = [c.strip() for c in spec["y_cols"].split(",")]

    source = _get_flat_source(key, source_cache)
    vkw = dict(view=view) if view else {}

    lw = spec.get("line_width", 2.5)
    pt_size = spec.get("point_size", 7)

    hover = _build_hover_tool(spec)
    tools = "pan,wheel_zoom,box_zoom,reset,save,box_select,tap"
    if hover is None:
        tools = "pan,wheel_zoom,box_zoom,reset,save,hover,box_select,tap"

    ts = _x_axis_time_scale(spec)
    if x_range is not None:
        # Range1d provided by a RangeTool spec — use it as the detail view
        kw = _figure_kw(spec, x_axis_type="datetime" if ts else None)
        kw["x_range"] = x_range
    elif ts:
        # Datetime x axis: use Range1d so Bokeh renders datetime ticks correctly
        vals = df[x_col].to_list()
        kw = _figure_kw(spec, x_axis_type="datetime")
        kw["x_range"] = Range1d(start=min(vals), end=max(vals))
    else:
        kw = _figure_kw(spec)
        kw["x_range"] = df[x_col].to_list()
    kw["tools"] = tools
    fig = figure(**kw)
    if hover:
        fig.add_tools(hover)

    palette = _resolve_palette(spec.get("palette"), len(y_cols))
    for i, col in enumerate(y_cols):
        color = palette[i % len(palette)]
        fig.line(x=x_col, y=col, source=source, line_width=lw,
                 color=color, legend_label=col, **vkw)
        fig.scatter(x=x_col, y=col, source=source, size=pt_size,
                    color=color, legend_label=col,
                    selection_color="firebrick",
                    nonselection_alpha=0.3,
                    **vkw)
    fig.yaxis.axis_label = spec.get("y_label", "")
    fig.legend.location = "top_left"
    fig.legend.click_policy = "hide"

    _apply_axis_config(spec.get("x_axis"), fig.xaxis[0], fig.x_range, fig.xgrid[0])
    _apply_axis_config(spec.get("y_axis"), fig.yaxis[0], fig.y_range, fig.ygrid[0])
    return fig


def build_hbar(spec, source_cache, view=None):
    key = spec["source_key"]
    df = dataframes[key]
    cat_col = spec["category_col"]
    val_col = spec["value_col"]

    source = _get_flat_source(key, source_cache)
    vkw = dict(view=view) if view else {}

    cats = df[cat_col].to_list()
    default_height = max(300, len(cats) * 40 + 80)

    hover = _build_hover_tool(spec)
    tools = "pan,wheel_zoom,box_zoom,reset,save,box_select,tap"
    if hover is None:
        tools = "pan,wheel_zoom,box_zoom,reset,save,hover,box_select,tap"

    kw = _figure_kw(spec, default_height=default_height)
    kw["y_range"] = list(reversed(cats))
    kw["tools"] = tools
    fig = figure(**kw)
    if hover:
        fig.add_tools(hover)

    fill_color = spec.get("color", "#4C72B0")
    fig.hbar(
        y=cat_col, right=val_col, height=0.7, source=source,
        line_color="white", fill_color=fill_color,
        selection_fill_color="firebrick",
        nonselection_fill_alpha=0.2,
        **vkw,
    )
    fig.xaxis.axis_label = spec.get("x_label", "")
    fig.ygrid.grid_line_color = None

    _apply_axis_config(spec.get("x_axis"), fig.xaxis[0], fig.x_range, fig.xgrid[0])
    _apply_axis_config(spec.get("y_axis"), fig.yaxis[0], fig.y_range, fig.ygrid[0])
    return fig


def build_scatter(spec, source_cache, view=None, x_range=None):
    key = spec["source_key"]
    x_col = spec["x_col"]
    y_col = spec["y_col"]

    source = _get_flat_source(key, source_cache)
    vkw = dict(view=view) if view else {}

    hover = _build_hover_tool(spec)
    tools = "pan,wheel_zoom,box_zoom,reset,save,box_select,tap"
    if hover is None:
        tools = "pan,wheel_zoom,box_zoom,reset,save,hover,box_select,tap"

    ts = _x_axis_time_scale(spec)
    kw = _figure_kw(spec, x_axis_type="datetime" if ts else None)
    if x_range is not None:
        kw["x_range"] = x_range
    kw["tools"] = tools
    fig = figure(**kw)
    if hover:
        fig.add_tools(hover)

    fig.scatter(
        x=x_col, y=y_col, source=source,
        size=spec.get("marker_size", 10),
        color=spec.get("color", "#4C72B0"),
        alpha=spec.get("alpha", 0.7),
        marker=spec.get("marker", "circle"),
        selection_color="firebrick",
        nonselection_alpha=0.2,
        **vkw,
    )
    fig.xaxis.axis_label = spec.get("x_label", "")
    fig.yaxis.axis_label = spec.get("y_label", "")

    _apply_axis_config(spec.get("x_axis"), fig.xaxis[0], fig.x_range, fig.xgrid[0])
    _apply_axis_config(spec.get("y_axis"), fig.yaxis[0], fig.y_range, fig.ygrid[0])
    return fig


def build_pie(spec, source_cache, view=None):
    """Build a pie or donut chart.

    When ``inner_radius`` is present in the spec the chart is rendered as a
    donut (annular_wedge); otherwise it is a solid pie (wedge).

    Pie charts compute their own ColumnDataSource (start/end angles derived
    from the raw values) so they do not share the flat source cache used by
    line and scatter charts.  CDSView-based filtering is therefore not
    supported — the ``view`` parameter is accepted for API compatibility but
    is silently ignored.
    """
    from math import pi as _PI

    key = spec["source_key"]
    df = dataframes[key]
    label_col = spec["label_col"]
    value_col = spec["value_col"]

    values = df[value_col].cast(float).to_list()
    labels = df[label_col].to_list()
    total = sum(values)
    n = len(values)
    palette = _resolve_palette(spec.get("palette"), n)
    angles = [v / total * 2 * _PI for v in values]

    source = ColumnDataSource({
        "labels": labels,
        "values": values,
        "angle": angles,
        "color": palette,
    })

    legend_side = spec.get("legend_side", "right")

    kw = {
        "title": spec["title"],
        "toolbar_location": "above",
        "height": spec["height"] if spec.get("height") else 400,
        "x_range": (-1.2, 1.2),
        "y_range": (-1.2, 1.2),
    }
    if spec.get("width"):
        kw["width"] = spec["width"]
        kw["sizing_mode"] = "fixed"
    else:
        kw["sizing_mode"] = "stretch_width"

    hover = _build_hover_tool(spec)
    if hover is None:
        kw["tools"] = "hover,save,reset"
        kw["tooltips"] = [("", "@labels"), ("Value", "@values")]
    else:
        kw["tools"] = "save,reset"

    fig = figure(**kw)
    if hover:
        fig.add_tools(hover)

    fig.axis.visible = False
    fig.grid.grid_line_color = None

    # Render one glyph per slice so each has its own renderer.
    # This allows click_policy="hide" to toggle individual slices rather
    # than hiding the entire chart.
    inner_radius = spec.get("inner_radius")
    legend_items = []
    start_angle = 0.0
    for i, (label, angle, color) in enumerate(zip(labels, angles, palette)):
        end_angle = start_angle + angle
        slice_view = CDSView(filter=IndexFilter(indices=[i]))
        if inner_radius:
            r = fig.annular_wedge(
                x=0, y=0,
                inner_radius=inner_radius,
                outer_radius=0.9,
                start_angle=start_angle,
                end_angle=end_angle,
                line_color="white",
                fill_color=color,
                source=source,
                view=slice_view,
            )
        else:
            r = fig.wedge(
                x=0, y=0,
                radius=0.9,
                start_angle=start_angle,
                end_angle=end_angle,
                line_color="white",
                fill_color=color,
                source=source,
                view=slice_view,
            )
        legend_items.append(LegendItem(label=label, renderers=[r]))
        start_angle = end_angle

    if spec.get("show_legend", True) is not False:
        legend = Legend(
            items=legend_items,
            label_text_font_size="10pt",
            click_policy="hide",
        )
        fig.add_layout(legend, legend_side)

    return fig


def build_range_tool_overview(rt_spec, source_cache, shared_x_range):
    """Build a compact navigator chart with a RangeTool attached.

    The returned figure shows ``rt_spec["y_column"]`` plotted over the full
    x extent.  A ``RangeTool`` overlay lets the user drag to update
    ``shared_x_range``, which is linked to the detail charts above.
    """
    source_key = rt_spec["source_key"]
    x_col = rt_spec["column"]
    y_col = rt_spec["y_column"]
    time_scale = rt_spec.get("time_scale")

    source = _get_flat_source(source_key, source_cache)

    kw = {
        "title": rt_spec["label"],
        "height": 130,
        "toolbar_location": None,
        "sizing_mode": "stretch_width",
    }
    if time_scale:
        kw["x_axis_type"] = "datetime"
        vals = source.data[x_col]
        kw["x_range"] = Range1d(start=min(vals), end=max(vals))
    else:
        kw["x_range"] = list(source.data[x_col])

    fig = figure(**kw)
    fig.line(x=x_col, y=y_col, source=source, color=_PALETTE[0], line_width=1)
    if time_scale:
        fig.xaxis.formatter = _datetime_formatter(time_scale)

    range_tool = RangeTool(x_range=shared_x_range)
    range_tool.overlay.fill_color = "#4C72B0"
    range_tool.overlay.fill_alpha = 0.2
    fig.add_tools(range_tool)

    return fig


def build_histogram(spec, source_cache, view=None):
    """Render a histogram from a pre-computed histogram DataFrame.

    Expects a DataFrame produced by compute_histogram() (Rust side) with
    columns: left, right, count, pdf, cdf.  The 'display' field in the spec
    selects which statistic to render:
      - 'count' / 'pdf' : bar chart (quad glyph)
      - 'cdf'           : step line chart (line glyph on step coordinates)
    """
    key = spec["source_key"]
    df = dataframes[key]
    display = spec.get("display", "count")

    left = df["left"].to_list()
    right = df["right"].to_list()
    fill_color = spec.get("color", "#4C72B0")
    line_color = spec.get("line_color", "white")
    alpha = spec.get("alpha", 0.7)

    hover = _build_hover_tool(spec)
    tools = "pan,wheel_zoom,box_zoom,reset,save,box_select,tap"
    kw = _figure_kw(spec)
    kw["tools"] = tools

    if display == "cdf":
        # Build step-line coordinates: prepend (left[0], 0.0) so the curve
        # starts from the baseline, then step up at each bin's right edge.
        cdf_vals = df["cdf"].to_list()
        step_x = [left[0]] + right
        step_y = [0.0] + cdf_vals
        source = ColumnDataSource(dict(x=step_x, y=step_y))

        if hover is None:
            kw["tooltips"] = [("Value", "@x{0.00}"), ("Cumulative", "@y{0.000}")]
        fig = figure(**kw)
        if hover:
            fig.add_tools(hover)

        fig.line(
            x="x", y="y", source=source,
            line_width=2.5, line_color=fill_color, line_alpha=min(alpha + 0.3, 1.0),
        )
        fig.yaxis.axis_label = spec.get("y_label", "Cumulative Fraction")

    elif display == "pdf":
        pdf_vals = df["pdf"].to_list()
        source = ColumnDataSource(dict(left=left, right=right, pdf=pdf_vals))

        if hover is None:
            kw["tooltips"] = [
                ("Range", "@left{0.00} – @right{0.00}"),
                ("Density", "@pdf{0.0000}"),
            ]
        fig = figure(**kw)
        if hover:
            fig.add_tools(hover)

        fig.quad(
            top="pdf", bottom=0, left="left", right="right", source=source,
            fill_color=fill_color, line_color=line_color, fill_alpha=alpha,
            selection_fill_color="firebrick", nonselection_fill_alpha=0.2,
        )
        fig.yaxis.axis_label = spec.get("y_label", "Density")

    else:  # count (default)
        count_vals = df["count"].to_list()
        source = ColumnDataSource(dict(left=left, right=right, count=count_vals))

        if hover is None:
            kw["tooltips"] = [
                ("Range", "@left{0.00} – @right{0.00}"),
                ("Count", "@count"),
            ]
        fig = figure(**kw)
        if hover:
            fig.add_tools(hover)

        fig.quad(
            top="count", bottom=0, left="left", right="right", source=source,
            fill_color=fill_color, line_color=line_color, fill_alpha=alpha,
            selection_fill_color="firebrick", nonselection_fill_alpha=0.2,
        )
        fig.yaxis.axis_label = spec.get("y_label", "Count")

    fig.xaxis.axis_label = spec.get("x_label", "")
    _apply_axis_config(spec.get("x_axis"), fig.xaxis[0], fig.x_range, fig.xgrid[0])
    _apply_axis_config(spec.get("y_axis"), fig.yaxis[0], fig.y_range, fig.ygrid[0])
    return fig


_BUILDERS = {
    "grouped_bar": build_grouped_bar,
    "line_multi": build_line_multi,
    "hbar": build_hbar,
    "scatter": build_scatter,
    "pie": build_pie,
    "histogram": build_histogram,
}

# ── Non-chart module builders ────────────────────────────────────────────────


def _build_paragraph_html(mod):
    """Render a paragraph module as a styled HTML string."""
    title_html = (
        f'<h3 class="module-title">{mod["title"]}</h3>'
        if mod.get("has_title") else ""
    )
    paras = "".join(
        f"<p>{para.strip()}</p>"
        for para in mod["text"].split("\n\n")
        if para.strip()
    )
    return f'<div class="paragraph-module">{title_html}{paras}</div>'


def _format_cell(val, col):
    """Format a single cell value according to the column's format spec."""
    fmt = col["format"]
    if val is None:
        return ""
    if fmt == "text":
        return str(val)
    if fmt == "number":
        return f"{float(val):.{col['decimals']}f}"
    if fmt == "currency":
        return f"{col['symbol']}{float(val):,.{col['decimals']}f}"
    if fmt == "percent":
        return f"{float(val):.{col['decimals']}f}%"
    return str(val)


def _build_table_html(mod, dfs):
    """Render a table module as a styled HTML string."""
    df = dfs[mod["source_key"]]
    cols = mod["columns"]
    headers = "".join(f"<th>{c['label']}</th>" for c in cols)
    rows = []
    for i in range(len(df)):
        cells = "".join(
            f"<td>{_format_cell(df[c['key']][i], c)}</td>"
            for c in cols
        )
        rows.append(f"<tr>{cells}</tr>")
    body = "".join(rows)
    return (
        f'<div class="table-module">'
        f'<h3 class="module-title">{mod["title"]}</h3>'
        f'<div class="table-wrapper">'
        f"<table>"
        f"<thead><tr>{headers}</tr></thead>"
        f"<tbody>{body}</tbody>"
        f"</table>"
        f"</div>"
        f"</div>"
    )


# ── Filter setup ─────────────────────────────────────────────────────────────
# Creates Bokeh filter objects (GroupFilter, BooleanFilter, IndexFilter) and
# CDSView instances.  Each filter kind maps to a specific Bokeh filter model:
#
#   Range     → BooleanFilter  (mask: lo <= col[i] <= hi)
#   Select    → GroupFilter    (column_name + group value)
#   Threshold → BooleanFilter  (mask: col[i] >= value or col[i] <= value)
#   TopN      → IndexFilter    (sorted indices, first N)
#
# Multiple filters on the same source_key are combined via IntersectionFilter.
# A CDSView wrapping that combined filter is passed to chart renderers.


def build_filter_objects(page_filters, source_cache):
    """Build Bokeh filter objects, CDSViews, and widgets from filter specs.

    Returns (views, widgets, date_range_sliders) where:
      views:              dict[source_key → CDSView]
      widgets:            list[Bokeh model]  (for embedding via components())
      date_range_sliders: list of (source_key, col_name, DatetimeRangeSlider)
    """
    # Collect individual filter objects per source_key
    filters_by_source = {}  # source_key → list[Filter]
    widgets = []
    date_range_sliders = []

    for filt in page_filters:
        source_key = filt["source_key"]
        col_name = filt["column"]
        kind = filt["kind"]

        source = source_cache.get(source_key)
        if source is None:
            continue

        n = len(list(source.data.values())[0])

        if source_key not in filters_by_source:
            filters_by_source[source_key] = []

        if kind == "range":
            # BooleanFilter driven by a RangeSlider
            bf = BooleanFilter(booleans=[True] * n)
            slider = RangeSlider(
                start=filt["min"], end=filt["max"],
                value=(filt["min"], filt["max"]),
                step=filt["step"],
                title=filt["label"],
                sizing_mode="stretch_width",
            )
            callback = CustomJS(
                args=dict(bf=bf, source=source, col=col_name),
                code="""
                    const [lo, hi] = cb_obj.value;
                    const data = source.data[col];
                    const bools = data.map(v => v >= lo && v <= hi);
                    bf.booleans = bools;
                    source.change.emit();
                """,
            )
            slider.js_on_change("value", callback)
            filters_by_source[source_key].append(bf)
            widgets.append(slider)

        elif kind == "select":
            # BooleanFilter driven by a Select dropdown.
            # We use BooleanFilter rather than GroupFilter directly so that
            # "(All)" can show every row (GroupFilter only matches one value).
            # The GroupFilter model is still used conceptually — the callback
            # implements group-matching logic via the boolean mask.
            options = filt["options"]
            bf = BooleanFilter(booleans=[True] * n)
            select = Select(
                value="(All)",
                options=["(All)"] + options,
                title=filt["label"],
                sizing_mode="stretch_width",
            )
            callback = CustomJS(
                args=dict(bf=bf, source=source, col=col_name),
                code="""
                    const val = cb_obj.value;
                    const data = source.data[col];
                    if (val === "(All)") {
                        bf.booleans = data.map(() => true);
                    } else {
                        bf.booleans = data.map(v => v === val);
                    }
                    source.change.emit();
                """,
            )
            select.js_on_change("value", callback)
            filters_by_source[source_key].append(bf)
            widgets.append(select)

        elif kind == "group":
            # GroupFilter driven by a Select dropdown (no "All" option).
            # Uses Bokeh's native GroupFilter model directly.
            options = filt["options"]
            gf = GroupFilter(column_name=col_name, group=options[0])
            select = Select(
                value=options[0],
                options=options,
                title=filt["label"],
                sizing_mode="stretch_width",
            )
            callback = CustomJS(
                args=dict(gf=gf, source=source),
                code="""
                    gf.group = cb_obj.value;
                    source.change.emit();
                """,
            )
            select.js_on_change("value", callback)
            filters_by_source[source_key].append(gf)
            widgets.append(select)

        elif kind == "threshold":
            # BooleanFilter driven by a Switch toggle
            threshold = filt["value"]
            above = filt["above"]
            # Start unfiltered (all visible)
            bf = BooleanFilter(booleans=[True] * n)
            switch = Switch(active=False)
            callback = CustomJS(
                args=dict(bf=bf, source=source, col=col_name,
                          threshold=threshold, above=above),
                code="""
                    const data = source.data[col];
                    if (cb_obj.active) {
                        bf.booleans = data.map(v => above ? v >= threshold : v <= threshold);
                    } else {
                        bf.booleans = data.map(() => true);
                    }
                    source.change.emit();
                """,
            )
            switch.js_on_change("active", callback)
            filters_by_source[source_key].append(bf)
            widgets.append({"switch": switch, "label": filt["label"]})

        elif kind == "top_n":
            # IndexFilter driven by a Slider
            max_n = filt["max_n"]
            descending = filt["descending"]
            idx_filter = IndexFilter(indices=list(range(n)))
            slider = Slider(
                start=1, end=max_n, value=max_n, step=1,
                title=filt["label"],
                sizing_mode="stretch_width",
            )
            callback = CustomJS(
                args=dict(idx_filter=idx_filter, source=source,
                          col=col_name, descending=descending),
                code="""
                    const n = cb_obj.value;
                    const data = source.data[col];
                    const indexed = data.map((v, i) => ({v: v, i: i}));
                    if (descending) {
                        indexed.sort((a, b) => b.v - a.v);
                    } else {
                        indexed.sort((a, b) => a.v - b.v);
                    }
                    idx_filter.indices = indexed.slice(0, n).map(x => x.i);
                    source.change.emit();
                """,
            )
            slider.js_on_change("value", callback)
            filters_by_source[source_key].append(idx_filter)
            widgets.append(slider)

        elif kind == "date_range":
            # BooleanFilter driven by a DatetimeRangeSlider.
            # Column values must be milliseconds since the Unix epoch.
            # DatetimeRangeSlider.step is in milliseconds (unlike DateRangeSlider
            # whose step is in days), so step_ms can be passed directly.
            min_ms = filt["min_ms"]
            max_ms = filt["max_ms"]
            step_ms = filt["step_ms"]
            bf = BooleanFilter(booleans=[True] * n)
            dr_slider = DatetimeRangeSlider(
                start=int(min_ms), end=int(max_ms),
                value=(int(min_ms), int(max_ms)),
                step=int(step_ms),
                title=filt["label"],
                sizing_mode="stretch_width",
            )
            callback = CustomJS(
                args=dict(bf=bf, source=source, col=col_name),
                code="""
                    const [lo, hi] = cb_obj.value;
                    const data = source.data[col];
                    bf.booleans = data.map(v => v >= lo && v <= hi);
                    source.change.emit();
                """,
            )
            dr_slider.js_on_change("value", callback)
            filters_by_source[source_key].append(bf)
            widgets.append(dr_slider)
            date_range_sliders.append((source_key, col_name, dr_slider))

    # Build CDSView per source_key
    views = {}
    for source_key, filter_list in filters_by_source.items():
        if len(filter_list) == 0:
            views[source_key] = CDSView(filter=AllIndices())
        elif len(filter_list) == 1:
            views[source_key] = CDSView(filter=filter_list[0])
        else:
            views[source_key] = CDSView(
                filter=IntersectionFilter(operands=filter_list)
            )

    return views, widgets, date_range_sliders


# ── Nav tree builder ────────────────────────────────────────────────────────

def build_nav_tree(nav_links, current_slug):
    """
    Parse a flat nav_links list into a nested tree.

    Category strings use "/" as a hierarchy separator, e.g. "Financial/Revenue".
    Returns a root node dict::

        {
          "pages": [link, ...],          # pages with no category
          "children": [node, ...],       # top-level category nodes
          "has_active": bool,
        }

    Each interior node::

        {
          "label": str,
          "path": str,                   # full slash-joined path
          "pages": [link, ...],          # pages assigned to exactly this node
          "children": [node, ...],
          "has_active": bool,
        }
    """
    root = {"pages": [], "children": {}}

    for link in nav_links:
        cat = link.get("category", "").strip()
        if not cat:
            root["pages"].append(link)
        else:
            parts = [p.strip() for p in cat.split("/") if p.strip()]
            node = root
            path_parts = []
            for i, part in enumerate(parts):
                path_parts.append(part)
                path = "/".join(path_parts)
                if part not in node["children"]:
                    node["children"][part] = {
                        "label": part,
                        "path": path,
                        "pages": [],
                        "children": {},
                    }
                if i == len(parts) - 1:
                    node["children"][part]["pages"].append(link)
                else:
                    node = node["children"][part]

    def finalize(node):
        children = [finalize(c) for c in node["children"].values()]
        has_active = any(p["slug"] == current_slug for p in node["pages"]) or any(
            c["has_active"] for c in children
        )
        result = {"pages": node["pages"], "children": children, "has_active": has_active}
        if "label" in node:
            result["label"] = node["label"]
            result["path"] = node["path"]
        return result

    root_children = [finalize(c) for c in root["children"].values()]
    root_has_active = any(p["slug"] == current_slug for p in root["pages"]) or any(
        c["has_active"] for c in root_children
    )
    return {"pages": root["pages"], "children": root_children, "has_active": root_has_active}


# ── Render all pages ────────────────────────────────────────────────────────

os.makedirs(output_dir, exist_ok=True)
template = Template(html_template)
bokeh_js_urls = CDN.js_files
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""

for page in pages:
    source_cache = {}  # per-page CDS cache for linking
    bokeh_figs = []    # Bokeh figure objects in encounter order
    renderables = []   # unified list: {"type", "div"/"figure", "grid", "title", "module_type"}

    # Separate RangeTool specs (x-axis range sync) from CDSView filter specs.
    page_filters = page.get("filters", [])
    range_tool_specs = [f for f in page_filters if f["kind"] == "range_tool"]
    cds_filters = [f for f in page_filters if f["kind"] != "range_tool"]

    # Build one shared Range1d per source_key that has a RangeTool spec.
    range_tool_x_ranges = {}
    for rt in range_tool_specs:
        sk = rt["source_key"]
        range_tool_x_ranges[sk] = Range1d(start=rt["start"], end=rt["end"])

    # Pre-populate flat sources for any source_key referenced by filtered specs,
    # so that build_filter_objects can find them in the cache.
    filtered_keys = {f["source_key"] for f in cds_filters}
    # Also pre-populate sources needed for range_tool overview charts.
    for rt in range_tool_specs:
        filtered_keys.add(rt["source_key"])
    for key in filtered_keys:
        _get_flat_source(key, source_cache)

    # Build filter objects and CDSViews (range_tool is excluded — it is not
    # a CDSView filter).
    views, filter_widgets, date_range_sliders = build_filter_objects(cds_filters, source_cache)
    dt_line_x_ranges = {}  # source_key → list of Range1d for datetime line charts

    # For each RangeTool spec, add a BooleanFilter driven by the shared Range1d
    # so that charts marked .filtered() only show rows within the current date
    # window.  The callback fires whenever the Range1d start or end changes
    # (i.e. when the user drags the navigator overlay).
    for rt in range_tool_specs:
        sk = rt["source_key"]
        x_col = rt["column"]
        shared_x_range = range_tool_x_ranges[sk]

        source = source_cache.get(sk)
        if source is None:
            continue

        n = len(list(source.data.values())[0])
        bf = BooleanFilter(booleans=[True] * n)

        callback = CustomJS(
            args=dict(bf=bf, source=source, col=x_col),
            code="""
                const lo = cb_obj.start;
                const hi = cb_obj.end;
                const data = source.data[col];
                bf.booleans = data.map(v => v >= lo && v <= hi);
                source.change.emit();
            """,
        )
        shared_x_range.js_on_change("start", callback)
        shared_x_range.js_on_change("end", callback)

        # Merge the new BooleanFilter into any existing CDSView for this source
        # so that it combines with other filters (e.g. a Select dropdown).
        if sk in views:
            existing = views[sk].filter
            if isinstance(existing, AllIndices):
                views[sk] = CDSView(filter=bf)
            elif isinstance(existing, IntersectionFilter):
                existing.operands.append(bf)
            else:
                views[sk] = CDSView(filter=IntersectionFilter(operands=[existing, bf]))
        else:
            views[sk] = CDSView(filter=bf)

    # Index range_tool specs by source_key for O(1) lookup.
    range_tool_by_source = {rt["source_key"]: rt for rt in range_tool_specs}

    for mod in page["modules"]:
        grid = {
            "grid_row": mod["grid_row"] + 1,
            "grid_col": mod["grid_col"] + 1,
            "grid_col_span": mod["grid_col_span"],
        }
        mtype = mod["module_type"]

        if mtype == "chart":
            builder = _BUILDERS.get(mod["chart_type"])
            if builder is None:
                raise ValueError(f"Unknown chart_type: {mod['chart_type']!r}")
            view = views.get(mod["source_key"]) if mod.get("filtered") else None
            # Pass the shared Range1d only when the chart's x column matches
            # the range_tool's x column.  Scatter charts that use a different
            # x column (e.g. "temperature" vs the range_tool's "timestamp_ms")
            # must NOT receive the datetime Range1d or all their points will
            # fall outside the visible window.
            rt_x_range = None
            if mod["chart_type"] in ("line_multi", "scatter"):
                rt = range_tool_by_source.get(mod["source_key"])
                if rt is not None and mod.get("x_col") == rt["column"]:
                    rt_x_range = range_tool_x_ranges[mod["source_key"]]
            fig = builder(mod, source_cache, view=view, x_range=rt_x_range) \
                if rt_x_range is not None \
                else builder(mod, source_cache, view=view)
            bokeh_figs.append(fig)
            # Track datetime line chart x_ranges so date_range sliders can zoom them
            if mod["chart_type"] == "line_multi" and isinstance(fig.x_range, Range1d):
                dt_line_x_ranges.setdefault(mod["source_key"], []).append(fig.x_range)
            renderables.append({
                "type": "bokeh",
                "figure": fig,
                "grid": grid,
                "title": mod["title"],
                "module_type": "chart",
            })
        elif mtype == "paragraph":
            renderables.append({
                "type": "html",
                "div": _build_paragraph_html(mod),
                "grid": grid,
                "title": "",
                "module_type": "paragraph",
            })
        elif mtype == "table":
            renderables.append({
                "type": "html",
                "div": _build_table_html(mod, dataframes),
                "grid": grid,
                "title": "",
                "module_type": "table",
            })
        else:
            raise ValueError(f"Unknown module_type: {mtype!r}")

    # Wire date_range sliders to datetime line chart x_ranges so the line chart
    # zooms to the selected date window (CDSView filtering doesn't apply to line
    # renderers — updating the Range1d is the correct Bokeh pattern).
    for sk, _col, slider in date_range_sliders:
        for x_range in dt_line_x_ranges.get(sk, []):
            callback = CustomJS(
                args=dict(x_range=x_range),
                code="""
                    x_range.start = cb_obj.value[0];
                    x_range.end = cb_obj.value[1];
                """,
            )
            slider.js_on_change("value", callback)

    # Append auto-generated RangeTool overview charts below the grid.
    if range_tool_specs:
        max_row = max(
            (r["grid"]["grid_row"] for r in renderables if r["type"] == "bokeh"),
            default=0,
        )
        grid_cols = page["grid_cols"]
        for i, rt in enumerate(range_tool_specs):
            shared_x_range = range_tool_x_ranges[rt["source_key"]]
            overview_fig = build_range_tool_overview(rt, source_cache, shared_x_range)
            bokeh_figs.append(overview_fig)
            renderables.append({
                "type": "bokeh",
                "figure": overview_fig,
                "grid": {
                    "grid_row": max_row + 1 + i,
                    "grid_col": 1,
                    "grid_col_span": grid_cols,
                },
                "title": rt["label"],
                "module_type": "chart",
            })

    # Flatten filter widgets — Switch widgets are wrapped in a dict with label
    flat_widgets = []
    for w in filter_widgets:
        if isinstance(w, dict) and "switch" in w:
            flat_widgets.append(w["switch"])
        else:
            flat_widgets.append(w)

    # Run components() only on Bokeh objects (widgets + chart figures)
    all_bokeh = flat_widgets + bokeh_figs
    if all_bokeh:
        script, divs = components(all_bokeh)
    else:
        script, divs = "", []

    widget_divs = divs[: len(flat_widgets)]
    bokeh_chart_divs = divs[len(flat_widgets):]

    # Build unified plots list for the template
    bokeh_iter = iter(bokeh_chart_divs)
    plots = []
    for r in renderables:
        div = next(bokeh_iter) if r["type"] == "bokeh" else r["div"]
        plots.append({
            **r["grid"],
            "div": div,
            "title": r["title"],
            "module_type": r["module_type"],
        })

    # Pair widget divs with labels for Switch widgets (others use built-in titles)
    filter_items = []
    for i, w in enumerate(filter_widgets):
        if isinstance(w, dict) and "switch" in w:
            filter_items.append({"div": widget_divs[i], "label": w["label"]})
        else:
            filter_items.append({"div": widget_divs[i], "label": None})

    html = template.render(
        title=page["title"],
        report_title=report_title,
        nav_style=nav_style,
        bokeh_js_urls=bokeh_js_urls,
        bokeh_css_url=bokeh_css_url,
        plot_script=script,
        plots=plots,
        filter_items=filter_items,
        grid_cols=page["grid_cols"],
        nav_links=nav_links,
        nav_tree=build_nav_tree(nav_links, page["slug"]),
        current_slug=page["slug"],
    )

    path = os.path.join(output_dir, f"{page['slug']}.html")
    with open(path, "w", encoding="utf-8") as f:
        f.write(html)
