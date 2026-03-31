use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Line = LineConfig;
type Scat = ScatterConfig;
type Bar = GroupedBarConfig;
type HB = HBarConfig;
type Para = ParagraphSpec;
type Tbl = TableSpec;
type TC = TableColumn;

pub fn page_module_showcase() -> Result<Page, ChartError> {
    PageBuilder::new("module-showcase", "Module Showcase", "Showcase", 2)
        .category("Reference")
        .paragraph(
            Para::new(
                "This page demonstrates the three content module types available \
                 in RustToBokeh: charts, paragraphs, and data tables.\n\n\
                 Paragraph modules render styled text blocks and support multiple \
                 paragraphs separated by blank lines. They are useful for adding \
                 context, annotations, or commentary alongside data visualisations.\n\n\
                 Table modules pull directly from any registered DataFrame and \
                 support per-column formatting: plain text, fixed-point numbers, \
                 currency with thousands separators, and percentage values.",
            )
            .title("About This Page")
            .at(0, 0, 2)
            .build(),
        )
        .table(
            Tbl::new("Monthly Revenue & Expenses", "monthly_revenue")
                .column(TC::text("month", "Month"))
                .column(TC::text("category", "Category"))
                .column(TC::currency("value", "Amount (k)", "$", 1))
                .at(1, 0, 1)
                .build(),
        )
        .chart(
            C::line(
                "Revenue Trend",
                "monthly_trends",
                Line::builder()
                    .x("month")
                    .y_cols(&["revenue", "expenses", "profit"])
                    .y_label("USD (k)")
                    .build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .table(
            Tbl::new("Project Status", "project_status")
                .column(TC::text("project", "Project"))
                .column(TC::percent("completion", "Completion %", 0))
                .at(2, 0, 1)
                .build(),
        )
        .table(
            Tbl::new("Performance Snapshot", "scatter_performance")
                .column(TC::text("tier", "Tier"))
                .column(TC::currency("revenue", "Revenue (k)", "$", 0))
                .column(TC::number("profit", "Profit (k)", 1))
                .column(TC::number("employees", "Headcount", 0))
                .column(TC::number("satisfaction", "Satisfaction", 2))
                .at(2, 1, 1)
                .build(),
        )
        .build()
}

pub fn page_chart_customization() -> Result<Page, ChartError> {
    PageBuilder::new(
        "chart-customization",
        "Chart Customization",
        "Customization",
        2,
    )
    .category("Reference")
    // Scatter — custom marker, color, alpha, tooltip, axis ranges + bounds, fixed dimensions
    .chart(
        C::scatter(
            "Revenue vs Profit (styled)",
            "scatter_performance",
            Scat::builder()
                .x("revenue")
                .y("profit")
                .x_label("Revenue (k)")
                .y_label("Profit (k)")
                .color("#e74c3c")
                .marker("diamond")
                .marker_size(12.0)
                .alpha(0.85)
                .tooltips(
                    TooltipSpec::builder()
                        .field("tier", "Tier", TooltipFormat::Text)
                        .field("revenue", "Revenue", TooltipFormat::Currency)
                        .field("profit", "Profit", TooltipFormat::Number(Some(1)))
                        .field("satisfaction", "Satisfaction", TooltipFormat::Number(Some(2)))
                        .build(),
                )
                .x_axis(
                    AxisConfig::builder()
                        .range(0.0, 350.0)
                        .bounds(0.0, 400.0)
                        .tick_format("$0,0")
                        .build(),
                )
                .y_axis(
                    AxisConfig::builder()
                        .range(0.0, 100.0)
                        .bounds(0.0, 120.0)
                        .show_grid(false)
                        .build(),
                )
                .build()?,
        )
        .at(0, 0, 1)
        .dimensions(550, 380)
        .build(),
    )
    // HBar — custom color, tick format, axis range + bounds on value axis
    .chart(
        C::hbar(
            "Market Share % (styled)",
            "market_share",
            HB::builder()
                .category("company")
                .value("share")
                .x_label("Share (%)")
                .color("#9b59b6")
                .tooltips(
                    TooltipSpec::builder()
                        .field("company", "Company", TooltipFormat::Text)
                        .field("share", "Share", TooltipFormat::Number(Some(1)))
                        .build(),
                )
                .x_axis(
                    AxisConfig::builder()
                        .range(0.0, 35.0)
                        .bounds(0.0, 40.0)
                        .tick_format("0.0")
                        .build(),
                )
                .build()?,
        )
        .at(0, 1, 1)
        .build(),
    )
    // Line — custom palette, line_width, point_size, y-axis tick format
    .chart(
        C::line(
            "Revenue & Profit Trends (styled)",
            "monthly_trends",
            Line::builder()
                .x("month")
                .y_cols(&["revenue", "profit", "expenses"])
                .y_label("USD (k)")
                .palette(PaletteSpec::Custom(vec![
                    "#2ecc71".into(),
                    "#e74c3c".into(),
                    "#3498db".into(),
                ]))
                .line_width(3.5)
                .point_size(9.0)
                .y_axis(
                    AxisConfig::builder()
                        .tick_format("$0,0")
                        .build(),
                )
                .build()?,
        )
        .at(1, 0, 1)
        .build(),
    )
    // Grouped bar — named Bokeh palette, narrower bars, label rotation on x
    .chart(
        C::bar(
            "Quarterly Products (styled)",
            "quarterly_products",
            Bar::builder()
                .x("quarter")
                .group("product")
                .value("value")
                .y_label("Revenue (k)")
                .palette(PaletteSpec::Named("Category20".into()))
                .bar_width(0.65)
                .y_axis(AxisConfig::builder().tick_format("$0,0").build())
                .build()?,
        )
        .at(1, 1, 1)
        .build(),
    )
    .build()
}
