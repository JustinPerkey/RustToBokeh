use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Line = LineConfig;
type Scat = ScatterConfig;
type Bar = GroupedBarConfig;
type HB = HBarConfig;
type Hist = HistogramConfig;
type Para = ParagraphSpec;
type Pie = PieConfig;
type Tbl = TableSpec;
type TC = TableColumn;

// Jan 1 2024 00:00:00 UTC in milliseconds
const JAN_1_2024_MS: f64 = 1_704_067_200_000.0;
// Jan 30 2024 00:00:00 UTC in milliseconds
const JAN_30_2024_MS: f64 = 1_706_572_800_000.0;

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

/// Demonstrates the `RangeTool` navigator with a datetime X axis.
///
/// This page shows:
/// - `FilterConfig::RangeTool` — auto-generated overview chart with a
///   draggable range-selector overlay that zooms the detail charts
/// - `FilterConfig::Select` combined with RangeTool (CDSView filtering while
///   the range tool controls the x-axis window)
/// - `LineConfig` with a datetime X axis (`TimeScale::Days`)
/// - `ScatterConfig` sharing the same `ColumnDataSource` (linked selection)
/// - Hierarchical nav category `"Reference/Time Series"`
pub fn page_range_tool_demo() -> Result<Page, ChartError> {
    PageBuilder::new("range-tool-demo", "RangeTool Navigator", "RangeTool", 2)
        .category("Reference/Time Series")
        .paragraph(
            Para::new(
                "This page demonstrates the RangeTool navigator. The compact \
                 overview chart at the bottom lets you drag or resize the shaded \
                 selection window to zoom and pan the detail charts above.\n\n\
                 The Sensor dropdown applies a CDSView filter to the scatter \
                 chart independently of the range selection, showing how the two \
                 mechanisms can be combined on the same page.",
            )
            .title("About This Page")
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Sensor Readings Over Time",
                "sensor_events",
                Line::builder()
                    .x("timestamp_ms")
                    .y_cols(&["temperature", "humidity"])
                    .y_label("Reading")
                    .x_axis(
                        AxisConfig::builder()
                            .time_scale(TimeScale::Days)
                            .build(),
                    )
                    .tooltips(
                        TooltipSpec::builder()
                            .field("timestamp_ms", "Date", TooltipFormat::DateTime(TimeScale::Days))
                            .field("sensor", "Sensor", TooltipFormat::Text)
                            .field("temperature", "Temp (°C)", TooltipFormat::Number(Some(1)))
                            .field("humidity", "Humidity (%)", TooltipFormat::Number(Some(1)))
                            .build(),
                    )
                    .build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Temperature vs Humidity",
                "sensor_events",
                Scat::builder()
                    .x("temperature")
                    .y("humidity")
                    .x_label("Temperature (°C)")
                    .y_label("Humidity (%)")
                    .tooltips(
                        TooltipSpec::builder()
                            .field("sensor", "Sensor", TooltipFormat::Text)
                            .field("timestamp_ms", "Date", TooltipFormat::DateTime(TimeScale::Days))
                            .field("temperature", "Temp (°C)", TooltipFormat::Number(Some(1)))
                            .field("humidity", "Humidity (%)", TooltipFormat::Number(Some(1)))
                            .build(),
                    )
                    .build()?,
            )
            .at(2, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Temperature vs Pressure",
                "sensor_events",
                Scat::builder()
                    .x("temperature")
                    .y("pressure")
                    .x_label("Temperature (°C)")
                    .y_label("Pressure (hPa)")
                    .tooltips(
                        TooltipSpec::builder()
                            .field("sensor", "Sensor", TooltipFormat::Text)
                            .field("temperature", "Temp (°C)", TooltipFormat::Number(Some(1)))
                            .field("pressure", "Pressure", TooltipFormat::Number(Some(1)))
                            .build(),
                    )
                    .build()?,
            )
            .at(2, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::range_tool(
            "sensor_events",
            "timestamp_ms",
            "temperature",
            "Navigator — drag or resize to zoom",
            JAN_1_2024_MS,
            JAN_30_2024_MS,
            Some(TimeScale::Days),
        ))
        .filter(FilterSpec::select(
            "sensor_events",
            "sensor",
            "Sensor",
            vec!["Alpha", "Beta", "Gamma"],
        ))
        .build()
}

