//! Range filter — RangeSlider driving a BooleanFilter.

use crate::charts::FilterSpec;
use crate::error::ChartError;

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::FilterOutput;

pub(super) fn build_range_filter(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    n: usize,
    min: f64,
    max: f64,
    step: f64,
) -> Result<FilterOutput, ChartError> {
    let bf_id = id_gen.next();
    let bf = BokehObject::new("BooleanFilter", bf_id.clone())
        .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); n]));

    let cds_placeholder_id = format!("__cds_{}", &filter.source_key);

    let cb_id = id_gen.next();
    let callback = BokehObject::new("CustomJS", cb_id)
        .attr("args", BokehValue::Map(vec![
            ("bf".into(), BokehValue::ref_of(&bf_id)),
            ("source".into(), BokehValue::Ref(cds_placeholder_id)),
            ("col".into(), BokehValue::Str(filter.column.clone())),
        ]))
        .attr("code", BokehValue::Str(format!(
            "const [lo, hi] = cb_obj.value;\
             const data = source.data['{}'];\
             bf.booleans = data.map(v => v >= lo && v <= hi);\
             source.change.emit();",
            filter.column
        )));

    let slider_id = id_gen.next();
    let slider = BokehObject::new("RangeSlider", slider_id)
        .attr("title", BokehValue::Str(filter.label.clone()))
        .attr("start", BokehValue::Float(min))
        .attr("end", BokehValue::Float(max))
        .attr("value", BokehValue::Array(vec![BokehValue::Float(min), BokehValue::Float(max)]))
        .attr("step", BokehValue::Float(step))
        .attr("sizing_mode", BokehValue::Str("stretch_width".into()))
        .attr("js_property_callbacks", BokehValue::Map(vec![
            ("change:value".into(), BokehValue::Array(vec![callback.into_value()])),
        ]));

    Ok(FilterOutput {
        widget: slider,
        filter_id: bf_id,
        filter_obj: bf,
        source_key: filter.source_key.clone(),
        switch_label: None,
        is_range_tool: false,
        range_tool_range_id: None,
        range_tool_overview: None,
    })
}
