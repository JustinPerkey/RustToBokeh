//! Threshold filter — Switch toggle driving a BooleanFilter.

use crate::charts::FilterSpec;
use crate::error::ChartError;

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::FilterOutput;

pub(super) fn build_threshold_filter(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    n: usize,
    value: f64,
    above: bool,
) -> Result<FilterOutput, ChartError> {
    let bf_id = id_gen.next();
    let bf = BokehObject::new("BooleanFilter", bf_id.clone())
        .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); n]));

    let cds_placeholder_id = format!("__cds_{}", &filter.source_key);
    let above_str = if above { "true" } else { "false" };

    let cb_id = id_gen.next();
    let callback = BokehObject::new("CustomJS", cb_id)
        .attr("args", BokehValue::Map(vec![
            ("bf".into(), bf.clone().into_value()),
            ("source".into(), BokehValue::Ref(cds_placeholder_id)),
            ("col".into(), BokehValue::Str(filter.column.clone())),
            ("threshold".into(), BokehValue::Float(value)),
            ("above".into(), BokehValue::Bool(above)),
        ]))
        .attr("code", BokehValue::Str(format!(
            "const data = source.data['{}'];\
             if (cb_obj.active) {{\
                 bf.booleans = data.map(v => {} ? v >= threshold : v <= threshold);\
             }} else {{\
                 bf.booleans = data.map(() => true);\
             }}\
             source.change.emit();",
            filter.column, above_str
        )));

    let widget_id = id_gen.next();
    let widget = BokehObject::new("Switch", widget_id)
        .attr("active", BokehValue::Bool(false))
        .attr("js_property_callbacks", BokehValue::Map(vec![
            ("change:active".into(), BokehValue::Array(vec![callback.into_value()])),
        ]));

    Ok(FilterOutput {
        widget,
        filter_id: bf_id,
        filter_obj: bf,
        source_key: filter.source_key.clone(),
        switch_label: Some(filter.label.clone()),
        is_range_tool: false,
        range_tool_range_id: None,
        range_tool_overview: None,
    })
}
