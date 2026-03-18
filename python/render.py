# render.py — called from Rust via PyO3
# Variables injected by Rust: months, revenue, expenses, html_template, output_path

from bokeh.plotting import figure
from bokeh.models import ColumnDataSource, FactorRange
from bokeh.transform import factor_cmap
from bokeh.embed import components
from bokeh.resources import CDN
from jinja2 import Template

# Build grouped bar chart data: (month, category) pairs
categories = ["Revenue", "Expenses"]
x = [(m, cat) for m in months for cat in categories]
counts = []
for rev, exp in zip(revenue, expenses):
    counts.extend([rev, exp])

source = ColumnDataSource(dict(x=x, counts=counts))
palette = ["#4C72B0", "#DD8452"]

p = figure(
    x_range=FactorRange(*x),
    height=450,
    width=900,
    title="Monthly Revenue vs Expenses (2024)",
    toolbar_location="above",
    tools="pan,wheel_zoom,box_zoom,reset,save",
)

p.vbar(
    x="x",
    top="counts",
    width=0.9,
    source=source,
    line_color="white",
    fill_color=factor_cmap(
        "x", palette=palette, factors=categories, start=1, end=2
    ),
    legend_field="x",
)

p.x_range.range_padding = 0.1
p.xaxis.major_label_orientation = 1.0
p.xaxis.group_label_orientation = 0.5
p.yaxis.axis_label = "Amount (USD thousands)"
p.xgrid.grid_line_color = None
p.legend.location = "top_left"
p.legend.orientation = "horizontal"
p.legend.items = [
    (cat, [p.renderers[0]]) for cat in categories
]

# Extract Bokeh JS/CSS/script components
script, div = components(p)
bokeh_js_url = CDN.js_files[0]
bokeh_css_url = CDN.css_files[0] if CDN.css_files else ""

# Render Jinja2 template with Bokeh components
template = Template(html_template)
html = template.render(
    title="Monthly Revenue vs Expenses",
    bokeh_js_url=bokeh_js_url,
    bokeh_css_url=bokeh_css_url,
    plot_script=script,
    plot_div=div,
)

with open(output_path, "w") as f:
    f.write(html)
