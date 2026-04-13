//! Filter widget builders for native Bokeh rendering.
//!
//! Each filter type maps to a Bokeh widget + filter model + CustomJS callback.
//! Filter model objects are embedded inline inside the widget's
//! `js_property_callbacks` map so that chart `CDSView.filter` cross-references
//! resolve correctly.

use std::collections::HashMap;
use polars::prelude::DataFrame;

use crate::charts::{FilterConfig, FilterSpec};
use crate::error::ChartError;

use super::id_gen::IdGen;
use super::model::{BokehObject, BokehValue};

/// Output from building a single filter widget.
pub struct FilterOutput {
    /// The widget root (RangeSlider, Select, Slider, Switch, DatetimeRangeSlider).
    pub widget: BokehObject,
    /// ID of the filter model (BooleanFilter, IndexFilter, GroupFilter).
    pub filter_id: String,
    /// The standalone filter model object — must be added as a document root
    /// so that cross-root references (from charts to this filter) resolve.
    pub filter_obj: BokehObject,
    /// Source key this filter applies to.
    pub source_key: String,
    /// Optional label for Switch widgets (displayed alongside the toggle).
    pub switch_label: Option<String>,
    /// Whether this is a RangeTool (special: no CDSView, returns Range1d ID).
    pub is_range_tool: bool,
    /// For RangeTool: the shared Range1d ID.
    pub range_tool_range_id: Option<String>,
    /// For RangeTool: the overview figure.
    pub range_tool_overview: Option<BokehObject>,
}

/// Build all filter widgets for a page.
///
/// Returns `(filter_outputs, range_tool_outputs)` where `filter_outputs`
/// are CDSView-based filters and `range_tool_outputs` are RangeTool navigators.
pub fn build_filter_widgets(
    id_gen: &mut IdGen,
    filters: &[FilterSpec],
    frames: &HashMap<String, DataFrame>,
) -> Result<(Vec<FilterOutput>, Vec<FilterOutput>), ChartError> {
    let mut cds_filters: Vec<FilterOutput> = Vec::new();
    let mut range_tool_filters: Vec<FilterOutput> = Vec::new();

    for filter in filters {
        if matches!(filter.config, FilterConfig::RangeTool { .. }) {
            let df = frames.get(&filter.source_key).ok_or_else(|| {
                ChartError::NativeRender(format!("source_key '{}' not found", filter.source_key))
            })?;
            let output = build_range_tool(id_gen, filter, df)?;
            range_tool_filters.push(output);
        } else {
            let df = frames.get(&filter.source_key).ok_or_else(|| {
                ChartError::NativeRender(format!("source_key '{}' not found", filter.source_key))
            })?;
            let n = df.height();
            let output = build_cds_filter(id_gen, filter, df, n)?;
            cds_filters.push(output);
        }
    }

    Ok((cds_filters, range_tool_filters))
}

/// For a set of filter outputs targeting the same source_key, build a
/// combined filter value using inline objects.
///
/// Returns an inline `BokehValue` suitable for a CDSView `filter` attribute.
/// The filter objects are embedded inline (same ID as in the widget's CustomJS
/// args) so BokehJS recognises them as the same model instance — no cross-root
/// references needed.
///
/// When `filter_objs` is empty, returns `AllIndices`.
/// When 1 filter: returns the filter object inline.
/// When >1 filters: returns `IntersectionFilter{ operands: [...] }`.
pub fn combine_filters(
    id_gen: &mut IdGen,
    filter_objs: &[BokehObject],
) -> BokehValue {
    match filter_objs.len() {
        0 => {
            let aid = id_gen.next();
            BokehObject::new("AllIndices", aid).into_value()
        }
        1 => filter_objs[0].clone().into_value(),
        _ => {
            let isect_id = id_gen.next();
            let operands: Vec<BokehValue> = filter_objs
                .iter()
                .map(|obj| obj.clone().into_value())
                .collect();
            BokehObject::new("IntersectionFilter", isect_id)
                .attr("operands", BokehValue::Array(operands))
                .into_value()
        }
    }
}

