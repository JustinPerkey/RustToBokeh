use crate::charts::customization::axis::AxisConfig;
use crate::charts::customization::tooltip::TooltipSpec;
use crate::error::ChartError;

/// Controls which statistic is rendered from a pre-computed histogram DataFrame.
///
/// The DataFrame must be produced by [`compute_histogram`](crate::compute_histogram),
/// which generates `left`, `right`, `count`, `pdf`, and `cdf` columns.
///
/// | Variant | Renders | Glyph |
/// |---------|---------|-------|
/// | `Count` | Raw bin counts | Bars (quad) |
/// | `Pdf`   | Probability density (count / (n × bin_width)) | Bars (quad) |
/// | `Cdf`   | Cumulative fraction of values ≤ right edge | Step line |
#[derive(Clone, Debug, PartialEq)]
pub enum HistogramDisplay {
    /// Raw count of values in each bin (default).
    Count,
    /// Probability density: `count / (n × bin_width)`.
    Pdf,
    /// Cumulative distribution: fraction of values ≤ right edge of each bin.
    Cdf,
}

impl HistogramDisplay {
    /// Return the string identifier used by the Python renderer.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            HistogramDisplay::Count => "count",
            HistogramDisplay::Pdf => "pdf",
            HistogramDisplay::Cdf => "cdf",
        }
    }
}

/// Configuration for a histogram chart.
///
/// Histograms display the distribution of a numeric variable. The chart expects
/// a pre-computed histogram DataFrame produced by
/// [`compute_histogram`](crate::compute_histogram), which provides `left`,
/// `right`, `count`, `pdf`, and `cdf` columns. The [`display`](Self::display)
/// field selects which statistic to render.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
/// use polars::prelude::*;
///
/// // Prepare data:
/// let raw = df!["salary" => [42.0f64, 65.0, 80.0, 95.0]].unwrap();
/// let mut hist = compute_histogram(&raw, "salary", 12)?;
/// dash.add_df("salary_hist", &mut hist)?;
///
/// // Define chart:
/// let config = HistogramConfig::builder()
///     .x_label("Salary (k)")
///     .display(HistogramDisplay::Pdf)
///     .build()?;
/// ```
pub struct HistogramConfig {
    /// Label displayed on the X axis (the value axis).
    pub x_label: String,
    /// Which statistic to render. Defaults to [`HistogramDisplay::Count`] when `None`.
    pub display: Option<HistogramDisplay>,
    /// Label displayed on the Y axis. When `None`, a default is chosen based on
    /// the display mode (`"Count"`, `"Density"`, or `"Cumulative Fraction"`).
    pub y_label: Option<String>,
    /// Fill color for bars (count/pdf) or line color (cdf) as a hex string.
    /// Defaults to `"#4C72B0"` when `None`.
    pub color: Option<String>,
    /// Outline color for the bars. Defaults to `"white"` when `None`.
    /// Not used when `display` is `Cdf`.
    pub line_color: Option<String>,
    /// Fill alpha (0.0 = transparent, 1.0 = opaque). Defaults to `0.7` when `None`.
    pub alpha: Option<f64>,
    /// Custom hover tooltip. When `None`, a default is generated for the display mode.
    pub tooltips: Option<TooltipSpec>,
    /// X-axis display configuration.
    pub x_axis: Option<AxisConfig>,
    /// Y-axis display configuration.
    pub y_axis: Option<AxisConfig>,
}

/// Builder for [`HistogramConfig`].
///
/// The only required field is `x_label` (via [`x_label`](Self::x_label)).
/// Calling [`build`](Self::build) without setting it returns
/// [`ChartError::MissingField`].
pub struct HistogramConfigBuilder {
    x_label: Option<String>,
    display: Option<HistogramDisplay>,
    y_label: Option<String>,
    color: Option<String>,
    line_color: Option<String>,
    alpha: Option<f64>,
    tooltips: Option<TooltipSpec>,
    x_axis: Option<AxisConfig>,
    y_axis: Option<AxisConfig>,
}

impl HistogramConfig {
    /// Create a new builder for a histogram configuration.
    #[must_use]
    pub fn builder() -> HistogramConfigBuilder {
        HistogramConfigBuilder {
            x_label: None,
            display: None,
            y_label: None,
            color: None,
            line_color: None,
            alpha: None,
            tooltips: None,
            x_axis: None,
            y_axis: None,
        }
    }
}

impl HistogramConfigBuilder {
    /// Set the X-axis label text.
    #[must_use]
    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = Some(label.into());
        self
    }

    /// Set which statistic to render (default: [`HistogramDisplay::Count`]).
    #[must_use]
    pub fn display(mut self, mode: HistogramDisplay) -> Self {
        self.display = Some(mode);
        self
    }

    /// Set the Y-axis label text.
    #[must_use]
    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Set the fill/line color as a hex string (e.g. `"#2ecc71"`).
    #[must_use]
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set the bar outline color (not used for CDF display).
    #[must_use]
    pub fn line_color(mut self, color: &str) -> Self {
        self.line_color = Some(color.into());
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

    /// Build the config, returning an error if the required `x_label` is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if `x_label` was not set.
    pub fn build(self) -> Result<HistogramConfig, ChartError> {
        Ok(HistogramConfig {
            x_label: self.x_label.ok_or(ChartError::MissingField("x_label"))?,
            display: self.display,
            y_label: self.y_label,
            color: self.color,
            line_color: self.line_color,
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

    #[test]
    fn build_minimal() {
        let config = HistogramConfig::builder()
            .x_label("Salary (k)")
            .build()
            .unwrap();
        assert_eq!(config.x_label, "Salary (k)");
        assert!(config.display.is_none());
        assert!(config.y_label.is_none());
        assert!(config.color.is_none());
        assert!(config.line_color.is_none());
        assert!(config.alpha.is_none());
        assert!(config.tooltips.is_none());
        assert!(config.x_axis.is_none());
        assert!(config.y_axis.is_none());
    }

    #[test]
    fn build_with_display_modes() {
        for (mode, expected) in [
            (HistogramDisplay::Count, "count"),
            (HistogramDisplay::Pdf, "pdf"),
            (HistogramDisplay::Cdf, "cdf"),
        ] {
            let config = HistogramConfig::builder()
                .x_label("X")
                .display(mode.clone())
                .build()
                .unwrap();
            assert_eq!(config.display.unwrap().as_str(), expected);
        }
    }

    #[test]
    fn build_all_optional_fields() {
        let config = HistogramConfig::builder()
            .x_label("Age")
            .y_label("Frequency")
            .display(HistogramDisplay::Pdf)
            .color("#e74c3c")
            .line_color("#333333")
            .alpha(0.85)
            .build()
            .unwrap();
        assert_eq!(config.x_label, "Age");
        assert_eq!(config.y_label.as_deref(), Some("Frequency"));
        assert_eq!(config.color.as_deref(), Some("#e74c3c"));
        assert_eq!(config.line_color.as_deref(), Some("#333333"));
        assert_eq!(config.alpha, Some(0.85));
    }

    #[test]
    fn missing_x_label_returns_error() {
        assert!(matches!(
            HistogramConfig::builder().build(),
            Err(ChartError::MissingField("x_label"))
        ));
    }

    #[test]
    fn display_as_str() {
        assert_eq!(HistogramDisplay::Count.as_str(), "count");
        assert_eq!(HistogramDisplay::Pdf.as_str(), "pdf");
        assert_eq!(HistogramDisplay::Cdf.as_str(), "cdf");
    }
}
