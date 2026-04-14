//! RangeTool filter — overview chart with draggable range selector that syncs
//! the shared x-axis Range1d of the page's detail charts.

use polars::prelude::DataFrame;

use crate::charts::{FilterConfig, FilterSpec};
use crate::error::ChartError;

use super::super::charts::add_renderers;
use super::super::figure::{
    build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind,
};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::source::build_column_data_source;
use super::FilterOutput;

pub(super) fn build_range_tool(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    df: &DataFrame,
) -> Result<FilterOutput, ChartError> {
    let (start, end, y_col, time_scale) = match &filter.config {
        FilterConfig::RangeTool { start, end, y_column, time_scale } => {
            (*start, *end, y_column.clone(), time_scale.clone())
        }
        _ => unreachable!(),
    };

    // Shared Range1d for x-axis synchronisation (its ID is used for chart linking)
    let range_id = id_gen.next();

    // BooleanFilter driven by the Range1d (for .filtered() charts)
    let n = df.height();
    let bf_id = id_gen.next();
    let bf = BokehObject::new("BooleanFilter", bf_id.clone())
        .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); n]));

    let cds_placeholder_id = format!("__cds_{}", &filter.source_key);

    let range_cb_code = format!(
        "const lo = cb_obj.start;\
         const hi = cb_obj.end;\
         const data = source.data['{}'];\
         bf.booleans = data.map(v => v >= lo && v <= hi);\
         source.change.emit();",
        filter.column
    );

    let start_cb_id = id_gen.next();
    let start_cb = BokehObject::new("CustomJS", start_cb_id)
        .attr("args", BokehValue::Map(vec![
            ("bf".into(), bf.clone().into_value()),
            ("source".into(), BokehValue::Ref(cds_placeholder_id.clone())),
            ("col".into(), BokehValue::Str(filter.column.clone())),
        ]))
        .attr("code", BokehValue::Str(range_cb_code.clone()));

    let end_cb_id = id_gen.next();
    let end_cb = BokehObject::new("CustomJS", end_cb_id)
        .attr("args", BokehValue::Map(vec![
            ("bf".into(), BokehValue::ref_of(&bf_id)),
            ("source".into(), BokehValue::Ref(cds_placeholder_id)),
            ("col".into(), BokehValue::Str(filter.column.clone())),
        ]))
        .attr("code", BokehValue::Str(range_cb_code));

    let range_widget = BokehObject::new("Range1d", range_id.clone())
        .attr("start", BokehValue::Float(start))
        .attr("end", BokehValue::Float(end))
        .attr("js_property_callbacks", BokehValue::Map(vec![
            ("change:start".into(), BokehValue::Array(vec![start_cb.into_value()])),
            ("change:end".into(), BokehValue::Array(vec![end_cb.into_value()])),
        ]));

    // Overview figure
    let is_datetime = time_scale.is_some();
    let x_axis_type = if is_datetime { AxisType::Datetime } else { AxisType::Linear };

    let cds = build_column_data_source(id_gen, df);

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &filter.label,
        130,
        None,
        XRangeKind::DataRange,
        YRangeKind::DataRange,
        AxisBuilder::x(x_axis_type),
        AxisBuilder::y(AxisType::Linear),
        None,
    );

    let line_glyph_id = id_gen.next();
    let line_glyph = BokehObject::new("Line", line_glyph_id)
        .attr("x", BokehValue::field(&filter.column))
        .attr("y", BokehValue::field(&y_col))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("#4C72B0".into())))
        .attr("line_width", BokehValue::value_of(BokehValue::Float(1.5)));

    let line_nonsel_id = id_gen.next();
    let line_nonsel = BokehObject::new("Line", line_nonsel_id)
        .attr("x", BokehValue::field(&filter.column))
        .attr("y", BokehValue::field(&y_col))
        .attr("line_alpha", BokehValue::value_of(BokehValue::Float(0.1)));

    let renderer = build_glyph_renderer(id_gen, cds.into_value(), line_glyph, Some(line_nonsel), None);

    let range_tool_id = id_gen.next();
    let range_tool = BokehObject::new("RangeTool", range_tool_id)
        .attr("x_range", range_widget.to_ref())
        .attr("overlay", build_range_tool_overlay(id_gen).into_value());

    add_renderers(&mut figure, vec![renderer]);

    // Append RangeTool to toolbar.tools
    let mut range_tool_val = Some(range_tool.into_value());
    for (key, val) in &mut figure.attributes {
        if key == "toolbar" {
            if let BokehValue::Object(tb) = val {
                for (k, v) in &mut tb.attributes {
                    if k == "tools" {
                        if let BokehValue::Array(tools) = v {
                            if let Some(rt) = range_tool_val.take() {
                                tools.push(rt);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(FilterOutput {
        widget: range_widget,
        filter_id: bf_id,
        filter_obj: bf,
        source_key: filter.source_key.clone(),
        switch_label: None,
        is_range_tool: true,
        range_tool_range_id: Some(range_id),
        range_tool_overview: Some(figure),
    })
}

fn build_range_tool_overlay(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("BoxAnnotation", id_gen.next())
        .attr("fill_color", BokehValue::Str("navy".into()))
        .attr("fill_alpha", BokehValue::Float(0.2))
        .attr("line_color", BokehValue::Null)
        .attr("level", BokehValue::Str("underlay".into()))
}
