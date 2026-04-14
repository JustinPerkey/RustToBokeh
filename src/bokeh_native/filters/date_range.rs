//! DateRange filter — DatetimeRangeSlider driving a BooleanFilter on epoch-ms column.

use crate::charts::FilterSpec;
use crate::error::ChartError;

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::FilterOutput;

pub(super) fn build_date_range_filter(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    n: usize,
    min_ms: f64,
    max_ms: f64,
    step_ms: f64,
) -> Result<FilterOutput, ChartError> {
    let bf_id = id_gen.next();
    let bf = BokehObject::new("BooleanFilter", bf_id.clone())
        .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); n]));

    let cds_placeholder_id = format!("__cds_{}", &filter.source_key);

    let cb_id = id_gen.next();
    let callback = BokehObject::new("CustomJS", cb_id)
        .attr("args", BokehValue::Map(vec![
            ("bf".into(), bf.clone().into_value()),
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

    let widget_id = id_gen.next();
    let widget = BokehObject::new("DatetimeRangeSlider", widget_id)
        .attr("title", BokehValue::Str(filter.label.clone()))
        .attr("start", BokehValue::Float(min_ms))
        .attr("end", BokehValue::Float(max_ms))
        .attr("value", BokehValue::Array(vec![BokehValue::Float(min_ms), BokehValue::Float(max_ms)]))
        .attr("step", BokehValue::Float(step_ms))
        .attr("sizing_mode", BokehValue::Str("stretch_width".into()))
        .attr("js_property_callbacks", BokehValue::Map(vec![
            ("change:value".into(), BokehValue::Array(vec![callback.into_value()])),
        ]));

    Ok(FilterOutput {
        widget,
        filter_id: bf_id,
        filter_obj: bf,
        source_key: filter.source_key.clone(),
        switch_label: None,
        is_range_tool: false,
        range_tool_range_id: None,
        range_tool_overview: None,
    })
}
