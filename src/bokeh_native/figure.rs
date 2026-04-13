//! Figure builder — creates Bokeh Figure models with axes, grids, and toolbar.

use crate::charts::AxisConfig;

pub use super::axis::{AxisBuilder, AxisType};
use super::id_gen::IdGen;
use super::model::{BokehObject, BokehValue};

/// The kind of x-axis range to use.
pub enum XRangeKind {
    /// Categorical axis (FactorRange). Provide the list of category strings.
    Factor(Vec<BokehValue>),
    /// Numeric range (Range1d). `start` and `end` may both be 0.0 for auto.
    Numeric { start: f64, end: f64 },
    /// Datetime axis (Range1d with ms values).
    Datetime { start: f64, end: f64 },
    /// Use a pre-existing Range1d by ID (for RangeTool synchronisation).
    ExistingId(String),
    /// Auto-size (DataRange1d).
    DataRange,
}

/// The kind of y-axis range.
pub enum YRangeKind {
    /// Auto-size (DataRange1d).
    DataRange,
    /// Numeric Range1d.
    Numeric { start: f64, end: f64 },
    /// Categorical axis (FactorRange). Used for horizontal bar charts.
    Factor(Vec<BokehValue>),
}

/// Output produced by `build_figure`.
pub struct FigureOutput {
    /// The Figure model itself.
    pub figure: BokehObject,
    /// ID of the x Range object (FactorRange or Range1d or DataRange1d).
    pub x_range_id: String,
    /// ID of the y Range object.
    pub y_range_id: String,
    /// ID of the x-axis (CategoricalAxis or LinearAxis or DatetimeAxis).
    pub x_axis_id: String,
    /// ID of the y-axis.
    pub y_axis_id: String,
    /// ID of the x Grid.
    pub x_grid_id: String,
    /// ID of the y Grid.
    pub y_grid_id: String,
}

/// Build a Bokeh `Figure` model.
///
/// Returns the Figure and IDs of key sub-objects for later configuration.
pub fn build_figure(
    id_gen: &mut IdGen,
    title: &str,
    height: u32,
    width: Option<u32>,
    x_range: XRangeKind,
    y_range: YRangeKind,
    x_axis: AxisBuilder<'_>,
    y_axis: AxisBuilder<'_>,
    hover_tool: Option<BokehObject>,
) -> FigureOutput {
    // ── Ranges ───────────────────────────────────────────────────────────────
    let (x_range_obj, x_range_id) = build_x_range(id_gen, x_range, x_axis.cfg());
    let (y_range_obj, y_range_id) = build_y_range(id_gen, y_range, y_axis.cfg());

    // ── Scales ───────────────────────────────────────────────────────────────
    let x_scale_id = id_gen.next();
    let y_scale_id = id_gen.next();
    let x_scale = BokehObject::new(x_axis.scale_name(), x_scale_id.clone());
    let y_scale = BokehObject::new(y_axis.scale_name(), y_scale_id.clone());

    // ── Title ────────────────────────────────────────────────────────────────
    let title_id = id_gen.next();
    let title_obj = BokehObject::new("Title", title_id)
        .attr("text", BokehValue::Str(title.to_string()));

    // ── Axes ─────────────────────────────────────────────────────────────────
    let (x_axis_obj, x_axis_id, x_grid_obj, x_grid_id) = x_axis.build(id_gen);
    let (y_axis_obj, y_axis_id, y_grid_obj, y_grid_id) = y_axis.build(id_gen);

    // ── Toolbar ──────────────────────────────────────────────────────────────
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

    let toolbar = BokehObject::new("Toolbar", toolbar_id.clone())
        .attr("tools", BokehValue::Array(tools));

    // ── Figure ───────────────────────────────────────────────────────────────
    let fig_id = id_gen.next();
    let mut fig_attrs: Vec<(&str, BokehValue)> = vec![
        ("height", BokehValue::Int(height as i64)),
    ];

    if let Some(w) = width {
        fig_attrs.push(("width", BokehValue::Int(w as i64)));
        fig_attrs.push(("sizing_mode", BokehValue::Str("fixed".into())));
    } else {
        fig_attrs.push(("sizing_mode", BokehValue::Str("stretch_width".into())));
    }

    // Use ref for ranges since they were built inline — embed them inline instead
    fig_attrs.push(("x_range", x_range_obj.into_value()));
    fig_attrs.push(("y_range", y_range_obj.into_value()));
    fig_attrs.push(("x_scale", x_scale.into_value()));
    fig_attrs.push(("y_scale", y_scale.into_value()));
    fig_attrs.push(("title", title_obj.into_value()));
    fig_attrs.push(("renderers", BokehValue::Array(vec![])));
    fig_attrs.push(("toolbar", toolbar.into_value()));
    fig_attrs.push(("toolbar_location", BokehValue::Str("above".into())));
    fig_attrs.push(("left", BokehValue::Array(vec![y_axis_obj.into_value()])));
    fig_attrs.push(("below", BokehValue::Array(vec![x_axis_obj.into_value()])));
    fig_attrs.push((
        "center",
        BokehValue::Array(vec![x_grid_obj.into_value(), y_grid_obj.into_value()]),
    ));

    let figure = BokehObject::with_attrs("Figure", fig_id, fig_attrs);

    FigureOutput {
        figure,
        x_range_id,
        y_range_id,
        x_axis_id,
        y_axis_id,
        x_grid_id,
        y_grid_id,
    }
}

