//! Scatter chart builder.

use polars::prelude::DataFrame;

use crate::charts::charts::ScatterConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::source::build_column_data_source;
use super::{add_renderers, make_hover_tool, set_axis_labels};

/// Extract a named attribute from a `BokehObject`.
#[cfg(test)]
fn find_attr<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
    obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

pub fn build_scatter(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &ScatterConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
    range_tool_x_range_id: Option<&str>,
) -> Result<BokehObject, ChartError> {
    let ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &[cfg.x_col.as_str(), cfg.y_col.as_str()],
    );

    // Detect datetime x-axis from axis config
    let is_datetime = cfg.x_axis.as_ref()
        .and_then(|a| a.time_scale.as_ref())
        .is_some();

    let x_range = if let Some(rt_id) = range_tool_x_range_id {
        XRangeKind::ExistingId(rt_id.to_string())
    } else if is_datetime {
        XRangeKind::DataRange
    } else {
        XRangeKind::DataRange
    };

    let x_axis_type = if is_datetime { AxisType::Datetime } else { AxisType::Linear };

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        x_range,
        YRangeKind::DataRange,
        AxisBuilder::x(x_axis_type).config(cfg.x_axis.as_ref()),
        AxisBuilder::y(AxisType::Linear).config(cfg.y_axis.as_ref()),
        Some(ht),
    );

    let color = cfg.color.as_deref().unwrap_or("#4C72B0");
    let marker = cfg.marker.as_ref().map(|m| m.as_str()).unwrap_or("circle");
    let size = cfg.marker_size.unwrap_or(10.0);
    let alpha = cfg.alpha.unwrap_or(0.7);

    let cds = build_column_data_source(id_gen, df);
    let cds_ref = cds.into_value();

    let glyph_id = id_gen.next();
    let glyph = BokehObject::new("Scatter", glyph_id)
        .attr("x", BokehValue::field(&cfg.x_col))
        .attr("y", BokehValue::field(&cfg.y_col))
        .attr("size", BokehValue::value_of(BokehValue::Float(size)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(alpha)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
        .attr("marker", BokehValue::value_of(BokehValue::Str(marker.to_string())));

    let nonsel_id = id_gen.next();
    let nonsel = BokehObject::new("Scatter", nonsel_id)
        .attr("x", BokehValue::field(&cfg.x_col))
        .attr("y", BokehValue::field(&cfg.y_col))
        .attr("size", BokehValue::value_of(BokehValue::Float(size)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
        .attr("marker", BokehValue::value_of(BokehValue::Str(marker.to_string())));

    let renderer = build_glyph_renderer(id_gen, cds_ref, glyph, Some(nonsel), filter_ref);
    add_renderers(&mut figure, vec![renderer]);
    set_axis_labels(&mut figure, &cfg.x_label, &cfg.y_label);

    Ok(figure)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn test_df() -> DataFrame {
        df!["x" => [1.0, 2.0, 3.0], "y" => [4.0, 5.0, 6.0]].unwrap()
    }

    fn test_spec(title: &str) -> ChartSpec {
        ChartSpec {
            title: title.into(),
            source_key: "test".into(),
            config: crate::charts::ChartConfig::Scatter(
                ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap(),
            ),
            grid: crate::charts::GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
            width: None,
            height: None,
        }
    }

    #[test]
    fn scatter_produces_figure_with_scatter_glyph() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let spec = test_spec("Scatter");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();

        assert_eq!(fig.name, "Figure");
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            assert_eq!(arr.len(), 1);
            if let BokehValue::Object(r) = &arr[0] {
                assert_eq!(r.name, "GlyphRenderer");
                if let Some(BokehValue::Object(g)) = find_attr(r, "glyph") {
                    assert_eq!(g.name, "Scatter");
                }
            }
        } else {
            panic!("expected renderers array");
        }
    }

    #[test]
    fn scatter_default_color_and_marker() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let spec = test_spec("Defaults");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("#4C72B0"), "default color");
        assert!(json.contains("circle"), "default marker");
    }

    #[test]
    fn scatter_custom_color_marker_size_alpha() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("X").y_label("Y")
            .color("#ff0000")
            .marker(crate::charts::customization::marker::MarkerType::Square)
            .marker_size(15.0)
            .alpha(0.5)
            .build().unwrap();
        let spec = test_spec("Custom");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("#ff0000"));
        assert!(json.contains("square"));
        assert!(json.contains("15.0") || json.contains("15"));
    }

    #[test]
    fn scatter_with_filter_ref_embeds_filter_in_view() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let spec = test_spec("Filtered");
        let filter = BokehObject::new("BooleanFilter", "bf1".into())
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); 3]));
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, Some(filter.into_value()), None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("BooleanFilter"));
        assert!(json.contains("CDSView"));
    }

    #[test]
    fn scatter_without_filter_uses_all_indices() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let spec = test_spec("NoFilter");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("AllIndices"));
    }

    #[test]
    fn scatter_with_fixed_dimensions() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let mut spec = test_spec("Sized");
        spec.width = Some(800);
        spec.height = Some(600);
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("\"fixed\""));
        assert!(json.contains("800"));
        assert!(json.contains("600"));
    }

    #[test]
    fn scatter_has_hover_tool() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let spec = test_spec("Hover");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("HoverTool"));
    }

    #[test]
    fn scatter_cds_contains_data_columns() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let spec = test_spec("CDS");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("ColumnDataSource"));
        assert!(json.contains("\"x\""));
        assert!(json.contains("\"y\""));
    }

    #[test]
    fn scatter_nonselection_glyph_has_low_alpha() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let spec = test_spec("Nonsel");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            if let BokehValue::Object(r) = &arr[0] {
                if let Some(BokehValue::Object(ns)) = find_attr(r, "nonselection_glyph") {
                    assert_eq!(ns.name, "Scatter");
                    let ns_json = serde_json::to_string(&*ns).unwrap();
                    assert!(ns_json.contains("0.1"), "nonselection alpha should be 0.1");
                }
            }
        }
    }

    #[test]
    fn scatter_with_range_tool_x_range() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap();
        let spec = test_spec("RangeTool");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, Some("rt_range_1")).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("rt_range_1"));
    }

    #[test]
    fn scatter_axis_labels_applied() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = ScatterConfig::builder()
            .x("x").y("y").x_label("Revenue").y_label("Profit")
            .build().unwrap();
        let spec = test_spec("Labels");
        let fig = build_scatter(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("Revenue"));
        assert!(json.contains("Profit"));
    }
}
