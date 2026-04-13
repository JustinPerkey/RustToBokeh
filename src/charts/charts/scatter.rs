use crate::error::ChartError;
use crate::charts::customization::axis::AxisConfig;
use crate::charts::customization::marker::MarkerType;
use crate::charts::customization::tooltip::TooltipSpec;

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
    /// Marker shape.  Defaults to [`MarkerType::Circle`] when `None`.
    pub marker: Option<MarkerType>,
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
    marker: Option<MarkerType>,
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
    /// Set the marker shape.
    #[must_use]
    pub fn marker(mut self, marker: MarkerType) -> Self {
        self.marker = Some(marker);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::customization::axis::AxisConfig;
    use crate::charts::customization::marker::MarkerType;
    use crate::charts::customization::tooltip::{TooltipSpec, TooltipFormat};

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

    // ── ScatterConfig optional fields ─────────────────────────────────────────

    #[test]
    fn scatter_optional_fields_default_none() {
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .build().unwrap();
        assert!(cfg.color.is_none());
        assert!(cfg.marker.is_none());
        assert!(cfg.marker_size.is_none());
        assert!(cfg.alpha.is_none());
        assert!(cfg.tooltips.is_none());
        assert!(cfg.x_axis.is_none());
        assert!(cfg.y_axis.is_none());
    }

    #[test]
    fn scatter_with_color() {
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .color("#9b59b6")
            .build().unwrap();
        assert_eq!(cfg.color.as_deref(), Some("#9b59b6"));
    }

    #[test]
    fn scatter_with_marker() {
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .marker(MarkerType::Diamond)
            .build().unwrap();
        assert_eq!(cfg.marker, Some(MarkerType::Diamond));
    }

    #[test]
    fn scatter_with_marker_size() {
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .marker_size(14.0)
            .build().unwrap();
        assert_eq!(cfg.marker_size, Some(14.0));
    }

    #[test]
    fn scatter_with_alpha() {
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .alpha(0.5)
            .build().unwrap();
        assert_eq!(cfg.alpha, Some(0.5));
    }

    #[test]
    fn scatter_with_tooltips() {
        let tt = TooltipSpec::builder()
            .field("x", "X Axis", TooltipFormat::Currency)
            .field("y", "Y Axis", TooltipFormat::Percent(Some(1)))
            .build();
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .tooltips(tt)
            .build().unwrap();
        let fields = &cfg.tooltips.as_ref().unwrap().fields;
        assert_eq!(fields.len(), 2);
        assert!(matches!(fields[0].format, TooltipFormat::Currency));
        assert!(matches!(fields[1].format, TooltipFormat::Percent(Some(1))));
    }

    #[test]
    fn scatter_with_x_axis_range_and_bounds() {
        let ax = AxisConfig::builder()
            .range(0.0, 500.0)
            .bounds(0.0, 600.0)
            .build();
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .x_axis(ax)
            .build().unwrap();
        let x = cfg.x_axis.as_ref().unwrap();
        assert_eq!(x.start, Some(0.0));
        assert_eq!(x.end, Some(500.0));
        assert_eq!(x.bounds_min, Some(0.0));
        assert_eq!(x.bounds_max, Some(600.0));
    }

    #[test]
    fn scatter_with_y_axis() {
        let ax = AxisConfig::builder().tick_format("0.00").show_grid(false).build();
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .y_axis(ax)
            .build().unwrap();
        let y = cfg.y_axis.as_ref().unwrap();
        assert_eq!(y.tick_format.as_deref(), Some("0.00"));
        assert!(!y.show_grid);
    }
}
