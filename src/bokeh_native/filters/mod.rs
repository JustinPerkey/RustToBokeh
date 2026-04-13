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

fn build_range_tool_overlay(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("BoxAnnotation", id_gen.next())
        .attr("fill_color", BokehValue::Str("navy".into()))
        .attr("fill_alpha", BokehValue::Float(0.2))
        .attr("line_color", BokehValue::Null)
        .attr("level", BokehValue::Str("underlay".into()))
}
