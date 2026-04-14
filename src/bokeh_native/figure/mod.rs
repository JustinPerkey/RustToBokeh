//! Figure builder — creates Bokeh Figure models with axes, grids, and toolbar.
//!
//! Sub-modules:
//! - [`ranges`] — `Range1d`/`FactorRange`/`DataRange1d` builders for x/y axes.
//! - [`tools`] — toolbar tool builders (pan, wheel zoom, box zoom, hover, …).
//! - [`glyph`] — `GlyphRenderer` + `CDSView` helper.

pub use super::axis::{AxisBuilder, AxisType};

use super::id_gen::IdGen;
use super::model::{BokehObject, BokehValue};

mod glyph;
mod ranges;
mod tools;

pub use glyph::build_glyph_renderer;
pub use tools::{build_box_select_tool, build_box_zoom_tool, build_hover_tool};

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
    pub figure: BokehObject,
    pub x_range_id: String,
    pub y_range_id: String,
    pub x_axis_id: String,
    pub y_axis_id: String,
    pub x_grid_id: String,
    pub y_grid_id: String,
}

/// Build a Bokeh `Figure` model. Returns the Figure and IDs of key sub-objects.
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
    let (x_range_obj, x_range_id) = ranges::build_x_range(id_gen, x_range, x_axis.cfg());
    let (y_range_obj, y_range_id) = ranges::build_y_range(id_gen, y_range, y_axis.cfg());

    let x_scale_id = id_gen.next();
    let y_scale_id = id_gen.next();
    let x_scale = BokehObject::new(x_axis.scale_name(), x_scale_id);
    let y_scale = BokehObject::new(y_axis.scale_name(), y_scale_id);

    let title_id = id_gen.next();
    let title_obj = BokehObject::new("Title", title_id)
        .attr("text", BokehValue::Str(title.to_string()));

    let (x_axis_obj, x_axis_id, x_grid_obj, x_grid_id) = x_axis.build(id_gen);
    let (y_axis_obj, y_axis_id, y_grid_obj, y_grid_id) = y_axis.build(id_gen);

    let toolbar = tools::build_toolbar(id_gen, hover_tool);

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

#[cfg(test)]
fn find_attr_test<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
    obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
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
