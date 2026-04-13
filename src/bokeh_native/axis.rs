//! Axis builder — creates Bokeh axis models with tickers, formatters, and grids.

use crate::charts::{AxisConfig, TimeScale};

use super::id_gen::IdGen;
use super::model::{BokehObject, BokehValue};

/// Scale type for a chart axis.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AxisType {
    Categorical,
    Linear,
    Datetime,
}

/// Builder for a Bokeh axis, ticker, formatter, and grid.
pub struct AxisBuilder<'a> {
    axis_type: AxisType,
    cfg: Option<&'a AxisConfig>,
    dimension: u8, // 0 = x (below), 1 = y (left)
}

impl<'a> AxisBuilder<'a> {
    /// Create an x-axis builder (grid dimension 0).
    pub fn x(axis_type: AxisType) -> Self {
        Self { axis_type, cfg: None, dimension: 0 }
    }

    /// Create a y-axis builder (grid dimension 1).
    pub fn y(axis_type: AxisType) -> Self {
        Self { axis_type, cfg: None, dimension: 1 }
    }

    /// Attach an optional `AxisConfig` for range, tick format, and label options.
    pub fn config(mut self, cfg: Option<&'a AxisConfig>) -> Self {
        self.cfg = cfg;
        self
    }

    /// The attached `AxisConfig`, if any.
    pub fn cfg(&self) -> Option<&AxisConfig> {
        self.cfg
    }

    /// The Bokeh scale model name for this axis type.
    pub fn scale_name(&self) -> &'static str {
        if self.axis_type == AxisType::Categorical {
            "CategoricalScale"
        } else {
            "LinearScale"
        }
    }

    /// Build the axis, ticker, formatter, and grid.
    ///
    /// Returns `(axis_obj, axis_id, grid_obj, grid_id)`.
    pub fn build(self, id_gen: &mut IdGen) -> (BokehObject, String, BokehObject, String) {
        let (axis_name, ticker_name, formatter_name) = match self.axis_type {
            AxisType::Categorical => (
                "CategoricalAxis",
                "CategoricalTicker",
                "CategoricalTickFormatter",
            ),
            AxisType::Datetime => (
                "DatetimeAxis",
                "DatetimeTicker",
                "DatetimeTickFormatter",
            ),
            AxisType::Linear => ("LinearAxis", "BasicTicker", "BasicTickFormatter"),
        };

        let ticker_id = id_gen.next();
        let fmt_id = id_gen.next();
        let axis_id = id_gen.next();
        let grid_id = id_gen.next();

        let ticker = match self.axis_type {
            AxisType::Linear => BokehObject::new(ticker_name, ticker_id.clone())
                .attr("mantissas", BokehValue::Array(vec![
                    BokehValue::Int(1),
                    BokehValue::Int(2),
                    BokehValue::Int(5),
                ])),
            _ => BokehObject::new(ticker_name, ticker_id.clone()),
        };

        let mut formatter = BokehObject::new(formatter_name, fmt_id.clone());
        if let Some(cfg) = self.cfg {
            formatter = apply_formatter_config(id_gen, formatter, cfg);
        }

        let mut axis = BokehObject::new(axis_name, axis_id.clone())
            .attr("ticker", ticker.into_value())
            .attr("formatter", formatter.into_value());

        if let Some(cfg) = self.cfg {
            axis = apply_axis_visual_config(axis, cfg);
        }

        let grid = BokehObject::new("Grid", grid_id.clone())
            .attr("axis", BokehValue::ref_of(&axis_id))
            .attr("dimension", BokehValue::Int(self.dimension as i64));

        (axis, axis_id, grid, grid_id)
    }
}

fn apply_formatter_config(
    id_gen: &mut IdGen,
    formatter: BokehObject,
    cfg: &AxisConfig,
) -> BokehObject {
    if let Some(ts) = &cfg.time_scale {
        let fmt_str = time_scale_to_fmt(ts);
        return BokehObject::new("DatetimeTickFormatter", id_gen.next())
            .attr("milliseconds", BokehValue::Str(fmt_str.clone()))
            .attr("seconds",      BokehValue::Str(fmt_str.clone()))
            .attr("minsec",       BokehValue::Str(fmt_str.clone()))
            .attr("minutes",      BokehValue::Str(fmt_str.clone()))
            .attr("hourmin",      BokehValue::Str(fmt_str.clone()))
            .attr("hours",        BokehValue::Str(fmt_str.clone()))
            .attr("days",         BokehValue::Str(fmt_str.clone()))
            .attr("months",       BokehValue::Str(fmt_str.clone()))
            .attr("years",        BokehValue::Str(fmt_str));
    }
    if let Some(fmt) = &cfg.tick_format {
        return BokehObject::new("NumeralTickFormatter", id_gen.next())
            .attr("format", BokehValue::Str(fmt.clone()));
    }
    formatter
}

fn apply_axis_visual_config(mut axis: BokehObject, cfg: &AxisConfig) -> BokehObject {
    if let Some(rot) = cfg.label_rotation {
        let radians = rot * std::f64::consts::PI / 180.0;
        axis = axis.attr("major_label_orientation", BokehValue::Float(radians));
    }
    axis
}

fn time_scale_to_fmt(ts: &TimeScale) -> String {
    match ts {
        TimeScale::Milliseconds => "%H:%M:%S.%3N".into(),
        TimeScale::Seconds      => "%H:%M:%S".into(),
        TimeScale::Minutes      => "%H:%M".into(),
        TimeScale::Hours        => "%m/%d %H:%M".into(),
        TimeScale::Days         => "%Y-%m-%d".into(),
        TimeScale::Months       => "%b %Y".into(),
        TimeScale::Years        => "%Y".into(),
    }
}
