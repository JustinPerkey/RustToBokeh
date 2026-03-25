//! Chart configuration types, layout primitives, and filter definitions.
//!
//! This module contains everything needed to declaratively describe charts
//! and filters in Rust. The types defined here are consumed by the Python
//! renderer ([`crate::render::render_dashboard`]) to produce interactive Bokeh
//! visualizations.
//!
//! # Chart types
//!
//! Each supported chart type has a dedicated config struct and builder:
//!
//! | Chart type | Config struct | Builder | Description |
//! |---|---|---|---|
//! | Grouped bar | [`GroupedBarConfig`] | [`GroupedBarConfigBuilder`] | Vertical bars grouped by category |
//! | Multi-line | [`LineConfig`] | [`LineConfigBuilder`] | One or more line series on a shared axis |
//! | Horizontal bar | [`HBarConfig`] | [`HBarConfigBuilder`] | Horizontal bars for ranked/categorical data |
//! | Scatter plot | [`ScatterConfig`] | [`ScatterConfigBuilder`] | X-Y scatter with circle markers |
//!
//! All config builders follow the same pattern: call the type's `builder()`
//! method, chain setter methods for each required field, then call `build()` to
//! get a `Result<Config, ChartError>`.
//!
//! # Layout
//!
//! Charts are positioned on a page grid using [`ChartSpecBuilder::at`], which
//! sets the row, column, and column span. The grid dimensions are defined by
//! [`PageBuilder::new`].
//!
//! # Filters
//!
//! Interactive filters are defined with [`FilterSpec`] factory methods and
//! attached to pages via [`crate::pages::PageBuilder::filter`]. Charts must be
//! marked with [`ChartSpecBuilder::filtered`] to opt into filtering.

use crate::error::ChartError;

// ── Grid constants ────────────────────────────────────────────────────────────

/// Maximum number of columns allowed in a page grid.
///
/// Grids wider than this are rejected by [`PageBuilder::build`](crate::pages::PageBuilder::build)
/// with a [`ChartError::GridValidation`](crate::error::ChartError::GridValidation) error.
pub const MAX_GRID_COLS: usize = 6;

// ── Visual customisation types ────────────────────────────────────────────────

/// A color palette used to assign colors to groups or series.
///
/// Used by grouped-bar and line charts to override the default seaborn-style
/// color cycle.
///
/// # Examples
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// // Bokeh built-in named palette
/// let p = PaletteSpec::Named("Plasma256".into());
///
/// // Custom hex colors — cycled when fewer colors than groups
/// let p = PaletteSpec::Custom(vec!["#e74c3c".into(), "#3498db".into()]);
/// ```
pub enum PaletteSpec {
    /// One of Bokeh's built-in named palettes (e.g. `"Category10"`,
    /// `"Category20"`, `"Viridis256"`, `"Plasma256"`).
    Named(String),
    /// A list of hex color strings (e.g. `"#4C72B0"`).  Cycled when fewer
    /// entries are supplied than there are groups or series.
    Custom(Vec<String>),
}

/// Format applied to a single tooltip field.
///
/// Used inside [`TooltipSpec`] to control how each hover value is displayed.
pub enum TooltipFormat {
    /// Plain text — renders the value as-is.
    Text,
    /// Fixed-point number.  Decimal places default to `2` when `None`.
    Number(Option<u8>),
    /// Percentage.  The raw value is shown with a `%` suffix.
    /// Decimal places default to `1` when `None`.
    Percent(Option<u8>),
    /// Currency — prefixed with `$` and formatted with thousand separators.
    Currency,
}

/// A single row in a chart tooltip.
pub struct TooltipField {
    /// Column name in the data source.
    pub column: String,
    /// Human-readable label shown before the value.
    pub label: String,
    /// How to format the column value.
    pub format: TooltipFormat,
}

/// Custom tooltip definition for a chart.
///
/// When provided, replaces the default Bokeh `HoverTool` tooltip with the
/// specified fields in the order they were added.  If omitted the renderer
/// falls back to a sensible default based on the chart's column names.
///
/// Build with [`TooltipSpec::builder`].
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let tt = TooltipSpec::builder()
///     .field("region",  "Region",  TooltipFormat::Text)
///     .field("revenue", "Revenue", TooltipFormat::Currency)
///     .field("growth",  "Growth",  TooltipFormat::Percent(Some(1)))
///     .build();
/// ```
pub struct TooltipSpec {
    /// Ordered list of fields to show in the tooltip.
    pub fields: Vec<TooltipField>,
}

/// Builder for [`TooltipSpec`].
///
/// Call [`field`](TooltipSpecBuilder::field) once per tooltip row, then
/// [`build`](TooltipSpecBuilder::build).
pub struct TooltipSpecBuilder {
    fields: Vec<TooltipField>,
}

impl TooltipSpec {
    /// Create a new builder for a tooltip specification.
    #[must_use]
    pub fn builder() -> TooltipSpecBuilder {
        TooltipSpecBuilder { fields: Vec::new() }
    }
}

impl TooltipSpecBuilder {
    /// Add a field row to the tooltip.
    ///
    /// Fields appear in the order they are added.
    #[must_use]
    pub fn field(mut self, column: &str, label: &str, format: TooltipFormat) -> Self {
        self.fields.push(TooltipField {
            column: column.into(),
            label: label.into(),
            format,
        });
        self
    }

    /// Consume the builder and produce a [`TooltipSpec`].
    #[must_use]
    pub fn build(self) -> TooltipSpec {
        TooltipSpec { fields: self.fields }
    }
}