// ── Individual filter builders ────────────────────────────────────────────────

fn build_cds_filter(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    _df: &DataFrame,
    n: usize,
) -> Result<FilterOutput, ChartError> {
    match &filter.config {
        FilterConfig::Range { min, max, step } => {
            build_range_filter(id_gen, filter, n, *min, *max, *step)
        }
        FilterConfig::Select { options } => {
            build_select_filter(id_gen, filter, n, options)
        }
        FilterConfig::Group { options } => {
            build_group_filter(id_gen, filter, options)
        }
        FilterConfig::Threshold { value, above } => {
            build_threshold_filter(id_gen, filter, n, *value, *above)
        }
        FilterConfig::TopN { max_n, descending } => {
            build_top_n_filter(id_gen, filter, n, *max_n, *descending)
        }
        FilterConfig::DateRange { min_ms, max_ms, step, .. } => {
            build_date_range_filter(id_gen, filter, n, *min_ms, *max_ms, step.as_ms())
        }
        FilterConfig::RangeTool { .. } => unreachable!(),
    }
}

fn build_range_filter(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    n: usize,
    min: f64,
    max: f64,
    step: f64,
) -> Result<FilterOutput, ChartError> {
    build_range_slider_with_filter(id_gen, filter, n, min, max, step)
}

