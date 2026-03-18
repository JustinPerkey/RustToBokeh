# render.py — called from Rust via PyO3
# Variables injected by Rust:
#   frames: dict[str, bytes]  — Arrow IPC serialized Polars DataFrames
#   html_template: str        — Jinja2 HTML template source
#   output_path: str          — destination file path

import io

import polars as pl
from bokeh.embed import components
from bokeh.models import ColumnDataSource, FactorRange
from bokeh.plotting import figure
from bokeh.resources import CDN
from bokeh.transform import factor_cmap
from jinja2 import Template

# Deserialize each Arrow IPC payload back into a Polars DataFrame
dataframes = {name: pl.read_ipc(io.BytesIO(data)) for name, data in frames.items()}

# ── Monthly grouped bar chart ────────────────────────────────────────────────

monthly_df = dataframes["monthly"]
months_list = monthly_df["month"].to_list()
revenue_list = monthly_df["revenue"].to_list()
expenses_list = monthly_df["expenses"].to_list()

categories = ["Revenue", "Expenses"]
x_monthly = [(m, cat) for m in months_list for cat in categories]
counts_monthly = []
for rev, exp in zip(revenue_list, expenses_list):
    counts_monthly.extend([rev, exp])

source_monthly = ColumnDataSource(dict(x=x_monthly, counts=counts_monthly))
palette_monthly = ["#4C72B0", "#DD8452"]

monthly_fig = figure(
    x_range=FactorRange(*x_monthly),
    height=450,
    width=900,
    title="Monthly Revenue vs Expenses (2024)",
    toolbar_location="above",
    tools="pan,wheel_zoom,box_zoom,reset,save",
)
monthly_fig.vbar(
    x="x",
    top="counts",
    width=0.9,
    source=source_monthly,
    line_color="white",
    fill_color=factor_cmap(
        "x", palette=palette_monthly, factors=categories, start=1, end=2
    ),
)
monthly_fig.x_range.range_padding = 0.1
monthly_fig.xaxis.major_label_orientation = 1.0
monthly_fig.xaxis.group_label_orientation = 0.5
monthly_fig.yaxis.axis_label = "Amount (USD thousands)"
monthly_fig.xgrid.grid_line_color = None

# ── Quarterly product breakdown grouped bar chart ────────────────────────────

quarterly_df = dataframes["quarterly"]
quarters_list = quarterly_df["quarter"].to_list()
product_a_list = quarterly_df["product_a"].to_list()
product_b_list = quarterly_df["product_b"].to_list()
product_c_list = quarterly_df["product_c"].to_list()

products = ["Product A", "Product B", "Product C"]
x_quarterly = [(q, prod) for q in quarters_list for prod in products]
counts_quarterly = []
for a, b, c in zip(product_a_list, product_b_list, product_c_list):
    counts_quarterly.extend([a, b, c])

source_quarterly = ColumnDataSource(dict(x=x_quarterly, counts=counts_quarterly))
palette_quarterly = ["#2ca02c", "#9467bd", "#e377c2"]

quarterly_fig = figure(
    x_range=FactorRange(*x_quarterly),
    height=450,
    width=900,
    title="Quarterly Product Revenue",
    toolbar_location="above",
    tools="pan,wheel_zoom,box_zoom,reset,save",
)
quarterly_fig.vbar(
    x="x",
    top="counts",
    width=0.9,
    source=source_quarterly,
    line_color="white",
    fill_color=factor_cmap(
        "x", palette=palette_quarterly, factors=products, start=1, end=2
    ),
)
quarterly_fig.x_range.range_padding = 0.1
quarterly_fig.xaxis.major_label_orientation = 1.0
quarterly_fig.xaxis.group_label_orientation = 0.5
quarterly_fig.yaxis.axis_label = "Revenue (USD thousands)"
quarterly_fig.xgrid.grid_line_color = None

# ── Combine all figures into a single Bokeh script ───────────────────────────

script, divs = components([monthly_fig, quarterly_fig])
bokeh_js_url = CDN.js_files[0]
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""

plots = [
    {"title": "Monthly Revenue vs Expenses (2024)", "div": divs[0]},
    {"title": "Quarterly Product Revenue", "div": divs[1]},
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