/// Demonstrates the `DateRange` filter with a datetime X axis and hierarchical nav category.
///
/// This page shows:
/// - `FilterConfig::DateRange` via a `DateRangeSlider` widget
/// - `FilterConfig::Select` combined with DateRange (two filters on one source)
/// - `LineConfig` with a datetime X axis (`TimeScale::Days`)
/// - `ScatterConfig` sharing the same `ColumnDataSource` (linked selection)
/// - Hierarchical nav category `"Reference/Time Series"`
pub fn page_time_series_events() -> Result<Page, ChartError> {
    PageBuilder::new("time-series-events", "Sensor Time Series", "Time Series", 2)
        .category("Reference/Time Series")
        .paragraph(
            Para::new(
                "This page demonstrates the DateRange filter and datetime X axis. \
                 Use the date-range slider to zoom in on a specific window, and the \
                 sensor selector to highlight readings from a single sensor.\n\n\
                 The line chart and scatter plot share one ColumnDataSource, so \
                 selections and filters apply to both simultaneously.",
            )
            .title("About This Page")
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Sensor Readings Over Time",
                "sensor_events",
                Line::builder()
                    .x("timestamp_ms")
                    .y_cols(&["temperature", "humidity"])
                    .y_label("Reading")
                    .x_axis(
                        AxisConfig::builder()
                            .time_scale(TimeScale::Days)
                            .build(),
                    )
                    .tooltips(
                        TooltipSpec::builder()
                            .field("timestamp_ms", "Date", TooltipFormat::DateTime(TimeScale::Days))
                            .field("sensor", "Sensor", TooltipFormat::Text)
                            .field("temperature", "Temp (°C)", TooltipFormat::Number(Some(1)))
                            .field("humidity", "Humidity (%)", TooltipFormat::Number(Some(1)))
                            .build(),
                    )
                    .build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Temperature vs Humidity",
                "sensor_events",
                Scat::builder()
                    .x("temperature")
                    .y("humidity")
                    .x_label("Temperature (°C)")
                    .y_label("Humidity (%)")
                    .tooltips(
                        TooltipSpec::builder()
                            .field("sensor", "Sensor", TooltipFormat::Text)
                            .field("timestamp_ms", "Date", TooltipFormat::DateTime(TimeScale::Days))
                            .field("temperature", "Temp (°C)", TooltipFormat::Number(Some(1)))
                            .field("humidity", "Humidity (%)", TooltipFormat::Number(Some(1)))
                            .build(),
                    )
                    .build()?,
            )
            .at(2, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Temperature vs Pressure",
                "sensor_events",
                Scat::builder()
                    .x("temperature")
                    .y("pressure")
                    .x_label("Temperature (°C)")
                    .y_label("Pressure (hPa)")
                    .tooltips(
                        TooltipSpec::builder()
                            .field("sensor", "Sensor", TooltipFormat::Text)
                            .field("temperature", "Temp (°C)", TooltipFormat::Number(Some(1)))
                            .field("pressure", "Pressure", TooltipFormat::Number(Some(1)))
                            .build(),
                    )
                    .build()?,
            )
            .at(2, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::date_range(
            "sensor_events",
            "timestamp_ms",
            "Date Range",
            JAN_1_2024_MS,
            JAN_30_2024_MS,
            DateStep::Day,
            TimeScale::Days,
        ))
        .filter(FilterSpec::select(
            "sensor_events",
            "sensor",
            "Sensor",
            vec!["Alpha", "Beta", "Gamma"],
        ))
        .build()
}

pub fn page_pie_donut_charts() -> Result<Page, ChartError> {
    PageBuilder::new("pie-donut-charts", "Pie & Donut Charts", "Pie & Donut", 2)
        .category("Reference")
        .chart(
            C::pie(
                "Market Share",
                "market_share",
                Pie::builder().label("company").value("share").build()?,
            )
            .at(0, 0, 1)
            .dimensions(380, 380)
            .build(),
        )
        .chart(
            C::pie(
                "Cost Breakdown",
                "cost_breakdown",
                Pie::builder()
                    .label("category")
                    .value("amount")
                    .inner_radius(0.45)
                    .build()?,
            )
            .at(0, 1, 1)
            .dimensions(380, 380)
            .build(),
        )
        .build()
}

pub fn page_histogram_demo() -> Result<Page, ChartError> {
    PageBuilder::new("histogram-demo", "Histogram Demo", "Histogram", 2)
        .category("Reference")
        .chart(
            C::histogram(
                "Salary Distribution — Count",
                "salary_hist",
                Hist::builder()
                    .x_label("Salary (k)")
                    .build()?,
            )
            .at(0, 0, 1)
            .build(),
        )
        .chart(
            C::histogram(
                "Salary Distribution — Density (PDF)",
                "salary_hist",
                Hist::builder()
                    .x_label("Salary (k)")
                    .display(HistogramDisplay::Pdf)
                    .color("#2ecc71")
                    .build()?,
            )
            .at(0, 1, 1)
            .build(),
        )
        .chart(
            C::histogram(
                "Salary Distribution — Cumulative (CDF)",
                "salary_hist",
                Hist::builder()
                    .x_label("Salary (k)")
                    .display(HistogramDisplay::Cdf)
                    .color("#e74c3c")
                    .build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .build()
}
