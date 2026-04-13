//! Scatter chart builder.

use polars::prelude::DataFrame;

use crate::charts::charts::ScatterConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::source::build_column_data_source;
use super::{add_renderers, make_hover_tool, set_axis_labels};

pub fn build_scatter(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &ScatterConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
    range_tool_x_range_id: Option<&str>,
) -> Result<BokehObject, ChartError> {
    let ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &[cfg.x_col.as_str(), cfg.y_col.as_str()],
    );

    // Detect datetime x-axis from axis config
    let is_datetime = cfg.x_axis.as_ref()
        .and_then(|a| a.time_scale.as_ref())
        .is_some();

    let x_range = if let Some(rt_id) = range_tool_x_range_id {
        XRangeKind::ExistingId(rt_id.to_string())
    } else if is_datetime {
        XRangeKind::DataRange
    } else {
        XRangeKind::DataRange
    };

    let x_axis_type = if is_datetime { AxisType::Datetime } else { AxisType::Linear };

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        x_range,
        YRangeKind::DataRange,
        AxisBuilder::x(x_axis_type).config(cfg.x_axis.as_ref()),
        AxisBuilder::y(AxisType::Linear).config(cfg.y_axis.as_ref()),
        Some(ht),
    );

    let color = cfg.color.as_deref().unwrap_or("#4C72B0");
    let marker = cfg.marker.as_deref().unwrap_or("circle");
    let size = cfg.marker_size.unwrap_or(10.0);
    let alpha = cfg.alpha.unwrap_or(0.7);

    let cds = build_column_data_source(id_gen, df);
    let cds_ref = cds.into_value();

    let glyph_id = id_gen.next();
    let glyph = BokehObject::new("Scatter", glyph_id)
        .attr("x", BokehValue::field(&cfg.x_col))
        .attr("y", BokehValue::field(&cfg.y_col))
        .attr("size", BokehValue::value_of(BokehValue::Float(size)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(alpha)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
        .attr("marker", BokehValue::value_of(BokehValue::Str(marker.to_string())));

    let nonsel_id = id_gen.next();
    let nonsel = BokehObject::new("Scatter", nonsel_id)
        .attr("x", BokehValue::field(&cfg.x_col))
        .attr("y", BokehValue::field(&cfg.y_col))
        .attr("size", BokehValue::value_of(BokehValue::Float(size)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
        .attr("marker", BokehValue::value_of(BokehValue::Str(marker.to_string())));

    let renderer = build_glyph_renderer(id_gen, cds_ref, glyph, Some(nonsel), filter_ref);
    add_renderers(&mut figure, vec![renderer]);
    set_axis_labels(&mut figure, &cfg.x_label, &cfg.y_label);

    Ok(figure)
}
