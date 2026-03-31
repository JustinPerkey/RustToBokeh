use crate::charts::customization::axis::AxisConfig;
use crate::charts::customization::palette::PaletteSpec;
use crate::error::ChartError;

/// Configuration for a density plot (violin or sina chart).
///
/// A density plot visualises the distribution of a numeric variable across
/// categories. The renderer automatically selects the best variant based on
/// the number of data points per category:
///
/// - **Sina plot** (few points, ≤ `point_threshold`): each raw data point is
///   drawn as a scatter marker whose horizontal jitter is sampled uniformly
///   within the local KDE density envelope at that value, so points fill the
///   interior of the distribution rather than clustering on the boundary.
/// - **Violin plot** (many points, > `point_threshold`): a mirrored KDE
///   polygon is drawn per category, showing the full probability density
///   shape. A median line is overlaid at the 50th percentile.
///
/// The default threshold is **50 points per category** (configurable via
/// [`point_threshold`](DensityConfigBuilder::point_threshold)).
///
/// # Data format
///
/// The chart expects a "long-format" `DataFrame` with one row per observation:
/// - A categorical column (X axis grouping, set via `.category()`)
/// - A numeric column (Y axis values, set via `.value()`)
///
/// This is the same shape produced by
/// [`build_salary_raw`](crate::build_salary_raw) and consumed by
/// [`compute_box_stats`](crate::compute_box_stats).
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = DensityConfig::builder()
///     .category("department")
///     .value("salary_k")
///     .y_label("Salary (k USD)")
///     .palette(PaletteSpec::Named("Set2".into()))
///     .build()?;
/// ```
pub struct DensityConfig {
    /// Column name for the category labels (X axis).
    pub category_col: String,
    /// Column name for the numeric values (Y axis).
    pub value_col: String,
    /// Label displayed on the Y axis.
    pub y_label: String,
    /// Color palette — one color per category.
    ///
    /// When set, each category gets a distinct color. When `None` and `color`
    /// is also `None`, the built-in default palette is used.
    pub palette: Option<PaletteSpec>,
    /// Single fill color for all categories as a hex string.
    /// Ignored when `palette` is set. Defaults to `"#4C72B0"`.
    pub color: Option<String>,
    /// Fill alpha (0.0 = transparent, 1.0 = opaque). Defaults to `0.65`.
    pub alpha: Option<f64>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
    /// Point count per category above which violin is used instead of sina.
    ///
    /// If the most-populated category has more than this many data points the
    /// renderer draws KDE violin polygons; otherwise it draws sina scatter.
    /// Defaults to `50` when `None`.
    pub point_threshold: Option<u32>,
}

/// Builder for [`DensityConfig`].
///
/// All three core fields are required. Calling
/// [`build`](DensityConfigBuilder::build) without setting any of them returns
/// [`ChartError::MissingField`].
pub struct DensityConfigBuilder {
    category_col: Option<String>,
    value_col: Option<String>,
    y_label: Option<String>,
    palette: Option<PaletteSpec>,
    color: Option<String>,
    alpha: Option<f64>,
    y_axis: Option<AxisConfig>,
    point_threshold: Option<u32>,
}

impl DensityConfig {
    /// Create a new builder for a density plot configuration.
    #[must_use]
    pub fn builder() -> DensityConfigBuilder {
        DensityConfigBuilder {
            category_col: None,
            value_col: None,
            y_label: None,
            palette: None,
            color: None,
            alpha: None,
            y_axis: None,
            point_threshold: None,
        }
    }
}

impl DensityConfigBuilder {
    /// Set the category column name (X axis labels / grouping key).
    #[must_use]
    pub fn category(mut self, col: &str) -> Self {
        self.category_col = Some(col.into());
        self
    }

    /// Set the numeric value column name (Y axis values).
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