// ── Range builders ────────────────────────────────────────────────────────────

fn build_x_range(
    id_gen: &mut IdGen,
    kind: XRangeKind,
    cfg: Option<&AxisConfig>,
) -> (BokehObject, String) {
    match kind {
        XRangeKind::Factor(factors) => {
            let id = id_gen.next();
            let obj = BokehObject::new("FactorRange", id.clone())
                .attr("factors", BokehValue::Array(factors));
            (obj, id)
        }
        XRangeKind::Numeric { start, end } => {
            let id = id_gen.next();
            let mut obj = BokehObject::new("DataRange1d", id.clone());
            // If explicit bounds given, use Range1d
            if start != 0.0 || end != 0.0 {
                obj = BokehObject::new("Range1d", id.clone())
                    .attr("start", BokehValue::Float(start))
                    .attr("end", BokehValue::Float(end));
            }
            if let Some(cfg) = cfg {
                obj = apply_range_config(id_gen, obj, cfg);
            }
            (obj, id)
        }
        XRangeKind::Datetime { start, end } => {
            let id = id_gen.next();
            let obj = BokehObject::new("Range1d", id.clone())
                .attr("start", BokehValue::Float(start))
                .attr("end", BokehValue::Float(end));
            (obj, id)
        }
        XRangeKind::ExistingId(existing_id) => {
            // Create a placeholder object — caller will handle the actual
            // range object. We return a dummy so the Figure's x_range ref works.
            let obj = BokehObject::new("Range1d", existing_id.clone());
            (obj, existing_id)
        }
        XRangeKind::DataRange => {
            let id = id_gen.next();
            let obj = BokehObject::new("DataRange1d", id.clone());
            (obj, id)
        }
    }
}

fn build_y_range(
    id_gen: &mut IdGen,
    kind: YRangeKind,
    cfg: Option<&AxisConfig>,
) -> (BokehObject, String) {
    match kind {
        YRangeKind::DataRange => {
            let id = id_gen.next();
            let mut obj = BokehObject::new("DataRange1d", id.clone());
            if let Some(cfg) = cfg {
                obj = apply_range_config(id_gen, obj, cfg);
            }
            (obj, id)
        }
        YRangeKind::Numeric { start, end } => {
            let id = id_gen.next();
            let obj = BokehObject::new("Range1d", id.clone())
                .attr("start", BokehValue::Float(start))
                .attr("end", BokehValue::Float(end));
            (obj, id)
        }
        YRangeKind::Factor(factors) => {
            let id = id_gen.next();
            let obj = BokehObject::new("FactorRange", id.clone())
                .attr("factors", BokehValue::Array(factors));
            (obj, id)
        }
    }
}

fn apply_range_config(
    _id_gen: &mut IdGen,
    mut obj: BokehObject,
    cfg: &AxisConfig,
) -> BokehObject {
    if let Some(start) = cfg.start {
        obj = obj.attr("start", BokehValue::Float(start));
    }
    if let Some(end) = cfg.end {
        obj = obj.attr("end", BokehValue::Float(end));
    }
    if let (Some(bmin), Some(bmax)) = (cfg.bounds_min, cfg.bounds_max) {
        obj = obj.attr(
            "bounds",
            BokehValue::Array(vec![BokehValue::Float(bmin), BokehValue::Float(bmax)]),
        );
    }
    obj
}

// ── Tool builders ─────────────────────────────────────────────────────────────

fn build_pan_tool(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("PanTool", id_gen.next())
}

fn build_wheel_zoom_tool(id_gen: &mut IdGen) -> BokehObject {
    BokehObject::new("WheelZoomTool", id_gen.next())
        .attr("renderers", BokehValue::Str("auto".into()))
}

