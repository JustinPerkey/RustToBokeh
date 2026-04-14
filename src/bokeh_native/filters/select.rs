//! Select filter — dropdown with "(All)" option driving a BooleanFilter.

use crate::charts::FilterSpec;
use crate::error::ChartError;

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::FilterOutput;

pub(super) fn build_select_filter(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    n: usize,
    options: &[String],
) -> Result<FilterOutput, ChartError> {
    let bf_id = id_gen.next();
    let bf = BokehObject::new("BooleanFilter", bf_id.clone())
        .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); n]));

    let cds_placeholder_id = format!("__cds_{}", &filter.source_key);
    let all_opts: Vec<BokehValue> = std::iter::once("(All)".to_string())
        .chain(options.iter().cloned())
        .map(|s| BokehValue::Str(s))
        .collect();

    let cb_id = id_gen.next();
    let callback = BokehObject::new("CustomJS", cb_id)
        .attr("args", BokehValue::Map(vec![
            ("bf".into(), bf.clone().into_value()),
            ("source".into(), BokehValue::Ref(cds_placeholder_id)),
            ("col".into(), BokehValue::Str(filter.column.clone())),
        ]))
        .attr("code", BokehValue::Str(format!(
            "const val = cb_obj.value;\
             const data = source.data['{}'];\
             if (val === '(All)') {{\
                 bf.booleans = data.map(() => true);\
             }} else {{\
                 bf.booleans = data.map(v => v === val);\
             }}\
             source.change.emit();",
            filter.column
        )));

    let widget_id = id_gen.next();
    let widget = BokehObject::new("Select", widget_id)
        .attr("title", BokehValue::Str(filter.label.clone()))
        .attr("value", BokehValue::Str("(All)".into()))
        .attr("options", BokehValue::Array(all_opts))
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