    /// Set a color palette — one distinct color per category.
    ///
    /// Accepts any [`PaletteSpec`]: a named Bokeh palette (e.g. `"Set2"`) or
    /// a custom list of hex strings. When set, the `color` field is ignored.
    #[must_use]
    pub fn palette(mut self, palette: PaletteSpec) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a single fill color for all categories as a hex string.
    /// Ignored when `palette` is set.
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the fill alpha (0.0 = transparent, 1.0 = opaque).
    #[must_use]
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(alpha);
        self
    }

    /// Configure the Y axis appearance.
    #[must_use]
    pub fn y_axis(mut self, axis: AxisConfig) -> Self {
        self.y_axis = Some(axis);
        self
    }

    /// Set the point-count threshold that controls automatic mode selection.
    ///
    /// When the most-populated category has more than `n` data points the
    /// renderer uses a **violin plot**; otherwise it uses a **sina plot**.
    /// Defaults to `50` when not set.
    #[must_use]
    pub fn point_threshold(mut self, n: u32) -> Self {
        self.point_threshold = Some(n);
        self
    }

    /// Build the config, returning an error if any required field is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if any required field was not set.
    pub fn build(self) -> Result<DensityConfig, ChartError> {
        Ok(DensityConfig {
            category_col: self.category_col.ok_or(ChartError::MissingField("category_col"))?,
            value_col:    self.value_col.ok_or(ChartError::MissingField("value_col"))?,
            y_label:      self.y_label.ok_or(ChartError::MissingField("y_label"))?,
            palette:         self.palette,
            color:           self.color,
            alpha:           self.alpha,
            y_axis:          self.y_axis,
            point_threshold: self.point_threshold,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::customization::axis::AxisConfig;
    use crate::charts::customization::palette::PaletteSpec;

    fn minimal() -> DensityConfig {
        DensityConfig::builder()
            .category("department")
            .value("salary_k")
            .y_label("Salary (k USD)")
            .build()
            .unwrap()
    }

    // ── Required field validation ─────────────────────────────────────────────

    #[test]
    fn missing_category_col() {
        assert!(matches!(
            DensityConfig::builder()
                .value("salary_k")
                .y_label("Salary")
                .build(),
            Err(ChartError::MissingField("category_col"))
        ));
    }

    #[test]
    fn missing_value_col() {
        assert!(matches!(
            DensityConfig::builder()
                .category("department")
                .y_label("Salary")
                .build(),
            Err(ChartError::MissingField("value_col"))
        ));
    }

    #[test]
    fn missing_y_label() {
        assert!(matches!(
            DensityConfig::builder()
                .category("department")
                .value("salary_k")
                .build(),
            Err(ChartError::MissingField("y_label"))
        ));
    }

    // ── Build success ─────────────────────────────────────────────────────────

    #[test]
    fn build_success() {
        let cfg = minimal();
        assert_eq!(cfg.category_col, "department");
        assert_eq!(cfg.value_col, "salary_k");
        assert_eq!(cfg.y_label, "Salary (k USD)");
    }

    // ── Optional fields default to None ──────────────────────────────────────

    #[test]
    fn optional_fields_default_none() {
        let cfg = minimal();
        assert!(cfg.palette.is_none());
        assert!(cfg.color.is_none());
        assert!(cfg.alpha.is_none());
        assert!(cfg.y_axis.is_none());
        assert!(cfg.point_threshold.is_none());
    }

    // ── Optional field setters ────────────────────────────────────────────────

    #[test]
    fn with_palette() {
        let cfg = DensityConfig::builder()
            .category("dept").value("val").y_label("Y")
            .palette(PaletteSpec::Named("Set2".into()))
            .build()
            .unwrap();
        assert!(matches!(cfg.palette, Some(PaletteSpec::Named(_))));
    }

    #[test]
    fn with_custom_palette() {
        let cfg = DensityConfig::builder()
            .category("dept").value("val").y_label("Y")
            .palette(PaletteSpec::Custom(vec!["#ff0000".into(), "#00ff00".into()]))
            .build()
            .unwrap();
        assert!(matches!(cfg.palette, Some(PaletteSpec::Custom(_))));
    }

    #[test]
    fn with_color() {
        let cfg = DensityConfig::builder()
            .category("dept").value("val").y_label("Y")
            .color("#e74c3c")
            .build()
            .unwrap();
        assert_eq!(cfg.color.as_deref(), Some("#e74c3c"));
    }

    #[test]
    fn with_alpha() {
        let cfg = DensityConfig::builder()
            .category("dept").value("val").y_label("Y")
            .alpha(0.5)
            .build()
            .unwrap();
        assert_eq!(cfg.alpha, Some(0.5));
    }

    #[test]
    fn with_y_axis() {
        let ax = AxisConfig::builder().tick_format("0.0").show_grid(false).build();
        let cfg = DensityConfig::builder()
            .category("dept").value("val").y_label("Y")
            .y_axis(ax)
            .build()
            .unwrap();
        let y = cfg.y_axis.as_ref().unwrap();
        assert_eq!(y.tick_format.as_deref(), Some("0.0"));
        assert!(!y.show_grid);
    }

    #[test]
    fn with_point_threshold() {
        let cfg = DensityConfig::builder()
            .category("dept").value("val").y_label("Y")
            .point_threshold(50)
            .build()
            .unwrap();
        assert_eq!(cfg.point_threshold, Some(50));
    }

    #[test]
    fn with_all_optional_fields() {
        let cfg = DensityConfig::builder()
            .category("dept").value("val").y_label("Y")
            .palette(PaletteSpec::Named("Category10".into()))
            .alpha(0.7)
            .point_threshold(20)
            .build()
            .unwrap();
        assert!(cfg.palette.is_some());
        assert_eq!(cfg.alpha, Some(0.7));
        assert_eq!(cfg.point_threshold, Some(20));
    }
}