pub fn build_box_zoom_tool(id_gen: &mut IdGen) -> BokehObject {
    let ann = build_box_annotation(id_gen);
    BokehObject::new("BoxZoomTool", id_gen.next())
        .attr("dimensions", BokehValue::Str("both".into()))
        .attr("overlay", ann.into_value())
}

pub fn build_box_select_tool(id_gen: &mut IdGen) -> BokehObject {
    let ann = build_box_annotation_editable(id_gen);
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
///
/// Returns `None` if `tooltips_spec` is empty.
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

fn build_box_annotation(id_gen: &mut IdGen) -> BokehObject {
    build_box_annotation_inner(id_gen, false)
}

fn build_box_annotation_editable(id_gen: &mut IdGen) -> BokehObject {
    build_box_annotation_inner(id_gen, true)
}

#[cfg(test)]
fn find_attr_test<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
    obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
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

// ── GlyphRenderer helper ─────────────────────────────────────────────────────

/// Build a `GlyphRenderer` with a given glyph and optional CDSView filter.
pub fn build_glyph_renderer(
    id_gen: &mut IdGen,
    source_ref: BokehValue,
    glyph: BokehObject,
    nonselection_glyph: Option<BokehObject>,
    filter_ref: Option<BokehValue>, // None → AllIndices
) -> BokehObject {
    let view_id = id_gen.next();
    let all_indices_id = id_gen.next();
    let renderer_id = id_gen.next();

    let filter_val = filter_ref.unwrap_or_else(|| {
        BokehObject::new("AllIndices", all_indices_id).into_value()
    });

    let view = BokehObject::new("CDSView", view_id)
        .attr("filter", filter_val);

    let nonsel = nonselection_glyph.unwrap_or_else(|| {
        let id = id_gen.next();
        BokehObject::new("Line", id) // placeholder; caller should provide proper one
    });

    BokehObject::new("GlyphRenderer", renderer_id)
        .attr("data_source", source_ref)
        .attr("view", view.into_value())
        .attr("glyph", glyph.into_value())
        .attr("nonselection_glyph", nonsel.into_value())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── build_figure ────────────────────────────────────────────────────────

    #[test]
    fn figure_with_factor_range() {
        let mut id_gen = IdGen::new();
        let factors = vec![BokehValue::Str("A".into()), BokehValue::Str("B".into())];
        let out = build_figure(
            &mut id_gen, "Test", 400, None,
            XRangeKind::Factor(factors),
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Categorical),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        assert_eq!(out.figure.name, "Figure");
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("FactorRange"));
        assert!(json.contains("CategoricalScale"));
        assert!(json.contains("CategoricalAxis"));
    }

    #[test]
    fn figure_with_numeric_range() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "Num", 400, None,
            XRangeKind::Numeric { start: 0.0, end: 100.0 },
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("Range1d"));
        assert!(json.contains("LinearScale"));
    }

    #[test]
    fn figure_with_data_range() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "Auto", 400, None,
            XRangeKind::DataRange,
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("DataRange1d"));
    }

    #[test]
    fn figure_with_datetime_range() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "DT", 400, None,
            XRangeKind::Datetime { start: 1000.0, end: 9000.0 },
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Datetime),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("DatetimeAxis"));
        assert!(json.contains("1000.0") || json.contains("1000"));
    }

    #[test]
    fn figure_with_existing_x_range_id() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "Existing", 400, None,
            XRangeKind::ExistingId("shared_r1".into()),
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        assert_eq!(out.x_range_id, "shared_r1");
    }

    #[test]
    fn figure_with_fixed_width() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "Fixed", 400, Some(800),
            XRangeKind::DataRange,
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("\"fixed\""));
        assert!(json.contains("800"));
    }

    #[test]
    fn figure_stretch_width_when_no_width() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "Stretch", 400, None,
            XRangeKind::DataRange,
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("stretch_width"));
    }

    #[test]
    fn figure_has_standard_tools() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "Tools", 400, None,
            XRangeKind::DataRange,
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("PanTool"));
        assert!(json.contains("WheelZoomTool"));
        assert!(json.contains("BoxZoomTool"));
        assert!(json.contains("ResetTool"));
        assert!(json.contains("SaveTool"));
        assert!(json.contains("BoxSelectTool"));
        assert!(json.contains("TapTool"));
    }

    #[test]
    fn figure_with_hover_tool() {
        let mut id_gen = IdGen::new();
        let ht = build_hover_tool(&mut id_gen, &[("X", "@{x}")], &[]);
        let out = build_figure(
            &mut id_gen, "Hover", 400, None,
            XRangeKind::DataRange,
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            Some(ht),
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("HoverTool"));
    }

    #[test]
    fn figure_has_title() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "My Title", 400, None,
            XRangeKind::DataRange,
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("My Title"));
    }

    #[test]
    fn figure_has_grids() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "Grid", 400, None,
            XRangeKind::DataRange,
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("Grid"));
    }

    #[test]
    fn figure_returns_unique_ids() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "IDs", 400, None,
            XRangeKind::DataRange,
            YRangeKind::DataRange,
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        // All IDs should be distinct
        let ids = vec![
            &out.x_range_id, &out.y_range_id,
            &out.x_axis_id, &out.y_axis_id,
            &out.x_grid_id, &out.y_grid_id,
        ];
        let mut unique = std::collections::HashSet::new();
        for id in &ids {
            assert!(unique.insert(id.as_str()), "duplicate ID: {}", id);
        }
    }

    // ── build_glyph_renderer ────────────────────────────────────────────────

    #[test]
    fn glyph_renderer_without_filter_uses_all_indices() {
        let mut id_gen = IdGen::new();
        let glyph = BokehObject::new("Scatter", id_gen.next());
        let source_ref = BokehValue::Ref("cds1".into());
        let renderer = build_glyph_renderer(&mut id_gen, source_ref, glyph, None, None);
        assert_eq!(renderer.name, "GlyphRenderer");
        let json = serde_json::to_string(&renderer).unwrap();
        assert!(json.contains("AllIndices"));
        assert!(json.contains("CDSView"));
    }

    #[test]
    fn glyph_renderer_with_filter_embeds_filter() {
        let mut id_gen = IdGen::new();
        let glyph = BokehObject::new("Scatter", id_gen.next());
        let source_ref = BokehValue::Ref("cds1".into());
        let filter = BokehObject::new("BooleanFilter", "bf1".into()).into_value();
        let renderer = build_glyph_renderer(&mut id_gen, source_ref, glyph, None, Some(filter));
        let json = serde_json::to_string(&renderer).unwrap();
        assert!(json.contains("BooleanFilter"));
    }

    #[test]
    fn glyph_renderer_has_nonselection_glyph() {
        let mut id_gen = IdGen::new();
        let glyph = BokehObject::new("Scatter", id_gen.next());
        let nonsel = BokehObject::new("Scatter", id_gen.next())
            .attr("fill_alpha", BokehValue::Float(0.1));
        let source_ref = BokehValue::Ref("cds1".into());
        let renderer = build_glyph_renderer(&mut id_gen, source_ref, glyph, Some(nonsel), None);
        if let Some(BokehValue::Object(ns)) = find_attr_test(&renderer, "nonselection_glyph") {
            assert_eq!(ns.name, "Scatter");
        }
    }

    // ── build_hover_tool ────────────────────────────────────────────────────

    #[test]
    fn hover_tool_with_formatters() {
        let mut id_gen = IdGen::new();
        let ht = build_hover_tool(
            &mut id_gen,
            &[("Time", "@{ts}{%Y-%m-%d}")],
            &[("@{ts}", "datetime")],
        );
        assert_eq!(ht.name, "HoverTool");
        let json = serde_json::to_string(&ht).unwrap();
        assert!(json.contains("datetime"));
        assert!(json.contains("formatters"));
    }

    #[test]
    fn hover_tool_without_formatters() {
        let mut id_gen = IdGen::new();
        let ht = build_hover_tool(&mut id_gen, &[("X", "@{x}")], &[]);
        let json = serde_json::to_string(&ht).unwrap();
        assert!(!json.contains("formatters"), "no formatters should be emitted");
    }

    // ── Y FactorRange ───────────────────────────────────────────────────────

    #[test]
    fn figure_with_y_factor_range() {
        let mut id_gen = IdGen::new();
        let factors = vec![BokehValue::Str("X".into()), BokehValue::Str("Y".into())];
        let out = build_figure(
            &mut id_gen, "YFactor", 400, None,
            XRangeKind::DataRange,
            YRangeKind::Factor(factors),
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Categorical),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("FactorRange"));
    }

    #[test]
    fn figure_with_y_numeric_range() {
        let mut id_gen = IdGen::new();
        let out = build_figure(
            &mut id_gen, "YNum", 400, None,
            XRangeKind::DataRange,
            YRangeKind::Numeric { start: 0.0, end: 100.0 },
            AxisBuilder::x(AxisType::Linear),
            AxisBuilder::y(AxisType::Linear),
            None,
        );
        let json = serde_json::to_string(&out.figure).unwrap();
        assert!(json.contains("100.0") || json.contains("100"));
    }
}
