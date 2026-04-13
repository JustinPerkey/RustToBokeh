//! Multi-line chart builder.

use polars::prelude::*;

use crate::charts::charts::line::LineConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::palette::resolve_palette;
use super::super::source::build_column_data_source;
use super::{add_legend, add_renderers, make_hover_tool, set_axis_labels};

pub fn build_line(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &LineConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
    range_tool_x_range_id: Option<&str>,
) -> Result<BokehObject, ChartError> {
    let colors = resolve_palette(cfg.palette.as_ref(), cfg.y_cols.len());
    let line_width = cfg.line_width.unwrap_or(2.5);
    let point_size = cfg.point_size.unwrap_or(7.0);

    // Detect datetime x-axis
    let is_datetime = cfg.x_axis.as_ref()
        .and_then(|a| a.time_scale.as_ref())
        .is_some();

    // Detect categorical (string) x-column
    let x_col_dtype = df.column(&cfg.x_col)
        .map(|c| c.dtype().clone())
        .unwrap_or(DataType::Float64);
    let is_categorical = matches!(
        x_col_dtype,
        DataType::String | DataType::Categorical(_, _) | DataType::Enum(_, _)
    );

    let x_range = if let Some(rt_id) = range_tool_x_range_id {
        XRangeKind::ExistingId(rt_id.to_string())
    } else if is_categorical {
        // String x-data needs FactorRange for BokehJS to position points
        let x_series = df.column(&cfg.x_col).unwrap();
        let x_cast = x_series.cast(&DataType::String).unwrap();
        let factors: Vec<BokehValue> = x_cast.str().unwrap()
            .into_iter()
            .map(|v| BokehValue::Str(v.unwrap_or("").to_string()))
            .collect();
        XRangeKind::Factor(factors)
    } else {
        XRangeKind::DataRange
    };

    let x_axis_type = if is_datetime {
        AxisType::Datetime
    } else if is_categorical {
        AxisType::Categorical
    } else {
        AxisType::Linear
    };

    let mut default_cols: Vec<&str> = vec![cfg.x_col.as_str()];
    default_cols.extend(cfg.y_cols.iter().map(|s| s.as_str()));

    let ht = make_hover_tool(id_gen, cfg.tooltips.as_ref(), &default_cols);

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

    let cds = build_column_data_source(id_gen, df);
    let cds_id = cds.id.clone();
    // We embed the CDS inline for the first renderer, then reference by ID for others
    let cds_value = cds.into_value();

    let mut legend_items: Vec<BokehValue> = Vec::new();
    let mut first_renderer = true;

    for (i, y_col) in cfg.y_cols.iter().enumerate() {
        let color = &colors[i];

        // Line glyph
        let line_glyph_id = id_gen.next();
        let line_glyph = BokehObject::new("Line", line_glyph_id)
            .attr("x", BokehValue::field(&cfg.x_col))
            .attr("y", BokehValue::field(y_col))
            .attr("line_color", BokehValue::value_of(BokehValue::Str(color.clone())))
            .attr("line_width", BokehValue::value_of(BokehValue::Float(line_width)));

        let line_nonsel_id = id_gen.next();
        let line_nonsel = BokehObject::new("Line", line_nonsel_id)
            .attr("x", BokehValue::field(&cfg.x_col))
            .attr("y", BokehValue::field(y_col))
            .attr("line_color", BokehValue::value_of(BokehValue::Str(color.clone())))
            .attr("line_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
            .attr("line_width", BokehValue::value_of(BokehValue::Float(line_width)));

        let cds_ref = if first_renderer {
            cds_value.clone()
        } else {
            BokehValue::ref_of(&cds_id)
        };

        let line_renderer = build_glyph_renderer(
            id_gen,
            cds_ref,
            line_glyph,
            Some(line_nonsel),
            filter_ref.clone(),
        );
        let line_renderer_id = line_renderer.id.clone();

        // Circle marker glyph
        let circle_glyph_id = id_gen.next();
        let circle_glyph = BokehObject::new("Scatter", circle_glyph_id)
            .attr("x", BokehValue::field(&cfg.x_col))
            .attr("y", BokehValue::field(y_col))
            .attr("size", BokehValue::value_of(BokehValue::Float(point_size)))
            .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.clone())))
            .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
            .attr("marker", BokehValue::value_of(BokehValue::Str("circle".to_string())));

        let circle_nonsel_id = id_gen.next();
        let circle_nonsel = BokehObject::new("Scatter", circle_nonsel_id)
            .attr("x", BokehValue::field(&cfg.x_col))
            .attr("y", BokehValue::field(y_col))
            .attr("size", BokehValue::value_of(BokehValue::Float(point_size)))
            .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.clone())))
            .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
            .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
            .attr("marker", BokehValue::value_of(BokehValue::Str("circle".to_string())));

        let circle_cds_ref = BokehValue::ref_of(&cds_id);
        let circle_renderer = build_glyph_renderer(
            id_gen,
            circle_cds_ref,
            circle_glyph,
            Some(circle_nonsel),
            filter_ref.clone(),
        );

        // Legend item for this series
        let legend_item_id = id_gen.next();
        let legend_item = BokehObject::new("LegendItem", legend_item_id)
            .attr("label", BokehValue::value_of(BokehValue::Str(y_col.clone())))
            .attr(
                "renderers",
                BokehValue::Array(vec![BokehValue::ref_of(&line_renderer_id)]),
            );
        legend_items.push(legend_item.into_value());

        add_renderers(&mut figure, vec![line_renderer, circle_renderer]);
        first_renderer = false;
    }

    // Build legend
    let legend_id = id_gen.next();
    let legend = BokehObject::new("Legend", legend_id)
        .attr("items", BokehValue::Array(legend_items))
        .attr("location", BokehValue::Str("top_right".into()))
        .attr("click_policy", BokehValue::Str("hide".into()));
    add_legend(&mut figure, legend);

    set_axis_labels(&mut figure, "", &cfg.y_label);
    Ok(figure)
}
