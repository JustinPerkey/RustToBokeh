use crate::error::ChartError;
use crate::charts::customization::palette::PaletteSpec;
use crate::charts::customization::tooltip::TooltipSpec;

/// Configuration for a pie or donut chart.
///
/// Pie charts display part-to-whole relationships as wedge slices. Setting
/// [`inner_radius`](PieConfig::inner_radius) converts the pie into a donut
/// chart by cutting out a circular hole in the centre.
///
/// # Example — Pie
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = PieConfig::builder()
///     .label("category")
///     .value("amount")
///     .build()?;
/// ```
///
/// # Example — Donut
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let config = PieConfig::builder()
///     .label("category")
///     .value("amount")
///     .inner_radius(0.45)
///     .build()?;
/// ```
pub struct PieConfig {
    /// Column name whose values are used as slice labels / legend entries.
    pub label_col: String,
    /// Column name containing the numeric values for each slice.
    pub value_col: String,
    /// Inner radius of the donut hole in plot units (0.0–0.9).
    ///
    /// When `None` the chart is a solid pie. When `Some(r)` the chart is a
    /// donut with the hole radius `r`; the outer radius is always `0.9`.
    /// A value of `0.45` gives a typical half-width donut.
    pub inner_radius: Option<f64>,
    /// Colour palette for the slices.  Defaults to the built-in ten-colour
    /// seaborn palette when `None`.
    pub palette: Option<PaletteSpec>,
    /// Custom hover tooltip.  Defaults to showing the label and raw value
    /// when `None`.
    pub tooltips: Option<TooltipSpec>,
    /// Whether to show the legend.  Defaults to `true` when `None`.
    pub show_legend: Option<bool>,
}

/// Builder for [`PieConfig`].
///
/// Call [`PieConfig::builder`] to obtain one.  At minimum [`label`](Self::label)
/// and [`value`](Self::value) must be set before calling [`build`](Self::build).
pub struct PieConfigBuilder {
    label_col: Option<String>,
    value_col: Option<String>,
    inner_radius: Option<f64>,
    palette: Option<PaletteSpec>,
    tooltips: Option<TooltipSpec>,
    show_legend: Option<bool>,
}

impl PieConfig {
    /// Create a new builder for a pie / donut chart configuration.
    #[must_use]
    pub fn builder() -> PieConfigBuilder {
        PieConfigBuilder {
            label_col: None,
            value_col: None,
            inner_radius: None,
            palette: None,
            tooltips: None,
            show_legend: None,
        }
    }
}

impl PieConfigBuilder {
    /// Set the column used for slice labels (category names).
    #[must_use]
    pub fn label(mut self, col: &str) -> Self {
        self.label_col = Some(col.into());
        self
    }

    /// Set the column containing numeric values for each slice.
    #[must_use]
    pub fn value(mut self, col: &str) -> Self {
        self.value_col = Some(col.into());
        self
    }

    /// Set the inner radius to render a donut chart.
    ///
    /// `radius` is in plot units; the outer radius is always `0.9`, so a
    /// value of `0.45` gives a typical half-width donut hole.
    #[must_use]
    pub fn inner_radius(mut self, radius: f64) -> Self {
        self.inner_radius = Some(radius);
        self
    }

    /// Set a colour palette for the slices.
    #[must_use]
    pub fn palette(mut self, palette: PaletteSpec) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Set a custom hover tooltip.
    #[must_use]
    pub fn tooltips(mut self, tooltips: TooltipSpec) -> Self {
        self.tooltips = Some(tooltips);
        self
    }

    /// Show or hide the legend.  Defaults to `true` when not called.
    #[must_use]
    pub fn show_legend(mut self, show: bool) -> Self {
        self.show_legend = Some(show);
        self
    }

    /// Build the config, returning an error if any required field is missing.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::MissingField`] if `label` or `value` was not set.
    pub fn build(self) -> Result<PieConfig, ChartError> {
        Ok(PieConfig {
            label_col: self.label_col.ok_or(ChartError::MissingField("label_col"))?,
            value_col: self.value_col.ok_or(ChartError::MissingField("value_col"))?,
            inner_radius: self.inner_radius,
            palette: self.palette,
            tooltips: self.tooltips,
            show_legend: self.show_legend,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::customization::tooltip::{TooltipFormat, TooltipSpec};

    // ── Required fields ───────────────────────────────────────────────────────

    #[test]
    fn pie_missing_label_col() {
        assert!(matches!(
            PieConfig::builder().value("amount").build(),
            Err(ChartError::MissingField("label_col"))
        ));
    }

    #[test]
    fn pie_missing_value_col() {
        assert!(matches!(
            PieConfig::builder().label("category").build(),
            Err(ChartError::MissingField("value_col"))
        ));
    }

    #[test]
    fn pie_build_success() {
        let cfg = PieConfig::builder()
            .label("company")
            .value("share")
            .build()
            .unwrap();
        assert_eq!(cfg.label_col, "company");
        assert_eq!(cfg.value_col, "share");
    }

    // ── Optional fields ───────────────────────────────────────────────────────

    #[test]
    fn pie_optional_fields_default_none() {
        let cfg = PieConfig::builder()
            .label("l")
            .value("v")
            .build()
            .unwrap();
        assert!(cfg.inner_radius.is_none());
        assert!(cfg.palette.is_none());
        assert!(cfg.tooltips.is_none());
        assert!(cfg.show_legend.is_none());
    }

    #[test]
    fn pie_with_inner_radius() {
        let cfg = PieConfig::builder()
            .label("l")
            .value("v")
            .inner_radius(0.45)
            .build()
            .unwrap();
        assert_eq!(cfg.inner_radius, Some(0.45));
    }

    #[test]
    fn pie_with_named_palette() {
        let cfg = PieConfig::builder()
            .label("l")
            .value("v")
            .palette(PaletteSpec::Named("Viridis".into()))
            .build()
            .unwrap();
        assert!(matches!(cfg.palette, Some(PaletteSpec::Named(_))));
    }

    #[test]
    fn pie_with_custom_palette() {
        let colors = vec!["#ff0000".into(), "#00ff00".into()];
        let cfg = PieConfig::builder()
            .label("l")
            .value("v")
            .palette(PaletteSpec::Custom(colors))
            .build()
            .unwrap();
        assert!(matches!(cfg.palette, Some(PaletteSpec::Custom(_))));
    }

    #[test]
    fn pie_with_tooltips() {
        let tt = TooltipSpec::builder()
            .field("share", "Share", TooltipFormat::Percent(Some(1)))
            .build();
        let cfg = PieConfig::builder()
            .label("l")
            .value("v")
            .tooltips(tt)
            .build()
            .unwrap();
        let fields = &cfg.tooltips.as_ref().unwrap().fields;
        assert_eq!(fields.len(), 1);
        assert!(matches!(fields[0].format, TooltipFormat::Percent(Some(1))));
    }

    #[test]
    fn pie_with_show_legend_false() {
        let cfg = PieConfig::builder()
            .label("l")
            .value("v")
            .show_legend(false)
            .build()
            .unwrap();
        assert_eq!(cfg.show_legend, Some(false));
    }

    #[test]
    fn pie_with_show_legend_true() {
        let cfg = PieConfig::builder()
            .label("l")
            .value("v")
            .show_legend(true)
            .build()
            .unwrap();
        assert_eq!(cfg.show_legend, Some(true));
    }
}
