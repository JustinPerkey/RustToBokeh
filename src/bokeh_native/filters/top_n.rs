//! Top-N filter — Slider driving an IndexFilter (top/bottom N rows).

use crate::charts::FilterSpec;
use crate::error::ChartError;

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::FilterOutput;

pub(super) fn build_top_n_filter(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    n: usize,
    max_n: usize,
    descending: bool,
) -> Result<FilterOutput, ChartError> {
    let idx_id = id_gen.next();
    let indices: Vec<BokehValue> = (0..n).map(|i| BokehValue::Int(i as i64)).collect();
    let idx_filter = BokehObject::new("IndexFilter", idx_id.clone())
        .attr("indices", BokehValue::Array(indices));

    let cds_placeholder_id = format!("__cds_{}", &filter.source_key);
    let desc_str = if descending { "true" } else { "false" };

    let cb_id = id_gen.next();
    let callback = BokehObject::new("CustomJS", cb_id)
        .attr("args", BokehValue::Map(vec![
            ("idx_filter".into(), BokehValue::ref_of(&idx_id)),
            ("source".into(), BokehValue::Ref(cds_placeholder_id)),
            ("col".into(), BokehValue::Str(filter.column.clone())),
            ("descending".into(), BokehValue::Bool(descending)),
        ]))
        .attr("code", BokehValue::Str(format!(
            "const n = cb_obj.value;\
             const data = source.data['{}'];\
             const indexed = data.map((v, i) => ({{v: v, i: i}}));\
             if ({}) {{\
                 indexed.sort((a, b) => b.v - a.v);\
             }} else {{\
                 indexed.sort((a, b) => a.v - b.v);\
             }}\
             idx_filter.indices = indexed.slice(0, n).map(x => x.i);\
             source.change.emit();",
            filter.column, desc_str
        )));

    let widget_id = id_gen.next();
    let widget = BokehObject::new("Slider", widget_id)
        .attr("title", BokehValue::Str(filter.label.clone()))
        .attr("start", BokehValue::Int(1))
        .attr("end", BokehValue::Int(max_n as i64))
        .attr("value", BokehValue::Int(max_n as i64))
        .attr("step", BokehValue::Int(1))
        .attr("sizing_mode", BokehValue::Str("stretch_width".into()))
        .attr("js_property_callbacks", BokehValue::Map(vec![
            ("change:value".into(), BokehValue::Array(vec![callback.into_value()])),
        ]));

    Ok(FilterOutput {
        widget,
        filter_id: idx_id,
        filter_obj: idx_filter,
        source_key: filter.source_key.clone(),
        switch_label: None,
        is_range_tool: false,
        range_tool_range_id: None,
        range_tool_overview: None,
    })
}