/// Per-axis display customisation for a chart.
///
/// Controls the initial visible range, pan/zoom bounds, tick-label formatting,
/// label orientation, and grid-line visibility.  All fields are optional;
/// omitting a field preserves the Bokeh default for that property.
///
/// Build with [`AxisConfig::builder`].
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// // Dollar-formatted X axis, view 0–500, pan locked to 0–600
/// let x = AxisConfig::builder()
///     .range(0.0, 500.0)
///     .bounds(0.0, 600.0)
///     .tick_format("$0,0")
///     .build();
///
/// // Y axis with 45° label rotation and no grid lines
/// let y = AxisConfig::builder()
///     .label_rotation(45.0)
///     .show_grid(false)
///     .build();
/// ```
pub struct AxisConfig {
    /// Start of the initial visible range (`Range1d.start` in Bokeh).
    pub start: Option<f64>,
    /// End of the initial visible range (`Range1d.end` in Bokeh).
    pub end: Option<f64>,
    /// Lower pan/zoom bound (`x_range.bounds[0]` in Bokeh).
    /// Requires [`bounds_max`](AxisConfig::bounds_max) to also be set.
    pub bounds_min: Option<f64>,
    /// Upper pan/zoom bound (`x_range.bounds[1]` in Bokeh).
    /// Requires [`bounds_min`](AxisConfig::bounds_min) to also be set.
    pub bounds_max: Option<f64>,
    /// Rotation of major-tick labels in degrees (e.g. `45.0`).
    pub label_rotation: Option<f64>,
    /// [Numeral.js](http://numeraljs.com/) format string for tick labels
    /// (e.g. `"$0,0"`, `"0.0%"`, `"0.00"`).
    pub tick_format: Option<String>,
    /// Whether to draw grid lines for this axis.  Defaults to `true`.
    pub show_grid: bool,
}

/// Builder for [`AxisConfig`].
pub struct AxisConfigBuilder {
    start: Option<f64>,
    end: Option<f64>,
    bounds_min: Option<f64>,
    bounds_max: Option<f64>,
    label_rotation: Option<f64>,
    tick_format: Option<String>,
    show_grid: bool,
}

impl AxisConfig {
    /// Create a new builder for axis configuration.
    #[must_use]
    pub fn builder() -> AxisConfigBuilder {
        AxisConfigBuilder {
            start: None,
            end: None,
            bounds_min: None,
            bounds_max: None,
            label_rotation: None,
            tick_format: None,
            show_grid: true,
        }
    }
}

impl AxisConfigBuilder {
    /// Set the initial visible range of the axis.
    ///
    /// Maps to `Range1d(start=start, end=end)` in Bokeh.
    #[must_use]
    pub fn range(mut self, start: f64, end: f64) -> Self {
        self.start = Some(start);
        self.end = Some(end);
        self
    }

    /// Set the pan/zoom bounding limits for the axis.
    ///
    /// Maps to `range.bounds = (min, max)` in Bokeh.  Both values must be
    /// supplied; the user cannot pan or zoom beyond these limits at runtime.
    #[must_use]
    pub fn bounds(mut self, min: f64, max: f64) -> Self {
        self.bounds_min = Some(min);
        self.bounds_max = Some(max);
        self
    }

    /// Set the rotation of major-tick labels in degrees.
    ///
    /// Positive values rotate the labels counter-clockwise.  `45.0` is a
    /// common choice for long category labels.
    #[must_use]
    pub fn label_rotation(mut self, degrees: f64) -> Self {
        self.label_rotation = Some(degrees);
        self
    }

    /// Set the [numeral.js](http://numeraljs.com/) format string for tick labels.
    ///
    /// Examples: `"$0,0"` (currency with commas), `"0.0%"` (percentage),
    /// `"0.00"` (fixed two decimals), `"0.0a"` (abbreviated, e.g. `"1.2k"`).
    #[must_use]
    pub fn tick_format(mut self, fmt: &str) -> Self {
        self.tick_format = Some(fmt.into());
        self
    }

    /// Control whether grid lines are drawn for this axis (default: `true`).
    #[must_use]
    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Consume the builder and produce an [`AxisConfig`].
    #[must_use]
    pub fn build(self) -> AxisConfig {
        AxisConfig {
            start: self.start,
            end: self.end,
            bounds_min: self.bounds_min,
            bounds_max: self.bounds_max,
            label_rotation: self.label_rotation,
            tick_format: self.tick_format,
            show_grid: self.show_grid,
        }
    }
}

// ── Chart configuration structs ──────────────────────────────────────────────