fn build_range_slider_with_filter(
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

    // CDS placeholder — will be cross-referenced at render time
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

fn build_select_filter(
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

fn build_group_filter(
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
            ("gf".into(), gf.clone().into_value()),
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

fn build_threshold_filter(
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

fn build_top_n_filter(
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
            ("idx_filter".into(), idx_filter.clone().into_value()),
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

fn build_date_range_filter(
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

fn build_range_tool(
    id_gen: &mut IdGen,
    filter: &FilterSpec,
    df: &DataFrame,
) -> Result<FilterOutput, ChartError> {
    use super::source::build_column_data_source;
    use super::figure::{build_figure, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};

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
    let _cds_id = cds.id.clone();

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

    // Add line renderer for y_col
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

    use super::figure::build_glyph_renderer;
    let renderer = build_glyph_renderer(id_gen, cds.into_value(), line_glyph, Some(line_nonsel), None);

    // Add RangeTool to toolbar
    let range_tool_id = id_gen.next();
    let range_tool = BokehObject::new("RangeTool", range_tool_id)
        .attr("x_range", range_widget.to_ref())
        .attr("overlay", build_range_tool_overlay(id_gen).into_value());

    // Add renderer and RangeTool to figure
    use super::charts::add_renderers;
    add_renderers(&mut figure, vec![renderer]);

    // Add range_tool to toolbar tools
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

#[cfg(test)]
fn find_attr_test<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
    obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn build_range_tool_overlay(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("BoxAnnotation", id_gen.next())
        .attr("fill_color", BokehValue::Str("navy".into()))
        .attr("fill_alpha", BokehValue::Float(0.2))
        .attr("line_color", BokehValue::Null)
        .attr("level", BokehValue::Str("underlay".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn make_frames(key: &str, df: DataFrame) -> HashMap<String, DataFrame> {
        let mut m = HashMap::new();
        m.insert(key.into(), df);
        m
    }

    fn test_df() -> DataFrame {
        df![
            "value"    => [10.0, 20.0, 30.0, 40.0, 50.0],
            "category" => ["A", "B", "A", "B", "A"],
        ].unwrap()
    }

    // ── Range filter ────────────────────────────────────────────────────────

    #[test]
    fn range_filter_produces_range_slider_and_boolean_filter() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::range("src", "value", "Value Range", 10.0, 50.0, 5.0),
        ];
        let mut id_gen = IdGen::new();
        let (cds_filters, rt_filters) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();

        assert_eq!(cds_filters.len(), 1);
        assert!(rt_filters.is_empty());

        let f = &cds_filters[0];
        assert_eq!(f.widget.name, "RangeSlider");
        assert_eq!(f.filter_obj.name, "BooleanFilter");
        assert_eq!(f.source_key, "src");
        assert!(!f.is_range_tool);
        assert!(f.switch_label.is_none());
    }

    #[test]
    fn range_filter_slider_has_correct_bounds() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::range("src", "value", "Range", 10.0, 50.0, 5.0)];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        let json = serde_json::to_string(&cds[0].widget).unwrap();
        assert!(json.contains("10.0") || json.contains("10"));
        assert!(json.contains("50.0") || json.contains("50"));
    }

    #[test]
    fn range_filter_boolean_array_all_true_initially() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::range("src", "value", "Range", 0.0, 100.0, 1.0)];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();

        if let Some(BokehValue::Array(bools)) = find_attr_test(&cds[0].filter_obj, "booleans") {
            assert_eq!(bools.len(), 5);
            for b in bools {
                assert!(matches!(b, BokehValue::Bool(true)));
            }
        } else {
            panic!("expected booleans array");
        }
    }

    #[test]
    fn range_filter_customjs_references_column() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::range("src", "value", "Range", 0.0, 100.0, 1.0)];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        let json = serde_json::to_string(&cds[0].widget).unwrap();
        assert!(json.contains("CustomJS"));
        assert!(json.contains("value"), "JS code should reference column");
    }

    // ── Select filter ───────────────────────────────────────────────────────

    #[test]
    fn select_filter_produces_select_widget_and_boolean_filter() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::select("src", "category", "Category", vec!["A", "B"]),
        ];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();

        assert_eq!(cds[0].widget.name, "Select");
        assert_eq!(cds[0].filter_obj.name, "BooleanFilter");
    }

    #[test]
    fn select_filter_has_all_option() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::select("src", "category", "Cat", vec!["A", "B"])];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        let json = serde_json::to_string(&cds[0].widget).unwrap();
        assert!(json.contains("(All)"));
    }

    #[test]
    fn select_filter_default_value_is_all() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::select("src", "category", "Cat", vec!["X"])];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        if let Some(BokehValue::Str(val)) = find_attr_test(&cds[0].widget, "value") {
            assert_eq!(val, "(All)");
        }
    }

    // ── Group filter ────────────────────────────────────────────────────────

    #[test]
    fn group_filter_produces_select_widget_and_group_filter() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::group("src", "category", "Group", vec!["A", "B"]),
        ];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();

        assert_eq!(cds[0].widget.name, "Select");
        assert_eq!(cds[0].filter_obj.name, "GroupFilter");
    }

    #[test]
    fn group_filter_default_is_first_option() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::group("src", "category", "Grp", vec!["A", "B"])];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        if let Some(BokehValue::Str(val)) = find_attr_test(&cds[0].widget, "value") {
            assert_eq!(val, "A");
        }
    }

    #[test]
    fn group_filter_has_column_name() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::group("src", "category", "Grp", vec!["A"])];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        if let Some(BokehValue::Str(col)) = find_attr_test(&cds[0].filter_obj, "column_name") {
            assert_eq!(col, "category");
        }
    }

    #[test]
    fn group_filter_no_all_option() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::group("src", "category", "Grp", vec!["A", "B"])];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        let json = serde_json::to_string(&cds[0].widget).unwrap();
        assert!(!json.contains("(All)"), "GroupFilter should not have All option");
    }

    // ── Threshold filter ────────────────────────────────────────────────────

    #[test]
    fn threshold_filter_produces_switch_and_boolean_filter() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::threshold("src", "value", "High Only", 30.0, true),
        ];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();

        assert_eq!(cds[0].widget.name, "Switch");
        assert_eq!(cds[0].filter_obj.name, "BooleanFilter");
        assert_eq!(cds[0].switch_label.as_deref(), Some("High Only"));
    }

    #[test]
    fn threshold_switch_starts_inactive() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::threshold("src", "value", "T", 30.0, true)];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        if let Some(BokehValue::Bool(active)) = find_attr_test(&cds[0].widget, "active") {
            assert!(!active);
        }
    }

    // ── TopN filter ─────────────────────────────────────────────────────────

    #[test]
    fn top_n_filter_produces_slider_and_index_filter() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::top_n("src", "value", "Top N", 5, true),
        ];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();

        assert_eq!(cds[0].widget.name, "Slider");
        assert_eq!(cds[0].filter_obj.name, "IndexFilter");
    }

    #[test]
    fn top_n_index_filter_initially_includes_all_rows() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::top_n("src", "value", "Top N", 5, true)];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        if let Some(BokehValue::Array(indices)) = find_attr_test(&cds[0].filter_obj, "indices") {
            assert_eq!(indices.len(), 5);
        }
    }

    #[test]
    fn top_n_slider_max_is_correct() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![FilterSpec::top_n("src", "value", "Top N", 10, false)];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        if let Some(BokehValue::Int(end)) = find_attr_test(&cds[0].widget, "end") {
            assert_eq!(*end, 10);
        }
    }

    // ── DateRange filter ────────────────────────────────────────────────────

    #[test]
    fn date_range_filter_produces_datetime_slider_and_boolean_filter() {
        let df = df![
            "timestamp_ms" => [1000.0, 2000.0, 3000.0],
            "value"        => [10.0, 20.0, 30.0],
        ].unwrap();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::date_range(
                "src", "timestamp_ms", "Date Range",
                1000.0, 3000.0,
                crate::charts::DateStep::Day,
                crate::charts::TimeScale::Days,
            ),
        ];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();

        assert_eq!(cds[0].widget.name, "DatetimeRangeSlider");
        assert_eq!(cds[0].filter_obj.name, "BooleanFilter");
    }

    // ── RangeTool filter ────────────────────────────────────────────────────

    #[test]
    fn range_tool_filter_produces_range1d_and_overview_figure() {
        let df = df![
            "x" => [1.0, 2.0, 3.0, 4.0, 5.0],
            "y" => [10.0, 20.0, 30.0, 40.0, 50.0],
        ].unwrap();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::range_tool("src", "x", "y", "Navigator", 1.0, 5.0, None),
        ];
        let mut id_gen = IdGen::new();
        let (cds, rt) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();

        assert!(cds.is_empty(), "RangeTool should not produce CDS filters");
        assert_eq!(rt.len(), 1);
        assert!(rt[0].is_range_tool);
        assert_eq!(rt[0].widget.name, "Range1d");
        assert!(rt[0].range_tool_range_id.is_some());
        assert!(rt[0].range_tool_overview.is_some());
    }

    #[test]
    fn range_tool_overview_is_figure_with_renderers() {
        let df = df!["x" => [1.0, 2.0], "y" => [10.0, 20.0]].unwrap();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::range_tool("src", "x", "y", "Nav", 1.0, 2.0, None),
        ];
        let mut id_gen = IdGen::new();
        let (_, rt) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        let overview = rt[0].range_tool_overview.as_ref().unwrap();
        assert_eq!(overview.name, "Figure");
        let json = serde_json::to_string(overview).unwrap();
        assert!(json.contains("RangeTool"));
    }

    #[test]
    fn range_tool_has_boolean_filter_for_filtered_charts() {
        let df = df!["x" => [1.0, 2.0], "y" => [10.0, 20.0]].unwrap();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::range_tool("src", "x", "y", "Nav", 1.0, 2.0, None),
        ];
        let mut id_gen = IdGen::new();
        let (_, rt) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        assert_eq!(rt[0].filter_obj.name, "BooleanFilter");
    }

    // ── Missing source key ──────────────────────────────────────────────────

    #[test]
    fn missing_source_key_returns_error() {
        let frames: HashMap<String, DataFrame> = HashMap::new();
        let filters = vec![FilterSpec::range("missing", "value", "R", 0.0, 1.0, 0.1)];
        let mut id_gen = IdGen::new();
        let result = build_filter_widgets(&mut id_gen, &filters, &frames);
        assert!(result.is_err());
    }

    // ── combine_filters ─────────────────────────────────────────────────────

    #[test]
    fn combine_zero_filters_produces_all_indices() {
        let mut id_gen = IdGen::new();
        let result = combine_filters(&mut id_gen, &[]);
        if let BokehValue::Object(obj) = result {
            assert_eq!(obj.name, "AllIndices");
        } else {
            panic!("expected AllIndices object");
        }
    }

    #[test]
    fn combine_one_filter_returns_inline_filter() {
        let mut id_gen = IdGen::new();
        let bf = BokehObject::new("BooleanFilter", "bf1".into())
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true)]));
        let result = combine_filters(&mut id_gen, &[bf]);
        if let BokehValue::Object(obj) = result {
            assert_eq!(obj.name, "BooleanFilter");
        } else {
            panic!("expected BooleanFilter object");
        }
    }

    #[test]
    fn combine_two_filters_produces_intersection_filter() {
        let mut id_gen = IdGen::new();
        let bf1 = BokehObject::new("BooleanFilter", "bf1".into());
        let bf2 = BokehObject::new("BooleanFilter", "bf2".into());
        let result = combine_filters(&mut id_gen, &[bf1, bf2]);
        if let BokehValue::Object(obj) = result {
            assert_eq!(obj.name, "IntersectionFilter");
            if let Some(BokehValue::Array(operands)) = find_attr_test(&obj, "operands") {
                assert_eq!(operands.len(), 2);
            } else {
                panic!("expected operands array");
            }
        } else {
            panic!("expected IntersectionFilter object");
        }
    }

    #[test]
    fn combine_three_filters_intersection_has_three_operands() {
        let mut id_gen = IdGen::new();
        let filters: Vec<BokehObject> = (0..3)
            .map(|i| BokehObject::new("BooleanFilter", format!("bf{i}")))
            .collect();
        let result = combine_filters(&mut id_gen, &filters);
        if let BokehValue::Object(obj) = result {
            assert_eq!(obj.name, "IntersectionFilter");
            if let Some(BokehValue::Array(operands)) = find_attr_test(&obj, "operands") {
                assert_eq!(operands.len(), 3);
            }
        }
    }

    // ── Multiple filters on same page ───────────────────────────────────────

    #[test]
    fn multiple_filter_types_on_same_source() {
        let df = test_df();
        let frames = make_frames("src", df);
        let filters = vec![
            FilterSpec::range("src", "value", "Range", 0.0, 100.0, 1.0),
            FilterSpec::select("src", "category", "Category", vec!["A", "B"]),
            FilterSpec::threshold("src", "value", "High", 30.0, true),
        ];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        assert_eq!(cds.len(), 3);
        assert_eq!(cds[0].widget.name, "RangeSlider");
        assert_eq!(cds[1].widget.name, "Select");
        assert_eq!(cds[2].widget.name, "Switch");
    }

    // ── CDS placeholder in callbacks ────────────────────────────────────────

    #[test]
    fn filter_callbacks_reference_cds_placeholder() {
        let df = test_df();
        let frames = make_frames("mydata", df);
        let filters = vec![FilterSpec::range("mydata", "value", "R", 0.0, 100.0, 1.0)];
        let mut id_gen = IdGen::new();
        let (cds, _) = build_filter_widgets(&mut id_gen, &filters, &frames).unwrap();
        let json = serde_json::to_string(&cds[0].widget).unwrap();
        assert!(json.contains("__cds_mydata"), "CustomJS should ref CDS placeholder");
    }
}
