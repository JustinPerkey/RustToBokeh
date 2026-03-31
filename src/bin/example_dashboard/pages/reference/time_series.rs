use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Line = LineConfig;
type Scat = ScatterConfig;
type Para = ParagraphSpec;

// Jan 1 2024 00:00:00 UTC in milliseconds
const JAN_1_2024_MS: f64 = 1_704_067_200_000.0;
// Jan 30 2024 00:00:00 UTC in milliseconds
const JAN_30_2024_MS: f64 = 1_706_572_800_000.0;

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