/// Configuration for a grouped bar chart.
///
/// Grouped bar charts display vertical bars organized by a categorical X axis,
/// with bars within each group distinguished by a grouping column. The
/// `DataFrame` must contain three columns: one for the X-axis categories, one
/// for the group within each category, and one for the numeric value.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = GroupedBarConfig::builder()
///     .x("month")           // X-axis category column
///     .group("product")     // Grouping column (one bar per group value)
///     .value("revenue")     // Numeric value column (bar height)
///     .y_label("Revenue (k)")
///     .build()?;
/// ```
pub struct GroupedBarConfig {
    /// Column name for the X-axis categories (e.g. `"month"`, `"quarter"`).
    pub x_col: String,
    /// Column name for the grouping variable within each X category.
    pub group_col: String,
    /// Column name for the numeric values (bar heights).
    pub value_col: String,
    /// Label displayed on the Y axis.
    pub y_label: String,
    /// Color palette for the group bars.  Defaults to the built-in seaborn
    /// color cycle when `None`.
    pub palette: Option<PaletteSpec>,
    /// Width of each bar as a fraction of the available slot (0.0–1.0).
    /// Defaults to `0.9` when `None`.
    pub bar_width: Option<f64>,
    /// Custom hover tooltip.  Defaults to the chart column names when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`GroupedBarConfig`].
///
/// All fields are required. Calling [`build`](GroupedBarConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct GroupedBarConfigBuilder {
    x_col: Option<String>,
    group_col: Option<String>,
    value_col: Option<String>,
    y_label: Option<String>,
    palette: Option<PaletteSpec>,
    bar_width: Option<f64>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl GroupedBarConfig {
    /// Create a new builder for a grouped bar chart configuration.
    #[must_use]
    pub fn builder() -> GroupedBarConfigBuilder {
        GroupedBarConfigBuilder {
            x_col: None,
            group_col: None,
            value_col: None,
            y_label: None,
            palette: None,
            bar_width: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl GroupedBarConfigBuilder {
    /// Set the X-axis category column name.
    #[must_use] 
    pub fn x(mut self, col: &str) -> Self {
        self.x_col = Some(col.into());
        self
    }
    /// Set the grouping column name.
    #[must_use] 
    pub fn group(mut self, col: &str) -> Self {
        self.group_col = Some(col.into());
        self
    }
    /// Set the numeric value column name.
    #[must_use] 
    pub fn value(mut self, col: &str) -> Self {
        self.value_col = Some(col.into());
        self
    }
    /// Set the Y-axis label text.
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }
    /// Set the color palette for the group bars.
    #[must_use]
    pub fn palette(mut self, palette: PaletteSpec) -> Self {
        self.palette = Some(palette);
        self
    }
    /// Set the bar width as a fraction of the available slot width (0.0–1.0).
    #[must_use]
    pub fn bar_width(mut self, width: f64) -> Self {
        self.bar_width = Some(width);
        self
    }
    /// Set a custom hover tooltip.
    #[must_use]
    pub fn tooltips(mut self, tooltips: TooltipSpec) -> Self {
        self.tooltips = Some(tooltips);
        self
    }
    /// Configure the X axis appearance.
    #[must_use]
    pub fn x_axis(mut self, axis: AxisConfig) -> Self {
        self.x_axis = Some(axis);
        self
    }
    /// Configure the Y axis appearance.
    #[must_use]
    pub fn y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }

    /// Build the config, returning an error if any required field is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if any required field was not set.
    pub fn build(self) -> Result<GroupedBarConfig, ChartError> {
        Ok(GroupedBarConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            group_col: self
                .group_col
                .ok_or(ChartError::MissingField("group_col"))?,
            value_col: self
                .value_col
                .ok_or(ChartError::MissingField("value_col"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
            palette: self.palette,
            bar_width: self.bar_width,
            tooltips: self.tooltips,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for a multi-line chart.
///
/// Line charts plot one or more numeric series against a shared X axis. Each
/// entry in `y_cols` becomes a separate line rendered with a distinct color.
/// When multiple line or scatter charts on the same page share the same
/// `source_key`, they share a single Bokeh `ColumnDataSource`, enabling linked
/// hover and selection across charts.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = LineConfig::builder()
///     .x("month")
///     .y_cols(&["revenue", "expenses", "profit"])
///     .y_label("USD (k)")
///     .build()?;
/// ```
pub struct LineConfig {
    /// Column name for the X axis (typically a time or category column).
    pub x_col: String,
    /// Column names for the Y-axis series. Each column produces one line.
    pub y_cols: Vec<String>,
    /// Label displayed on the Y axis.
    pub y_label: String,
    /// Color palette for the lines.  Defaults to the built-in seaborn color
    /// cycle when `None`.
    pub palette: Option<PaletteSpec>,
    /// Stroke width of the lines in screen units.  Defaults to `2.5` when `None`.
    pub line_width: Option<f64>,
    /// Size of the scatter markers drawn at each data point.
    /// Defaults to `7` when `None`.
    pub point_size: Option<f64>,
    /// Custom hover tooltip.  Defaults to the chart column names when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`LineConfig`].
///
/// All fields are required. Calling [`build`](LineConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct LineConfigBuilder {
    x_col: Option<String>,
    y_cols: Option<Vec<String>>,
    y_label: Option<String>,
    palette: Option<PaletteSpec>,
    line_width: Option<f64>,
    point_size: Option<f64>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl LineConfig {
    /// Create a new builder for a line chart configuration.
    #[must_use] 
    pub fn builder() -> LineConfigBuilder {
        LineConfigBuilder {
            x_col: None,
            y_cols: None,
            y_label: None,
            palette: None,
            line_width: None,
            point_size: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl LineConfigBuilder {
    /// Set the X-axis column name.
    #[must_use] 
    pub fn x(mut self, col: &str) -> Self {
        self.x_col = Some(col.into());
        self
    }
    /// Set the Y-axis column names. Each column becomes a separate line.
    #[must_use] 
    pub fn y_cols(mut self, cols: &[&str]) -> Self {
        self.y_cols = Some(cols.iter().map(|&s| s.into()).collect());
        self
    }
    /// Set the Y-axis label text.
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }
    /// Set the color palette for the lines.
    #[must_use]
    pub fn palette(mut self, palette: PaletteSpec) -> Self {
        self.palette = Some(palette);
        self
    }
    /// Set the stroke width of each line in screen units.
    #[must_use]
    pub fn line_width(mut self, width: f64) -> Self {
        self.line_width = Some(width);
        self
    }
    /// Set the size of the scatter markers drawn at each data point.
    #[must_use]
    pub fn point_size(mut self, size: f64) -> Self {
        self.point_size = Some(size);
        self
    }
    /// Set a custom hover tooltip.
    #[must_use]
    pub fn tooltips(mut self, tooltips: TooltipSpec) -> Self {
        self.tooltips = Some(tooltips);
        self
    }
    /// Configure the X axis appearance.
    #[must_use]
    pub fn x_axis(mut self, axis: AxisConfig) -> Self {
        self.x_axis = Some(axis);
        self
    }
    /// Configure the Y axis appearance.
    #[must_use]
    pub fn y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }

    /// Build the config, returning an error if any required field is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if any required field was not set.
    pub fn build(self) -> Result<LineConfig, ChartError> {
        Ok(LineConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            y_cols: self.y_cols.ok_or(ChartError::MissingField("y_cols"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
            palette: self.palette,
            line_width: self.line_width,
            point_size: self.point_size,
            tooltips: self.tooltips,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for a horizontal bar chart.
///
/// Horizontal bar charts are useful for displaying ranked or categorical data
/// where the category labels are long strings. The bars extend horizontally
/// from left to right, with categories listed vertically.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = HBarConfig::builder()
///     .category("department")
///     .value("headcount")
///     .x_label("Employees")
///     .build()?;
/// ```
pub struct HBarConfig {
    /// Column name for the categorical axis (displayed vertically).
    pub category_col: String,
    /// Column name for the numeric values (bar lengths).
    pub value_col: String,
    /// Label displayed on the X axis (the value axis for horizontal bars).
    pub x_label: String,
    /// Fill color for the bars as a hex string (e.g. `"#e74c3c"`).
    /// Defaults to `"#4C72B0"` when `None`.
    pub color: Option<String>,
    /// Custom hover tooltip.  Defaults to the chart column names when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis (value axis) display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis (category axis) display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`HBarConfig`].
///
/// All fields are required. Calling [`build`](HBarConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct HBarConfigBuilder {
    category_col: Option<String>,
    value_col: Option<String>,
    x_label: Option<String>,
    color: Option<String>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl HBarConfig {
    /// Create a new builder for a horizontal bar chart configuration.
    #[must_use] 
    pub fn builder() -> HBarConfigBuilder {
        HBarConfigBuilder {
            category_col: None,
            value_col: None,
            x_label: None,
            color: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl HBarConfigBuilder {
    /// Set the category column name.
    #[must_use] 
    pub fn category(mut self, col: &str) -> Self {
        self.category_col = Some(col.into());
        self
    }
    /// Set the numeric value column name.
    #[must_use] 
    pub fn value(mut self, col: &str) -> Self {
        self.value_col = Some(col.into());
        self
    }
    /// Set the X-axis label text.
    #[must_use]
    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = Some(label.into());
        self
    }
    /// Set the fill color for the bars as a hex string (e.g. `"#e74c3c"`).
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.into());
        self
    }
    /// Set a custom hover tooltip.
    #[must_use]
    pub fn tooltips(mut self, tooltips: TooltipSpec) -> Self {
        self.tooltips = Some(tooltips);
        self
    }
    /// Configure the X axis (value axis) appearance.
    #[must_use]
    pub fn x_axis(mut self, axis: AxisConfig) -> Self {
        self.x_axis = Some(axis);
        self
    }
    /// Configure the Y axis (category axis) appearance.
    #[must_use]
    pub fn y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }

    /// Build the config, returning an error if any required field is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if any required field was not set.
    pub fn build(self) -> Result<HBarConfig, ChartError> {
        Ok(HBarConfig {
            category_col: self
                .category_col
                .ok_or(ChartError::MissingField("category_col"))?,
            value_col: self
                .value_col
                .ok_or(ChartError::MissingField("value_col"))?,
            x_label: self.x_label.ok_or(ChartError::MissingField("x_label"))?,
            color: self.color,
            tooltips: self.tooltips,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for a scatter plot.
///
/// Scatter plots display individual data points as circles positioned by their
/// X and Y values. When multiple scatter (or line) charts share the same
/// `source_key` on a page, selecting points in one chart highlights the
/// corresponding points in the others via Bokeh's linked selection.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = ScatterConfig::builder()
///     .x("revenue")
///     .y("profit")
///     .x_label("Revenue (k)")
///     .y_label("Profit (k)")
///     .build()?;
/// ```
pub struct ScatterConfig {
    /// Column name for the X-axis values.
    pub x_col: String,
    /// Column name for the Y-axis values.
    pub y_col: String,
    /// Label displayed on the X axis.
    pub x_label: String,
    /// Label displayed on the Y axis.
    pub y_label: String,
    /// Fill color for the markers as a hex string.  Defaults to `"#4C72B0"`.
    pub color: Option<String>,
    /// Bokeh marker type (e.g. `"circle"`, `"square"`, `"diamond"`,
    /// `"triangle"`, `"inverted_triangle"`, `"hex"`, `"star"`).
    /// Defaults to `"circle"` when `None`.
    pub marker: Option<String>,
    /// Marker size in screen units.  Defaults to `10` when `None`.
    pub marker_size: Option<f64>,
    /// Fill alpha (0.0 = transparent, 1.0 = opaque).  Defaults to `0.7`.
    pub alpha: Option<f64>,
    /// Custom hover tooltip.  Defaults to the chart column names when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`ScatterConfig`].
///
/// All fields are required. Calling [`build`](ScatterConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct ScatterConfigBuilder {
    x_col: Option<String>,
    y_col: Option<String>,
    x_label: Option<String>,
    y_label: Option<String>,
    color: Option<String>,
    marker: Option<String>,
    marker_size: Option<f64>,
    alpha: Option<f64>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl ScatterConfig {
    /// Create a new builder for a scatter plot configuration.
    #[must_use] 
    pub fn builder() -> ScatterConfigBuilder {
        ScatterConfigBuilder {
            x_col: None,
            y_col: None,
            x_label: None,
            y_label: None,
            color: None,
            marker: None,
            marker_size: None,
            alpha: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl ScatterConfigBuilder {
    /// Set the X-axis value column name.
    #[must_use] 
    pub fn x(mut self, col: &str) -> Self {
        self.x_col = Some(col.into());
        self
    }
    /// Set the Y-axis value column name.
    #[must_use] 
    pub fn y(mut self, col: &str) -> Self {
        self.y_col = Some(col.into());
        self
    }
    /// Set the X-axis label text.
    #[must_use] 
    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = Some(label.into());
        self
    }
    /// Set the Y-axis label text.
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }
    /// Set the fill color for the markers as a hex string.
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.into());
        self
    }
    /// Set the Bokeh marker type (e.g. `"circle"`, `"square"`, `"diamond"`).
    #[must_use]
    pub fn marker(mut self, marker: &str) -> Self {
        self.marker = Some(marker.into());
        self
    }
    /// Set the marker size in screen units.
    #[must_use]
    pub fn marker_size(mut self, size: f64) -> Self {
        self.marker_size = Some(size);
        self
    }
    /// Set the fill alpha (0.0 = transparent, 1.0 = opaque).
    #[must_use]
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(alpha);
        self
    }
    /// Set a custom hover tooltip.
    #[must_use]
    pub fn tooltips(mut self, tooltips: TooltipSpec) -> Self {
        self.tooltips = Some(tooltips);
        self
    }
    /// Configure the X axis appearance.
    #[must_use]
    pub fn x_axis(mut self, axis: AxisConfig) -> Self {
        self.x_axis = Some(axis);
        self
    }
    /// Configure the Y axis appearance.
    #[must_use]
    pub fn y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }

    /// Build the config, returning an error if any required field is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if any required field was not set.
    pub fn build(self) -> Result<ScatterConfig, ChartError> {
        Ok(ScatterConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            y_col: self.y_col.ok_or(ChartError::MissingField("y_col"))?,
            x_label: self.x_label.ok_or(ChartError::MissingField("x_label"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
            color: self.color,
            marker: self.marker,
            marker_size: self.marker_size,
            alpha: self.alpha,
            tooltips: self.tooltips,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
        })
    }
}

// ── Chart config enum ────────────────────────────────────────────────────────

/// Enum wrapping all supported chart configuration types.
///
/// Each variant holds the typed configuration for one chart type. This is
/// stored inside [`ChartSpec`] and used by the renderer to determine how to
/// build the Bokeh figure. You typically won't construct this directly;
/// instead use the convenience constructors on [`ChartSpecBuilder`] (e.g.
/// [`ChartSpecBuilder::bar`], [`ChartSpecBuilder::line`]).
pub enum ChartConfig {
    /// A vertical grouped bar chart. See [`GroupedBarConfig`].
    GroupedBar(GroupedBarConfig),
    /// A multi-line chart. See [`LineConfig`].
    Line(LineConfig),
    /// A horizontal bar chart. See [`HBarConfig`].
    HBar(HBarConfig),
    /// A scatter plot. See [`ScatterConfig`].
    Scatter(ScatterConfig),
}

impl ChartConfig {
    /// Return the string identifier used by the Python renderer to dispatch
    /// chart construction (e.g. `"grouped_bar"`, `"line_multi"`).
    #[must_use] 
    pub fn chart_type_str(&self) -> &'static str {
        match self {
            ChartConfig::GroupedBar(_) => "grouped_bar",
            ChartConfig::Line(_) => "line_multi",
            ChartConfig::HBar(_) => "hbar",
            ChartConfig::Scatter(_) => "scatter",
        }
    }
}

// ── Layout structs ──────────────────────────────────────────────────────────

/// Position and span of a chart within the page's CSS grid layout.
///
/// The page grid has a fixed number of columns (set by
/// [`PageBuilder::new`](crate::pages::PageBuilder::new)). Each chart occupies
/// one or more columns in a given row.
pub struct GridCell {
    /// Zero-based row index.
    pub row: usize,
    /// Zero-based column index.
    pub col: usize,
    /// Number of grid columns this chart spans (minimum 1).
    pub col_span: usize,
}

/// A fully specified chart ready to be placed on a page.
///
/// Combines a chart's title, data source reference, typed configuration,
/// grid position, and filter opt-in flag. Constructed via
/// [`ChartSpecBuilder`].
pub struct ChartSpec {
    /// Display title rendered above the chart.
    pub title: String,
    /// Key into the frames dictionary identifying which `DataFrame` to use.
    /// Must match a key registered with [`Dashboard::add_df`](crate::Dashboard::add_df).
    pub source_key: String,
    /// Typed chart configuration (bar, line, hbar, or scatter).
    pub config: ChartConfig,
    /// Position of this chart in the page grid.
    pub grid: GridCell,
    /// Whether this chart participates in page-level filtering via
    /// [`CDSView`](https://docs.bokeh.org/en/latest/docs/reference/models/sources.html#cdsview).
    /// Set to `true` by calling [`ChartSpecBuilder::filtered`].
    pub filtered: bool,
    /// Explicit figure width in pixels.  When `None`, the chart uses
    /// `sizing_mode="stretch_width"` to fill its grid cell.
    pub width: Option<u32>,
    /// Explicit figure height in pixels.  When `None`, each chart type uses
    /// a sensible default (typically 400 px).
    pub height: Option<u32>,
}

// ── Filter types ─────────────────────────────────────────────────────────────

/// Configuration for a single interactive filter widget.
///
/// Each variant maps to a specific Bokeh widget and filter model:
///
/// | Variant | Widget | Bokeh filter |
/// |---|---|---|
/// | [`Range`](FilterConfig::Range) | `RangeSlider` | `BooleanFilter` |
/// | [`Select`](FilterConfig::Select) | `Select` (dropdown with "All") | `BooleanFilter` |
/// | [`Group`](FilterConfig::Group) | `Select` (dropdown) | `GroupFilter` |
/// | [`Threshold`](FilterConfig::Threshold) | `Switch` (toggle) | `BooleanFilter` |
/// | [`TopN`](FilterConfig::TopN) | `Slider` | `IndexFilter` |
///
/// Multiple filters targeting the same `source_key` are combined
/// automatically via Bokeh's `IntersectionFilter`.
pub enum FilterConfig {
    /// A range slider that filters rows where the column value falls within
    /// `[min, max]`. The slider moves in increments of `step`.
    Range { min: f64, max: f64, step: f64 },
    /// A dropdown selector with an "All" option. Selecting a specific value
    /// filters rows to those matching that value; "All" shows everything.
    Select { options: Vec<String> },
    /// A dropdown selector that uses Bokeh's `GroupFilter` to show only rows
    /// belonging to the selected group. Unlike [`Select`](FilterConfig::Select),
    /// there is no "All" option.
    Group { options: Vec<String> },
    /// A toggle switch that, when enabled, filters rows based on a threshold
    /// comparison. If `above` is `true`, keeps rows where the column value
    /// is greater than `value`; if `false`, keeps rows below the threshold.
    Threshold { value: f64, above: bool },
    /// A slider that limits the display to the top (or bottom) N rows, sorted
    /// by the filter's column. Uses Bokeh's `IndexFilter` to select row
    /// indices after sorting.
    TopN { max_n: usize, descending: bool },
}

/// A declarative filter definition attached to a page.
///
/// Each `FilterSpec` targets a specific data source and column, producing a
/// Bokeh widget with a `CustomJS` callback that updates the filter in real
/// time. Charts on the same page that share the filter's `source_key` and
/// have been marked as [`filtered`](ChartSpecBuilder::filtered) will respond
/// to changes.
///
/// Use the factory methods ([`range`](FilterSpec::range),
/// [`select`](FilterSpec::select), [`group`](FilterSpec::group),
/// [`threshold`](FilterSpec::threshold), [`top_n`](FilterSpec::top_n)) to
/// construct instances.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// // Only show rows where revenue is between 40 and 320
/// let f = FilterSpec::range("my_data", "revenue", "Revenue Range", 40.0, 320.0, 10.0);
/// ```
pub struct FilterSpec {
    /// Key identifying which data source this filter applies to.
    pub source_key: String,
    /// Column name in the data source to filter on.
    pub column: String,
    /// Human-readable label displayed next to the widget.
    pub label: String,
    /// The filter type and its parameters.
    pub config: FilterConfig,
}

// ── ChartSpec builder ────────────────────────────────────────────────────────

/// Fluent builder for constructing [`ChartSpec`] instances.
///
/// Provides convenience constructors for each chart type ([`bar`](Self::bar),
/// [`line`](Self::line), [`hbar`](Self::hbar), [`scatter`](Self::scatter))
/// and chainable methods for grid positioning and filter opt-in.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let spec = ChartSpecBuilder::scatter(
///         "Revenue vs Profit",
///         "performance_data",
///         ScatterConfig::builder()
///             .x("revenue").y("profit")
///             .x_label("Revenue").y_label("Profit")
///             .build()?,
///     )
///     .at(0, 0, 2)    // row 0, col 0, spanning 2 columns
///     .filtered()      // opt into page-level filters
///     .build();
/// ```
pub struct ChartSpecBuilder {
    title: String,
    source_key: String,
    config: ChartConfig,
    grid: GridCell,
    filtered: bool,
    width: Option<u32>,
    height: Option<u32>,
}

impl ChartSpecBuilder {
    /// Create a builder with an arbitrary [`ChartConfig`].
    ///
    /// Prefer the typed constructors ([`bar`](Self::bar), [`line`](Self::line),
    /// etc.) for a more ergonomic API.
    #[must_use] 
    pub fn new(title: &str, source_key: &str, config: ChartConfig) -> Self {
        Self {
            title: title.into(),
            source_key: source_key.into(),
            config,
            grid: GridCell {
                row: 0,
                col: 0,
                col_span: 1,
            },
            filtered: false,
            width: None,
            height: None,
        }
    }

    /// Create a grouped bar chart spec.
    #[must_use] 
    pub fn bar(title: &str, key: &str, config: GroupedBarConfig) -> Self {
        Self::new(title, key, ChartConfig::GroupedBar(config))
    }

    /// Create a multi-line chart spec.
    #[must_use] 
    pub fn line(title: &str, key: &str, config: LineConfig) -> Self {
        Self::new(title, key, ChartConfig::Line(config))
    }

    /// Create a horizontal bar chart spec.
    #[must_use] 
    pub fn hbar(title: &str, key: &str, config: HBarConfig) -> Self {
        Self::new(title, key, ChartConfig::HBar(config))
    }

    /// Create a scatter plot spec.
    #[must_use] 
    pub fn scatter(title: &str, key: &str, config: ScatterConfig) -> Self {
        Self::new(title, key, ChartConfig::Scatter(config))
    }

    /// Set the grid position and column span.
    ///
    /// `row` and `col` are zero-based indices into the page grid. `span`
    /// controls how many columns this chart occupies (e.g. `2` for full-width
    /// on a 2-column grid).
    #[must_use] 
    pub fn at(mut self, row: usize, col: usize, span: usize) -> Self {
        self.grid = GridCell {
            row,
            col,
            col_span: span,
        };
        self
    }

    /// Mark this chart as filtered, opting it into `CDSView`-based filtering.
    ///
    /// Only charts with the same `source_key` as a page's [`FilterSpec`]s
    /// will be affected. Charts that are not marked as filtered will display
    /// all data regardless of filter state.
    #[must_use]
    pub fn filtered(mut self) -> Self {
        self.filtered = true;
        self
    }

    /// Set explicit pixel dimensions for the figure.
    ///
    /// When called, the chart uses `sizing_mode="fixed"` at the given
    /// dimensions instead of the default `"stretch_width"` responsive layout.
    /// Either dimension can be changed independently by calling this method
    /// once with the desired values (both must be supplied together).
    #[must_use]
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Consume the builder and produce a [`ChartSpec`].
    #[must_use]
    pub fn build(self) -> ChartSpec {
        ChartSpec {
            title: self.title,
            source_key: self.source_key,
            config: self.config,
            grid: self.grid,
            filtered: self.filtered,
            width: self.width,
            height: self.height,
        }
    }
}

// ── FilterSpec factory methods ───────────────────────────────────────────────

impl FilterSpec {
    /// Create a range slider filter.
    ///
    /// Produces a `RangeSlider` widget that filters rows where the column
    /// value falls within the selected `[min, max]` range. The slider moves
    /// in increments of `step`.
    #[must_use] 
    pub fn range(
        source_key: &str,
        column: &str,
        label: &str,
        min: f64,
        max: f64,
        step: f64,
    ) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::Range { min, max, step },
        }
    }

    /// Create a dropdown select filter with an "All" option.
    ///
    /// The dropdown lists each value in `options` plus an "All" entry at the
    /// top. Selecting "All" removes the filter; selecting a specific value
    /// keeps only rows matching that value.
    pub fn select(source_key: &str, column: &str, label: &str, options: Vec<&str>) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::Select {
                options: options.into_iter().map(Into::into).collect(),
            },
        }
    }

    /// Create a group filter (dropdown without an "All" option).
    ///
    /// Uses Bokeh's `GroupFilter` to show only rows belonging to the selected
    /// group. The first option is selected by default.
    pub fn group(source_key: &str, column: &str, label: &str, options: Vec<&str>) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::Group {
                options: options.into_iter().map(Into::into).collect(),
            },
        }
    }

    /// Create a threshold toggle filter.
    ///
    /// Produces a `Switch` widget. When toggled on, rows are filtered based
    /// on whether the column value is above (`above = true`) or below
    /// (`above = false`) the given `value`.
    #[must_use] 
    pub fn threshold(source_key: &str, column: &str, label: &str, value: f64, above: bool) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::Threshold { value, above },
        }
    }

    /// Create a top-N slider filter.
    ///
    /// Produces a `Slider` widget that limits display to the top (or bottom)
    /// N rows sorted by the filter's column. `max_n` sets the slider's upper
    /// bound. If `descending` is `true`, the highest values are kept; if
    /// `false`, the lowest.
    #[must_use] 
    pub fn top_n(
        source_key: &str,
        column: &str,
        label: &str,
        max_n: usize,
        descending: bool,
    ) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::TopN { max_n, descending },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── GroupedBarConfig builder ──────────────────────────────────────────────

    #[test]
    fn grouped_bar_missing_x_col() {
        assert!(matches!(
            GroupedBarConfig::builder()
                .group("g")
                .value("v")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("x_col"))
        ));
    }

    #[test]
    fn grouped_bar_missing_group_col() {
        assert!(matches!(
            GroupedBarConfig::builder()
                .x("x")
                .value("v")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("group_col"))
        ));
    }

    #[test]
    fn grouped_bar_missing_value_col() {
        assert!(matches!(
            GroupedBarConfig::builder()
                .x("x")
                .group("g")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("value_col"))
        ));
    }

    #[test]
    fn grouped_bar_missing_y_label() {
        assert!(matches!(
            GroupedBarConfig::builder()
                .x("x")
                .group("g")
                .value("v")
                .build(),
            Err(ChartError::MissingField("y_label"))
        ));
    }

    #[test]
    fn grouped_bar_build_success() {
        let cfg = GroupedBarConfig::builder()
            .x("month")
            .group("category")
            .value("revenue")
            .y_label("USD")
            .build()
            .unwrap();
        assert_eq!(cfg.x_col, "month");
        assert_eq!(cfg.group_col, "category");
        assert_eq!(cfg.value_col, "revenue");
        assert_eq!(cfg.y_label, "USD");
    }

    // ── LineConfig builder ────────────────────────────────────────────────────

    #[test]
    fn line_missing_x_col() {
        assert!(matches!(
            LineConfig::builder().y_cols(&["a"]).y_label("Y").build(),
            Err(ChartError::MissingField("x_col"))
        ));
    }

    #[test]
    fn line_missing_y_cols() {
        assert!(matches!(
            LineConfig::builder().x("x").y_label("Y").build(),
            Err(ChartError::MissingField("y_cols"))
        ));
    }

    #[test]
    fn line_missing_y_label() {
        assert!(matches!(
            LineConfig::builder().x("x").y_cols(&["a"]).build(),
            Err(ChartError::MissingField("y_label"))
        ));
    }

    #[test]
    fn line_build_success() {
        let cfg = LineConfig::builder()
            .x("month")
            .y_cols(&["rev", "exp"])
            .y_label("USD")
            .build()
            .unwrap();
        assert_eq!(cfg.x_col, "month");
        assert_eq!(cfg.y_cols, vec!["rev".to_string(), "exp".to_string()]);
        assert_eq!(cfg.y_label, "USD");
    }

    // ── HBarConfig builder ────────────────────────────────────────────────────

    #[test]
    fn hbar_missing_category_col() {
        assert!(matches!(
            HBarConfig::builder().value("v").x_label("X").build(),
            Err(ChartError::MissingField("category_col"))
        ));
    }

    #[test]
    fn hbar_missing_value_col() {
        assert!(matches!(
            HBarConfig::builder().category("c").x_label("X").build(),
            Err(ChartError::MissingField("value_col"))
        ));
    }

    #[test]
    fn hbar_missing_x_label() {
        assert!(matches!(
            HBarConfig::builder().category("c").value("v").build(),
            Err(ChartError::MissingField("x_label"))
        ));
    }

    #[test]
    fn hbar_build_success() {
        let cfg = HBarConfig::builder()
            .category("dept")
            .value("headcount")
            .x_label("Employees")
            .build()
            .unwrap();
        assert_eq!(cfg.category_col, "dept");
        assert_eq!(cfg.value_col, "headcount");
        assert_eq!(cfg.x_label, "Employees");
    }

    // ── ScatterConfig builder ─────────────────────────────────────────────────

    #[test]
    fn scatter_missing_x_col() {
        assert!(matches!(
            ScatterConfig::builder()
                .y("y")
                .x_label("X")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("x_col"))
        ));
    }

    #[test]
    fn scatter_missing_y_col() {
        assert!(matches!(
            ScatterConfig::builder()
                .x("x")
                .x_label("X")
                .y_label("Y")
                .build(),
            Err(ChartError::MissingField("y_col"))
        ));
    }

    #[test]
    fn scatter_missing_x_label() {
        assert!(matches!(
            ScatterConfig::builder().x("x").y("y").y_label("Y").build(),
            Err(ChartError::MissingField("x_label"))
        ));
    }

    #[test]
    fn scatter_missing_y_label() {
        assert!(matches!(
            ScatterConfig::builder().x("x").y("y").x_label("X").build(),
            Err(ChartError::MissingField("y_label"))
        ));
    }

    #[test]
    fn scatter_build_success() {
        let cfg = ScatterConfig::builder()
            .x("revenue")
            .y("profit")
            .x_label("Revenue")
            .y_label("Profit")
            .build()
            .unwrap();
        assert_eq!(cfg.x_col, "revenue");
        assert_eq!(cfg.y_col, "profit");
        assert_eq!(cfg.x_label, "Revenue");
        assert_eq!(cfg.y_label, "Profit");
    }

    // ── ChartConfig::chart_type_str ───────────────────────────────────────────

    #[test]
    fn chart_type_str_grouped_bar() {
        let cfg = ChartConfig::GroupedBar(
            GroupedBarConfig::builder()
                .x("x")
                .group("g")
                .value("v")
                .y_label("Y")
                .build()
                .unwrap(),
        );
        assert_eq!(cfg.chart_type_str(), "grouped_bar");
    }

    #[test]
    fn chart_type_str_line() {
        let cfg = ChartConfig::Line(
            LineConfig::builder()
                .x("x")
                .y_cols(&["a"])
                .y_label("Y")
                .build()
                .unwrap(),
        );
        assert_eq!(cfg.chart_type_str(), "line_multi");
    }

    #[test]
    fn chart_type_str_hbar() {
        let cfg = ChartConfig::HBar(
            HBarConfig::builder()
                .category("c")
                .value("v")
                .x_label("X")
                .build()
                .unwrap(),
        );
        assert_eq!(cfg.chart_type_str(), "hbar");
    }

    #[test]
    fn chart_type_str_scatter() {
        let cfg = ChartConfig::Scatter(
            ScatterConfig::builder()
                .x("x")
                .y("y")
                .x_label("X")
                .y_label("Y")
                .build()
                .unwrap(),
        );
        assert_eq!(cfg.chart_type_str(), "scatter");
    }

    // ── ChartSpecBuilder ──────────────────────────────────────────────────────

    #[test]
    fn chart_spec_builder_defaults() {
        let cfg = HBarConfig::builder()
            .category("c")
            .value("v")
            .x_label("X")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::hbar("My Chart", "my_data", cfg).build();
        assert_eq!(spec.title, "My Chart");
        assert_eq!(spec.source_key, "my_data");
        assert_eq!(spec.grid.row, 0);
        assert_eq!(spec.grid.col, 0);
        assert_eq!(spec.grid.col_span, 1);
        assert!(!spec.filtered);
    }

    #[test]
    fn chart_spec_builder_at_sets_grid() {
        let cfg = HBarConfig::builder()
            .category("c")
            .value("v")
            .x_label("X")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::hbar("Chart", "data", cfg)
            .at(2, 1, 3)
            .build();
        assert_eq!(spec.grid.row, 2);
        assert_eq!(spec.grid.col, 1);
        assert_eq!(spec.grid.col_span, 3);
    }

    #[test]
    fn chart_spec_builder_filtered_flag() {
        let cfg = HBarConfig::builder()
            .category("c")
            .value("v")
            .x_label("X")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::hbar("Chart", "data", cfg)
            .filtered()
            .build();
        assert!(spec.filtered);
    }

    #[test]
    fn chart_spec_builder_bar_constructor() {
        let cfg = GroupedBarConfig::builder()
            .x("x")
            .group("g")
            .value("v")
            .y_label("Y")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::bar("Bar Chart", "src", cfg).build();
        assert_eq!(spec.config.chart_type_str(), "grouped_bar");
    }

    #[test]
    fn chart_spec_builder_line_constructor() {
        let cfg = LineConfig::builder()
            .x("x")
            .y_cols(&["a"])
            .y_label("Y")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::line("Line Chart", "src", cfg).build();
        assert_eq!(spec.config.chart_type_str(), "line_multi");
    }

    #[test]
    fn chart_spec_builder_scatter_constructor() {
        let cfg = ScatterConfig::builder()
            .x("x")
            .y("y")
            .x_label("X")
            .y_label("Y")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::scatter("Scatter", "src", cfg).build();
        assert_eq!(spec.config.chart_type_str(), "scatter");
    }

    // ── FilterSpec factory methods ────────────────────────────────────────────

    #[test]
    fn filter_spec_range_stores_config() {
        let f = FilterSpec::range("src", "col", "Label", 10.0, 100.0, 5.0);
        assert_eq!(f.source_key, "src");
        assert_eq!(f.column, "col");
        assert_eq!(f.label, "Label");
        match f.config {
            FilterConfig::Range { min, max, step } => {
                assert_eq!(min, 10.0);
                assert_eq!(max, 100.0);
                assert_eq!(step, 5.0);
            }
            _ => panic!("expected Range config"),
        }
    }

    #[test]
    fn filter_spec_select_stores_config() {
        let f = FilterSpec::select("src", "col", "Label", vec!["A", "B", "C"]);
        match f.config {
            FilterConfig::Select { options } => {
                assert_eq!(options, vec!["A", "B", "C"]);
            }
            _ => panic!("expected Select config"),
        }
    }

    #[test]
    fn filter_spec_group_stores_config() {
        let f = FilterSpec::group("src", "col", "Label", vec!["X", "Y"]);
        match f.config {
            FilterConfig::Group { options } => {
                assert_eq!(options, vec!["X", "Y"]);
            }
            _ => panic!("expected Group config"),
        }
    }

    #[test]
    fn filter_spec_threshold_stores_config() {
        let f = FilterSpec::threshold("src", "col", "Label", 50.0, true);
        match f.config {
            FilterConfig::Threshold { value, above } => {
                assert_eq!(value, 50.0);
                assert!(above);
            }
            _ => panic!("expected Threshold config"),
        }
    }

    #[test]
    fn filter_spec_top_n_stores_config() {
        let f = FilterSpec::top_n("src", "col", "Label", 10, false);
        match f.config {
            FilterConfig::TopN { max_n, descending } => {
                assert_eq!(max_n, 10);
                assert!(!descending);
            }
            _ => panic!("expected TopN config"),
        }
    }
}
