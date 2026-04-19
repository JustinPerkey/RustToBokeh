//! Group filter — dropdown driving a Bokeh GroupFilter (single group match).

use crate::charts::FilterSpec;
use crate::error::ChartError;

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::FilterOutput;

pub(super) fn build_group_filter(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    options: &[String],
) -> Result<FilterOutput, ChartError> {
    let gf_id = id_gen.next();
    let default_val = options.first().map(|s| s.as_str()).unwrap_or("");
    let gf = BokehObject::new("GroupFilter", gf_id.clone())
        .attr("column_name", BokehValue::Str(filter.column.clone()))
        .attr("group", BokehValue::Str(default_val.to_string()));

    let cds_placeholder_id = format!("__cds_{}", &filter.source_key);
    let opts: Vec<BokehValue> = options.iter().map(|s| BokehValue::Str(s.clone())).collect();

    let cb_id = id_gen.next();
    let callback = BokehObject::new("CustomJS", cb_id)
        .attr("args", BokehValue::Map(vec![
            ("gf".into(), BokehValue::ref_of(&gf_id)),
            ("source".into(), BokehValue::Ref(cds_placeholder_id)),
        ]))
        .attr("code", BokehValue::Str(
            "gf.group = cb_obj.value; source.change.emit();".into()
        ));

    let widget_id = id_gen.next();
    let widget = BokehObject::new("Select", widget_id)
        .attr("title", BokehValue::Str(filter.label.clone()))
        .attr("value", BokehValue::Str(default_val.to_string()))
        .attr("options", BokehValue::Array(opts))
        .attr("sizing_mode", BokehValue::Str("stretch_width".into()))
        .attr("js_property_callbacks", BokehValue::Map(vec![
            ("change:value".into(), BokehValue::Array(vec![callback.into_value()])),
        ]));

    Ok(FilterOutput {
        widget,
        filter_id: gf_id,
        filter_obj: gf,
        source_key: filter.source_key.clone(),
        switch_label: None,
        is_range_tool: false,
        range_tool_range_id: None,
        range_tool_overview: None,
    })
}
