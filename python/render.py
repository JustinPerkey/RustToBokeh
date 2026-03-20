# render.py — called from Rust via PyO3
# Variables injected by Rust:
#   frames: dict[str, bytes]   — Arrow IPC bytes keyed by source name
#   pages: list[dict]          — each page has slug, title, grid_cols, specs, filters
#   nav_links: list[dict]      — slug + label for every page (navigation)
#   html_template: str         — Jinja2 HTML template source
#   output_dir: str            — output directory path

import io
import os

import polars as pl
from bokeh.embed import components
from bokeh.models import (
    AllIndices,
    BooleanFilter,
    CDSView,
    ColumnDataSource,
    CustomJS,
    FactorRange,
    GroupFilter,
    IndexFilter,
    IntersectionFilter,
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


# ── Chart builders ──────────────────────────────────────────────────────────
# Each builder receives (spec_dict, source_cache, view) and returns a figure.
# If view is not None, renderers attach it for CDSView-based filtering.


def build_grouped_bar(spec, source_cache, view=None):
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


def build_line_multi(spec, source_cache, view=None):
    key = spec["source_key"]
    df = dataframes[key]
    x_col = spec["x_col"]
    y_cols = [c.strip() for c in spec["y_cols"].split(",")]

    source = _get_flat_source(key, source_cache)
    vkw = dict(view=view) if view else {}

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
                 color=color, legend_label=col, **vkw)
        fig.scatter(x=x_col, y=col, source=source, size=7,
                    color=color, legend_label=col, **vkw)
    fig.yaxis.axis_label = spec.get("y_label", "")
    fig.legend.location = "top_left"
    fig.legend.click_policy = "hide"
    return fig


def build_hbar(spec, source_cache, view=None):
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


def build_scatter(spec, source_cache, view=None):
    key = spec["source_key"]
    x_col = spec["x_col"]
    y_col = spec["y_col"]

    source = _get_flat_source(key, source_cache)
    vkw = dict(view=view) if view else {}

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
        **vkw,
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


# ── Render all pages ────────────────────────────────────────────────────────

os.makedirs(output_dir, exist_ok=True)
template = Template(html_template)
bokeh_js_urls = CDN.js_files
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""

for page in pages:
    source_cache = {}  # per-page CDS cache for linking
    figs = []
    grid_items = []

    # Pre-populate flat sources for any source_key referenced by filtered specs,
    # so that build_filter_objects can find them in the cache.
    page_filters = page.get("filters", [])
    filtered_keys = {f["source_key"] for f in page_filters}
    for key in filtered_keys:
        _get_flat_source(key, source_cache)

    # Build filter objects and CDSViews
    views, filter_widgets = build_filter_objects(page_filters, source_cache)

    for spec in page["specs"]:
        builder = _BUILDERS.get(spec["chart_type"])
        if builder is None:
            raise ValueError(f"Unknown chart_type: {spec['chart_type']!r}")
        view = views.get(spec["source_key"]) if spec.get("filtered") else None
        fig = builder(spec, source_cache, view=view)
        figs.append(fig)
        grid_items.append({
            "title": spec["title"],
            "grid_row": spec["grid_row"] + 1,
            "grid_col": spec["grid_col"] + 1,
            "grid_col_span": spec["grid_col_span"],
        })

    # Flatten filter widgets — Switch widgets are wrapped in a dict with label
    flat_widgets = []
    for w in filter_widgets:
        if isinstance(w, dict) and "switch" in w:
            flat_widgets.append(w["switch"])
        else:
            flat_widgets.append(w)

    # Combine widgets + figures for a single components() call
    all_objects = flat_widgets + figs
    script, divs = components(all_objects)

    widget_divs = divs[: len(flat_widgets)]
    chart_divs = divs[len(flat_widgets):]
    plots = [{**item, "div": div} for item, div in zip(grid_items, chart_divs)]

    # Pair widget divs with labels for Switch widgets (others use built-in titles)
    filter_items = []
    for i, w in enumerate(filter_widgets):
        if isinstance(w, dict) and "switch" in w:
            filter_items.append({"div": widget_divs[i], "label": w["label"]})
        else:
            filter_items.append({"div": widget_divs[i], "label": None})

    html = template.render(
        title=page["title"],
        bokeh_js_urls=bokeh_js_urls,
        bokeh_css_url=bokeh_css_url,
        plot_script=script,
        plots=plots,
        filter_items=filter_items,
        grid_cols=page["grid_cols"],
        nav_links=nav_links,
        current_slug=page["slug"],
    )

    path = os.path.join(output_dir, f"{page['slug']}.html")
    with open(path, "w") as f:
        f.write(html)
