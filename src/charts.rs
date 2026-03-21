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

// ── Chart configuration structs ──────────────────────────────────────────────

/// Configuration for a grouped bar chart.
///
/// Grouped bar charts display vertical bars organized by a categorical X axis,
/// with bars within each group distinguished by a grouping column. The
/// DataFrame must contain three columns: one for the X-axis categories, one
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
}

impl GroupedBarConfig {
    /// Create a new builder for a grouped bar chart configuration.
    pub fn builder() -> GroupedBarConfigBuilder {
        GroupedBarConfigBuilder {
            x_col: None,
            group_col: None,
            value_col: None,
            y_label: None,
        }
    }
}

impl GroupedBarConfigBuilder {
    /// Set the X-axis category column name.
    pub fn x(mut self, col: &str) -> Self { self.x_col = Some(col.into()); self }
    /// Set the grouping column name.
    pub fn group(mut self, col: &str) -> Self { self.group_col = Some(col.into()); self }
    /// Set the numeric value column name.
    pub fn value(mut self, col: &str) -> Self { self.value_col = Some(col.into()); self }
    /// Set the Y-axis label text.
    pub fn y_label(mut self, label: &str) -> Self { self.y_label = Some(label.into()); self }

    /// Build the config, returning an error if any required field is missing.
    pub fn build(self) -> Result<GroupedBarConfig, ChartError> {
        Ok(GroupedBarConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            group_col: self.group_col.ok_or(ChartError::MissingField("group_col"))?,
            value_col: self.value_col.ok_or(ChartError::MissingField("value_col"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
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
}

/// Builder for [`LineConfig`].
///
/// All fields are required. Calling [`build`](LineConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct LineConfigBuilder {
    x_col: Option<String>,
    y_cols: Option<Vec<String>>,
    y_label: Option<String>,
}

impl LineConfig {
    /// Create a new builder for a line chart configuration.
    pub fn builder() -> LineConfigBuilder {
        LineConfigBuilder {
            x_col: None,
            y_cols: None,
            y_label: None,
        }
    }
}

impl LineConfigBuilder {
    /// Set the X-axis column name.
    pub fn x(mut self, col: &str) -> Self { self.x_col = Some(col.into()); self }
    /// Set the Y-axis column names. Each column becomes a separate line.
    pub fn y_cols(mut self, cols: &[&str]) -> Self {
        self.y_cols = Some(cols.iter().map(|&s| s.into()).collect());
        self
    }
    /// Set the Y-axis label text.
    pub fn y_label(mut self, label: &str) -> Self { self.y_label = Some(label.into()); self }

    /// Build the config, returning an error if any required field is missing.
    pub fn build(self) -> Result<LineConfig, ChartError> {
        Ok(LineConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            y_cols: self.y_cols.ok_or(ChartError::MissingField("y_cols"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
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
}

/// Builder for [`HBarConfig`].
///
/// All fields are required. Calling [`build`](HBarConfigBuilder::build)
/// without setting a field returns [`ChartError::MissingField`].
pub struct HBarConfigBuilder {
    category_col: Option<String>,
    value_col: Option<String>,
    x_label: Option<String>,
}

impl HBarConfig {
    /// Create a new builder for a horizontal bar chart configuration.
    pub fn builder() -> HBarConfigBuilder {
        HBarConfigBuilder {
            category_col: None,
            value_col: None,
            x_label: None,
        }
    }
}

impl HBarConfigBuilder {
    /// Set the category column name.
    pub fn category(mut self, col: &str) -> Self { self.category_col = Some(col.into()); self }
    /// Set the numeric value column name.
    pub fn value(mut self, col: &str) -> Self { self.value_col = Some(col.into()); self }
    /// Set the X-axis label text.
    pub fn x_label(mut self, label: &str) -> Self { self.x_label = Some(label.into()); self }

    /// Build the config, returning an error if any required field is missing.
    pub fn build(self) -> Result<HBarConfig, ChartError> {
        Ok(HBarConfig {
            category_col: self.category_col.ok_or(ChartError::MissingField("category_col"))?,
            value_col: self.value_col.ok_or(ChartError::MissingField("value_col"))?,
            x_label: self.x_label.ok_or(ChartError::MissingField("x_label"))?,
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
}

impl ScatterConfig {
    /// Create a new builder for a scatter plot configuration.
    pub fn builder() -> ScatterConfigBuilder {
        ScatterConfigBuilder {
            x_col: None,
            y_col: None,
            x_label: None,
            y_label: None,
        }
    }
}

impl ScatterConfigBuilder {
    /// Set the X-axis value column name.
    pub fn x(mut self, col: &str) -> Self { self.x_col = Some(col.into()); self }
    /// Set the Y-axis value column name.
    pub fn y(mut self, col: &str) -> Self { self.y_col = Some(col.into()); self }
    /// Set the X-axis label text.
    pub fn x_label(mut self, label: &str) -> Self { self.x_label = Some(label.into()); self }
    /// Set the Y-axis label text.
    pub fn y_label(mut self, label: &str) -> Self { self.y_label = Some(label.into()); self }

    /// Build the config, returning an error if any required field is missing.
    pub fn build(self) -> Result<ScatterConfig, ChartError> {
        Ok(ScatterConfig {
            x_col: self.x_col.ok_or(ChartError::MissingField("x_col"))?,
            y_col: self.y_col.ok_or(ChartError::MissingField("y_col"))?,
            x_label: self.x_label.ok_or(ChartError::MissingField("x_label"))?,
            y_label: self.y_label.ok_or(ChartError::MissingField("y_label"))?,
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
    /// Key into the frames dictionary identifying which DataFrame to use.
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
}

impl ChartSpecBuilder {
    /// Create a builder with an arbitrary [`ChartConfig`].
    ///
    /// Prefer the typed constructors ([`bar`](Self::bar), [`line`](Self::line),
    /// etc.) for a more ergonomic API.
    pub fn new(title: &str, source_key: &str, config: ChartConfig) -> Self {
        Self {
            title: title.into(),
            source_key: source_key.into(),
            config,
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
        }
    }

    /// Create a grouped bar chart spec.
    pub fn bar(title: &str, key: &str, config: GroupedBarConfig) -> Self {
        Self::new(title, key, ChartConfig::GroupedBar(config))
    }

    /// Create a multi-line chart spec.
    pub fn line(title: &str, key: &str, config: LineConfig) -> Self {
        Self::new(title, key, ChartConfig::Line(config))
    }

    /// Create a horizontal bar chart spec.
    pub fn hbar(title: &str, key: &str, config: HBarConfig) -> Self {
        Self::new(title, key, ChartConfig::HBar(config))
    }

    /// Create a scatter plot spec.
    pub fn scatter(title: &str, key: &str, config: ScatterConfig) -> Self {
        Self::new(title, key, ChartConfig::Scatter(config))
    }

    /// Set the grid position and column span.
    ///
    /// `row` and `col` are zero-based indices into the page grid. `span`
    /// controls how many columns this chart occupies (e.g. `2` for full-width
    /// on a 2-column grid).
    pub fn at(mut self, row: usize, col: usize, span: usize) -> Self {
        self.grid = GridCell { row, col, col_span: span };
        self
    }

    /// Mark this chart as filtered, opting it into `CDSView`-based filtering.
    ///
    /// Only charts with the same `source_key` as a page's [`FilterSpec`]s
    /// will be affected. Charts that are not marked as filtered will display
    /// all data regardless of filter state.
    pub fn filtered(mut self) -> Self {
        self.filtered = true;
        self
    }

    /// Consume the builder and produce a [`ChartSpec`].
    pub fn build(self) -> ChartSpec {
        ChartSpec {
            title: self.title,
            source_key: self.source_key,
            config: self.config,
            grid: self.grid,
            filtered: self.filtered,
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
    pub fn range(source_key: &str, column: &str, label: &str, min: f64, max: f64, step: f64) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::Range { min, max, step } }
    }

    /// Create a dropdown select filter with an "All" option.
    ///
    /// The dropdown lists each value in `options` plus an "All" entry at the
    /// top. Selecting "All" removes the filter; selecting a specific value
    /// keeps only rows matching that value.
    pub fn select(source_key: &str, column: &str, label: &str, options: Vec<&str>) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::Select { options: options.into_iter().map(Into::into).collect() } }
    }

    /// Create a group filter (dropdown without an "All" option).
    ///
    /// Uses Bokeh's `GroupFilter` to show only rows belonging to the selected
    /// group. The first option is selected by default.
    pub fn group(source_key: &str, column: &str, label: &str, options: Vec<&str>) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::Group { options: options.into_iter().map(Into::into).collect() } }
    }

    /// Create a threshold toggle filter.
    ///
    /// Produces a `Switch` widget. When toggled on, rows are filtered based
    /// on whether the column value is above (`above = true`) or below
    /// (`above = false`) the given `value`.
    pub fn threshold(source_key: &str, column: &str, label: &str, value: f64, above: bool) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::Threshold { value, above } }
    }

    /// Create a top-N slider filter.
    ///
    /// Produces a `Slider` widget that limits display to the top (or bottom)
    /// N rows sorted by the filter's column. `max_n` sets the slider's upper
    /// bound. If `descending` is `true`, the highest values are kept; if
    /// `false`, the lowest.
    pub fn top_n(source_key: &str, column: &str, label: &str, max_n: usize, descending: bool) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::TopN { max_n, descending } }
    }
}
