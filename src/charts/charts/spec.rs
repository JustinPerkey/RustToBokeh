use super::{ChartConfig, GridCell, ChartSpec};
use super::box_plot::BoxPlotConfig;
use super::density::DensityConfig;
use super::grouped_bar::GroupedBarConfig;
use super::hbar::HBarConfig;
use super::histogram::HistogramConfig;
use super::line::LineConfig;
use super::pie::PieConfig;
use super::scatter::ScatterConfig;

/// Fluent builder for constructing [`ChartSpec`] instances.
///
/// Provides a constructor for each of the eight supported chart types and
/// chainable methods for grid positioning, filter opt-in, and explicit sizing.
///
/// | Constructor | Chart type |
/// |---|---|
/// | [`bar`](Self::bar) | Grouped vertical bars |
/// | [`line`](Self::line) | Multi-series line chart |
/// | [`hbar`](Self::hbar) | Horizontal bar chart |
/// | [`scatter`](Self::scatter) | Scatter plot |
/// | [`pie`](Self::pie) | Pie or donut chart |
/// | [`histogram`](Self::histogram) | Histogram (count / PDF / CDF) |
/// | [`box_plot`](Self::box_plot) | Box-and-whisker plot |
/// | [`density`](Self::density) | Density plot (sina or violin, auto-selected) |
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

    /// Create a grouped vertical bar chart spec.
    ///
    /// Bars are grouped by an X-axis category column, with sub-groups
    /// distinguished by a second grouping column and coloured by palette.
    /// See [`GroupedBarConfig`] for all configuration options.
    #[must_use]
    pub fn bar(title: &str, key: &str, config: GroupedBarConfig) -> Self {
        Self::new(title, key, ChartConfig::GroupedBar(config))
    }

    /// Create a multi-series line chart spec.
    ///
    /// Each column listed in [`LineConfig::y_cols`] becomes a separate line.
    /// Line and scatter charts that share the same `key` on a page share one
    /// `ColumnDataSource`, enabling linked hover and selection.
    #[must_use]
    pub fn line(title: &str, key: &str, config: LineConfig) -> Self {
        Self::new(title, key, ChartConfig::Line(config))
    }

    /// Create a horizontal bar chart spec.
    ///
    /// Useful for ranked or labelled categorical data where category names
    /// are long strings. See [`HBarConfig`] for all configuration options.
    #[must_use]
    pub fn hbar(title: &str, key: &str, config: HBarConfig) -> Self {
        Self::new(title, key, ChartConfig::HBar(config))
    }

    /// Create a scatter plot spec.
    ///
    /// Scatter charts that share the same `key` with line charts on the same
    /// page share a `ColumnDataSource`, so selecting points in one chart
    /// highlights the corresponding points in all others.
    #[must_use]
    pub fn scatter(title: &str, key: &str, config: ScatterConfig) -> Self {
        Self::new(title, key, ChartConfig::Scatter(config))
    }

    /// Create a pie or donut chart spec.
    ///
    /// Set [`PieConfig::inner_radius`] to render a donut instead of a solid pie.
    #[must_use]
    pub fn pie(title: &str, key: &str, config: PieConfig) -> Self {
        Self::new(title, key, ChartConfig::Pie(config))
    }

    /// Create a histogram spec.
    ///
    /// The DataFrame referenced by `key` must be a pre-computed histogram
    /// produced by [`compute_histogram`](crate::compute_histogram), which
    /// provides `left`, `right`, `count`, `pdf`, and `cdf` columns.
    /// [`HistogramConfig`] selects which statistic to render (count, PDF, CDF).
    #[must_use]
    pub fn histogram(title: &str, key: &str, config: HistogramConfig) -> Self {
        Self::new(title, key, ChartConfig::Histogram(config))
    }

    /// Create a box-and-whisker plot spec.
    ///
    /// The DataFrame referenced by `key` should contain pre-computed box
    /// statistics. Use [`compute_box_stats`](crate::compute_box_stats) to
    /// produce a compatible DataFrame from raw category + value data.
    #[must_use]
    pub fn box_plot(title: &str, key: &str, config: BoxPlotConfig) -> Self {
        Self::new(title, key, ChartConfig::BoxPlot(config))
    }

    /// Create a density plot spec (violin or sina, auto-selected).
    ///
    /// The DataFrame referenced by `key` should be in long format with one row
    /// per observation: a categorical column (X grouping) and a numeric column
    /// (Y values). The renderer automatically chooses **sina** (jittered
    /// scatter) when each category has few data points, or **violin** (filled
    /// KDE polygon) when a category is densely populated. The switch-over
    /// threshold defaults to 30 points and is configurable via
    /// [`DensityConfig::point_threshold`].
    #[must_use]
    pub fn density(title: &str, key: &str, config: DensityConfig) -> Self {
        Self::new(title, key, ChartConfig::Density(config))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::charts::box_plot::BoxPlotConfig;
    use crate::charts::charts::density::DensityConfig;
    use crate::charts::charts::grouped_bar::GroupedBarConfig;
    use crate::charts::charts::hbar::HBarConfig;
    use crate::charts::charts::line::LineConfig;
    use crate::charts::charts::pie::PieConfig;
    use crate::charts::charts::scatter::ScatterConfig;

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

    #[test]
    fn chart_type_str_pie() {
        let cfg = ChartConfig::Pie(
            PieConfig::builder()
                .label("category")
                .value("amount")
                .build()
                .unwrap(),
        );
        assert_eq!(cfg.chart_type_str(), "pie");
    }

    #[test]
    fn chart_type_str_box_plot() {
        let cfg = ChartConfig::BoxPlot(
            BoxPlotConfig::builder()
                .category("category")
                .q1("q1").q2("q2").q3("q3")
                .lower("lower").upper("upper")
                .y_label("Y")
                .build()
                .unwrap(),
        );
        assert_eq!(cfg.chart_type_str(), "box_plot");
    }

    #[test]
    fn chart_spec_builder_box_plot_constructor() {
        let cfg = BoxPlotConfig::builder()
            .category("category")
            .q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper")
            .y_label("Value")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::box_plot("Salary by Dept", "salary_box", cfg).build();
        assert_eq!(spec.config.chart_type_str(), "box_plot");
        assert_eq!(spec.title, "Salary by Dept");
        assert_eq!(spec.source_key, "salary_box");
    }

    #[test]
    fn chart_spec_builder_pie_constructor() {
        let cfg = PieConfig::builder()
            .label("category")
            .value("amount")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::pie("Market Share", "market_share", cfg).build();
        assert_eq!(spec.config.chart_type_str(), "pie");
        assert_eq!(spec.title, "Market Share");
        assert_eq!(spec.source_key, "market_share");
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

    // ── ChartSpecBuilder::density ─────────────────────────────────────────────

    #[test]
    fn chart_type_str_density() {
        let cfg = ChartConfig::Density(
            DensityConfig::builder()
                .category("dept")
                .value("salary_k")
                .y_label("Salary")
                .build()
                .unwrap(),
        );
        assert_eq!(cfg.chart_type_str(), "density");
    }

    #[test]
    fn chart_spec_builder_density_constructor() {
        let cfg = DensityConfig::builder()
            .category("dept")
            .value("salary_k")
            .y_label("Salary (k USD)")
            .build()
            .unwrap();
        let spec = ChartSpecBuilder::density("Salary by Dept", "salary_raw", cfg).build();
        assert_eq!(spec.config.chart_type_str(), "density");
        assert_eq!(spec.title, "Salary by Dept");
        assert_eq!(spec.source_key, "salary_raw");
    }

    // ── ChartSpecBuilder::dimensions ──────────────────────────────────────────

    #[test]
    fn chart_spec_dimensions_default_none() {
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X")
            .build().unwrap();
        let spec = ChartSpecBuilder::hbar("Chart", "data", cfg).build();
        assert!(spec.width.is_none());
        assert!(spec.height.is_none());
    }

    #[test]
    fn chart_spec_dimensions_sets_width_and_height() {
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X")
            .build().unwrap();
        let spec = ChartSpecBuilder::hbar("Chart", "data", cfg)
            .dimensions(800, 600)
            .build();
        assert_eq!(spec.width, Some(800));
        assert_eq!(spec.height, Some(600));
    }

    #[test]
    fn chart_spec_dimensions_independent_of_filtered_and_grid() {
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X")
            .build().unwrap();
        let spec = ChartSpecBuilder::hbar("Chart", "data", cfg)
            .at(1, 0, 2)
            .filtered()
            .dimensions(1200, 400)
            .build();
        assert_eq!(spec.grid.row, 1);
        assert_eq!(spec.grid.col_span, 2);
        assert!(spec.filtered);
        assert_eq!(spec.width, Some(1200));
        assert_eq!(spec.height, Some(400));
    }
}
