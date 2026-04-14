//! Toolbar tool builders: pan, wheel zoom, box zoom/select, tap, reset, save, hover.

use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};

/// Build the standard `Toolbar` with pan/zoom/reset/save + box-select + tap,
/// optionally including a `HoverTool`.
pub(super) fn build_toolbar(id_gen: &mut IdGen, hover_tool: Option<BokehObject>) -> BokehObject {
    let toolbar_id = id_gen.next();
    let mut tools: Vec<BokehValue> = vec![
        build_pan_tool(id_gen).into_value(),
        build_wheel_zoom_tool(id_gen).into_value(),
        build_box_zoom_tool(id_gen).into_value(),
        build_reset_tool(id_gen).into_value(),
        build_save_tool(id_gen).into_value(),
    ];
    if let Some(ht) = hover_tool {
        tools.push(ht.into_value());
    }
    tools.push(build_box_select_tool(id_gen).into_value());
    tools.push(build_tap_tool(id_gen).into_value());

    BokehObject::new("Toolbar", toolbar_id)
        .attr("tools", BokehValue::Array(tools))
}

fn build_pan_tool(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("PanTool", id_gen.next())
}

fn build_wheel_zoom_tool(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("WheelZoomTool", id_gen.next())
        .attr("renderers", BokehValue::Str("auto".into()))
}

pub fn build_box_zoom_tool(id_gen: &mut IdGen) -> BokehObject {
    let ann = build_box_annotation_inner(id_gen, false);
    BokehObject::new("BoxZoomTool", id_gen.next())
        .attr("dimensions", BokehValue::Str("both".into()))
        .attr("overlay", ann.into_value())
}

pub fn build_box_select_tool(id_gen: &mut IdGen) -> BokehObject {
    let ann = build_box_annotation_inner(id_gen, true);
    BokehObject::new("BoxSelectTool", id_gen.next())
        .attr("renderers", BokehValue::Str("auto".into()))
        .attr("overlay", ann.into_value())
}

fn build_tap_tool(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("TapTool", id_gen.next())
        .attr("renderers", BokehValue::Str("auto".into()))
}

fn build_reset_tool(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("ResetTool", id_gen.next())
}

fn build_save_tool(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("SaveTool", id_gen.next())
}

/// Build a `HoverTool` from tooltip fields.
pub fn build_hover_tool(
    id_gen: &mut IdGen,
    tooltips: &[(&str, &str)],
    formatters: &[(&str, &str)],
) -> BokehObject {
    let tooltip_array: Vec<BokehValue> = tooltips
        .iter()
        .map(|(label, fmt)| {
            BokehValue::Array(vec![
                BokehValue::Str(label.to_string()),
                BokehValue::Str(fmt.to_string()),
            ])
        })
        .collect();

    let fmt_entries: Vec<(String, BokehValue)> = formatters
        .iter()
        .map(|(k, v)| (k.to_string(), BokehValue::Str(v.to_string())))
        .collect();

    let mut tool = BokehObject::new("HoverTool", id_gen.next())
        .attr("renderers", BokehValue::Str("auto".into()))
        .attr("tooltips", BokehValue::Array(tooltip_array));

    if !fmt_entries.is_empty() {
        tool = tool.attr("formatters", BokehValue::Map(fmt_entries));
    }

    tool
}

fn build_box_annotation_inner(id_gen: &mut IdGen, editable: bool) -> BokehObject {
    let handles_id = id_gen.next();
    let visuals_id = id_gen.next();
    let ann_id = id_gen.next();

    let visuals = BokehObject::new("AreaVisuals", visuals_id)
        .attr("fill_color", BokehValue::Str("white".into()))
        .attr("hover_fill_color", BokehValue::Str("lightgray".into()));

    let handles = BokehObject::new("BoxInteractionHandles", handles_id)
        .attr("all", visuals.into_value());

    let mut ann = BokehObject::new("BoxAnnotation", ann_id)
        .attr("syncable", BokehValue::Bool(false))
        .attr("line_color", BokehValue::Str("black".into()))
        .attr("line_alpha", BokehValue::Float(1.0))
        .attr("line_width", BokehValue::Int(2))
        .attr("line_dash", BokehValue::Array(vec![BokehValue::Int(4), BokehValue::Int(4)]))
        .attr("fill_color", BokehValue::Str("lightgrey".into()))
        .attr("fill_alpha", BokehValue::Float(0.5))
        .attr("level", BokehValue::Str("overlay".into()))
        .attr("visible", BokehValue::Bool(false))
        .attr("left",   BokehValue::NaN)
        .attr("right",  BokehValue::NaN)
        .attr("top",    BokehValue::NaN)
        .attr("bottom", BokehValue::NaN)
        .attr("left_units",   BokehValue::Str("canvas".into()))
        .attr("right_units",  BokehValue::Str("canvas".into()))
        .attr("top_units",    BokehValue::Str("canvas".into()))
        .attr("bottom_units", BokehValue::Str("canvas".into()))
        .attr("handles", handles.into_value());

    if editable {
        ann = ann.attr("editable", BokehValue::Bool(true));
    }

    ann
}
