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
    FactorRange,
    GroupFilter,
    HoverTool,
    IndexFilter,
    IntersectionFilter,
    NumeralTickFormatter,
    Range1d,
    RangeSlider,
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
        else:
            fmt_str = f"@{{{col}}}"
        tooltips.append((label, fmt_str))
    return HoverTool(tooltips=tooltips)


def _apply_axis_config(axis_dict, bokeh_axis, range_obj, grid_obj):
    """Apply an axis config dict to a Bokeh axis, range, and grid object."""
    if axis_dict is None:
        return
    # Tick format
    if axis_dict.get("tick_format") is not None:
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


def _figure_kw(spec, default_height=400):
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
    return kw


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


def build_line_multi(spec, source_cache, view=None):
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


def build_scatter(spec, source_cache, view=None):
    key = spec["source_key"]
    x_col = spec["x_col"]
    y_col = spec["y_col"]

    source = _get_flat_source(key, source_cache)
    vkw = dict(view=view) if view else {}

    hover = _build_hover_tool(spec)
    tools = "pan,wheel_zoom,box_zoom,reset,save,box_select,tap"
    if hover is None:
        tools = "pan,wheel_zoom,box_zoom,reset,save,hover,box_select,tap"

    kw = _figure_kw(spec)
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


_BUILDERS = {
    "grouped_bar": build_grouped_bar,
    "line_multi": build_line_multi,
    "hbar": build_hbar,
    "scatter": build_scatter,
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

    Returns (views, widgets) where:
      views:   dict[source_key → CDSView]
      widgets: list[Bokeh model]  (for embedding via components())
    """
    # Collect individual filter objects per source_key
    filters_by_source = {}  # source_key → list[Filter]
    widgets = []

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

    return views, widgets


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

    # Pre-populate flat sources for any source_key referenced by filtered specs,
    # so that build_filter_objects can find them in the cache.
    page_filters = page.get("filters", [])
    filtered_keys = {f["source_key"] for f in page_filters}
    for key in filtered_keys:
        _get_flat_source(key, source_cache)

    # Build filter objects and CDSViews
    views, filter_widgets = build_filter_objects(page_filters, source_cache)

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
            fig = builder(mod, source_cache, view=view)
            bokeh_figs.append(fig)
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
