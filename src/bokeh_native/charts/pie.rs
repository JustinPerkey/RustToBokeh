//! Pie and donut chart builder.

use std::f64::consts::PI;
use polars::prelude::DataFrame;

use crate::charts::charts::pie::PieConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::palette::resolve_palette;
use super::super::source::{build_cds_from_entries, get_f64_column, get_str_column};
use super::{add_legend_panel, add_renderers, make_hover_tool};

pub fn build_pie(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &PieConfig,
    df: &DataFrame,
) -> Result<BokehObject, ChartError> {
    let labels = get_str_column(df, &cfg.label_col).map_err(ChartError::NativeRender)?;
    let values = get_f64_column(df, &cfg.value_col).map_err(ChartError::NativeRender)?;
    let n = labels.len();

    let colors = resolve_palette(cfg.palette.as_ref(), n);
    let show_legend = cfg.show_legend.unwrap_or(true);
    let (start_angles, end_angles) = compute_wedge_angles(&values);

    let ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &[cfg.label_col.as_str(), cfg.value_col.as_str()],
    );

    // Square data range; match_aspect keeps wedges circular while figure stretches to fill div.
    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        XRangeKind::Numeric { start: -1.2, end: 1.2 },
        YRangeKind::Numeric { start: -1.2, end: 1.2 },
        AxisBuilder::x(AxisType::Linear),
        AxisBuilder::y(AxisType::Linear),
        Some(ht),
    );
    // match_aspect locks plot-frame data axes to 1:1 in screen pixels, keeping
    // wedges circular regardless of the card aspect ratio.
    figure.attributes.push(("match_aspect".to_string(), BokehValue::Bool(true)));

    let cds = build_cds_from_entries(
        id_gen,
        vec![
            (cfg.label_col.clone(), BokehValue::Array(labels.iter().map(|s| BokehValue::Str(s.clone())).collect())),
            (cfg.value_col.clone(), BokehValue::Array(values.iter().map(|&v| BokehValue::Float(v)).collect())),
            ("start_angle".into(), BokehValue::Array(start_angles.iter().map(|&a| BokehValue::Float(a)).collect())),
            ("end_angle".into(), BokehValue::Array(end_angles.iter().map(|&a| BokehValue::Float(a)).collect())),
            ("color".into(), BokehValue::Array(colors.iter().map(|c| BokehValue::Str(c.clone())).collect())),
        ],
    );
    let cds_id = cds.id.clone();

    let inner_r = cfg.inner_radius.unwrap_or(0.0);
    let outer_r = 0.9_f64;

    let mut renderers: Vec<BokehObject> = Vec::new();
    let mut legend_items: Vec<BokehValue> = Vec::new();
    for i in 0..n {
        let cds_ref = if i == 0 { cds.clone().into_value() } else { BokehValue::ref_of(&cds_id) };
        let renderer = build_slice_renderer(id_gen, cds_ref, i, inner_r, outer_r);
        if show_legend {
            legend_items.push(build_slice_legend_item(id_gen, &labels[i], &renderer.id).into_value());
        }
        renderers.push(renderer);
    }
    add_renderers(&mut figure, renderers);

    if show_legend && !legend_items.is_empty() {
        let side = cfg.legend_side.as_deref().unwrap_or("right");
        let legend = BokehObject::new("Legend", id_gen.next())
            .attr("items", BokehValue::Array(legend_items))
            .attr("click_policy", BokehValue::Str("hide".into()))
            .attr("label_text_font_size", BokehValue::Str("10pt".into()));
        add_legend_panel(&mut figure, legend, side);
    }

    hide_axes(&mut figure);
    Ok(figure)
}

fn compute_wedge_angles(values: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let total: f64 = values.iter().sum();
    let mut starts = Vec::with_capacity(values.len());
    let mut ends = Vec::with_capacity(values.len());
    let mut cumulative = -PI / 2.0; // start from top (12 o'clock)
    for &v in values {
        let angle = if total > 0.0 { v / total * 2.0 * PI } else { 0.0 };
        starts.push(cumulative);
        ends.push(cumulative + angle);
        cumulative += angle;
    }
    (starts, ends)
}

fn build_slice_renderer(
    id_gen: &mut IdGen,
    cds_ref: BokehValue,
    slice_idx: usize,
    inner_r: f64,
    outer_r: f64,
) -> BokehObject {
    let filter = BokehObject::new("IndexFilter", id_gen.next())
        .attr("indices", BokehValue::Array(vec![BokehValue::Int(slice_idx as i64)]));

    let glyph = BokehObject::new("AnnularWedge", id_gen.next())
        .attr("x", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("y", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("outer_radius", BokehValue::value_of(BokehValue::Float(outer_r)))
        .attr("inner_radius", BokehValue::value_of(BokehValue::Float(inner_r)))
        .attr("start_angle", BokehValue::field("start_angle"))
        .attr("end_angle", BokehValue::field("end_angle"))
        .attr("fill_color", BokehValue::field("color"))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".into())));

    let nonsel = BokehObject::new("AnnularWedge", id_gen.next())
        .attr("x", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("y", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("outer_radius", BokehValue::value_of(BokehValue::Float(outer_r)))
        .attr("inner_radius", BokehValue::value_of(BokehValue::Float(inner_r)))
        .attr("start_angle", BokehValue::field("start_angle"))
        .attr("end_angle", BokehValue::field("end_angle"))
        .attr("fill_color", BokehValue::field("color"))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.3)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".into())));

    build_glyph_renderer(id_gen, cds_ref, glyph, Some(nonsel), Some(filter.into_value()))
}

fn build_slice_legend_item(id_gen: &mut IdGen, label: &str, renderer_id: &str) -> BokehObject {
    BokehObject::new("LegendItem", id_gen.next())
        .attr("label", BokehValue::value_of(BokehValue::Str(label.to_string())))
        .attr("renderers", BokehValue::Array(vec![BokehValue::ref_of(renderer_id)]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use crate::charts::{ChartConfig, ChartSpec, GridCell};

    fn test_df() -> DataFrame {
        df![
            "category" => ["A", "B", "C"],
            "amount"   => [30.0, 50.0, 20.0],
        ].unwrap()
    }

    fn test_spec(title: &str) -> ChartSpec {
        ChartSpec {
            title: title.into(),
            source_key: "test".into(),
            config: ChartConfig::Pie(
                PieConfig::builder().label("category").value("amount").build().unwrap(),
            ),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
            width: None,
            height: None,
        }
    }

    #[test]
    fn pie_produces_figure_with_annular_wedge_glyphs() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = PieConfig::builder().label("category").value("amount").build().unwrap();
        let spec = test_spec("Pie");
        let fig = build_pie(&mut id_gen, &spec, &cfg, &df).unwrap();

        assert_eq!(fig.name, "Figure");
        if let Some(BokehValue::Array(arr)) = find_attr_test(&fig, "renderers") {
            // 3 slices = 3 renderers
            assert_eq!(arr.len(), 3);
            for item in arr {
                if let BokehValue::Object(r) = item {
                    if let Some(BokehValue::Object(g)) = find_attr_test(r, "glyph") {
                        assert_eq!(g.name, "AnnularWedge");
                    }
                }
            }
        }
    }

    #[test]
    fn pie_has_legend_by_default() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = PieConfig::builder().label("category").value("amount").build().unwrap();
        let spec = test_spec("Legend");
        let fig = build_pie(&mut id_gen, &spec, &cfg, &df).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("Legend"));
        assert!(json.contains("LegendItem"));
    }

    #[test]
    fn pie_no_legend_when_disabled() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = PieConfig::builder()
            .label("category").value("amount")
            .show_legend(false)
            .build().unwrap();
        let spec = test_spec("NoLegend");
        let fig = build_pie(&mut id_gen, &spec, &cfg, &df).unwrap();
        // neither "right" nor "center" should contain a Legend
        for panel in &["right", "center"] {
            if let Some(BokehValue::Array(arr)) = find_attr_test(&fig, panel) {
                let has_legend = arr.iter().any(|v| {
                    if let BokehValue::Object(o) = v { o.name == "Legend" } else { false }
                });
                assert!(!has_legend, "legend should not be present in {}", panel);
            }
        }
    }

    #[test]
    fn pie_uses_numeric_range_for_axes() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = PieConfig::builder().label("category").value("amount").build().unwrap();
        let spec = test_spec("Range");
        let fig = build_pie(&mut id_gen, &spec, &cfg, &df).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("Range1d"));
        // Axes hidden
        assert!(json.contains("\"visible\":false"));
    }

    #[test]
    fn pie_donut_has_inner_radius() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = PieConfig::builder()
            .label("category").value("amount")
            .inner_radius(0.45)
            .build().unwrap();
        let spec = test_spec("Donut");
        let fig = build_pie(&mut id_gen, &spec, &cfg, &df).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("0.45"));
    }

    #[test]
    fn pie_each_slice_has_index_filter() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = PieConfig::builder().label("category").value("amount").build().unwrap();
        let spec = test_spec("IndexFilter");
        let fig = build_pie(&mut id_gen, &spec, &cfg, &df).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("IndexFilter"));
    }

    #[test]
    fn pie_legend_is_in_right_panel_not_center() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = PieConfig::builder().label("category").value("amount").build().unwrap();
        let spec = test_spec("PanelLegend");
        let fig = build_pie(&mut id_gen, &spec, &cfg, &df).unwrap();
        // legend must be in "right", not "center"
        let right_has_legend = find_attr_test(&fig, "right")
            .and_then(|v| if let BokehValue::Array(a) = v { Some(a) } else { None })
            .map(|arr| arr.iter().any(|v| matches!(v, BokehValue::Object(o) if o.name == "Legend")))
            .unwrap_or(false);
        assert!(right_has_legend, "legend should be in the right panel");
        let center_has_legend = find_attr_test(&fig, "center")
            .and_then(|v| if let BokehValue::Array(a) = v { Some(a) } else { None })
            .map(|arr| arr.iter().any(|v| matches!(v, BokehValue::Object(o) if o.name == "Legend")))
            .unwrap_or(false);
        assert!(!center_has_legend, "legend must not be in center");
    }

    #[test]
    fn pie_uses_match_aspect_to_keep_wedges_circular() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = PieConfig::builder().label("category").value("amount").build().unwrap();
        let spec = test_spec("MatchAspect");
        let fig = build_pie(&mut id_gen, &spec, &cfg, &df).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("match_aspect"), "match_aspect needed to keep wedges circular");
        assert!(json.contains("stretch_width"), "pie should stretch to fill card width");
    }
}

#[cfg(test)]
fn find_attr_test<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
    obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn hide_axes(fig: &mut BokehObject) {
    for (key, val) in &mut fig.attributes {
        if key == "below" || key == "left" {
            if let BokehValue::Array(axes) = val {
                for ax in axes.iter_mut() {
                    if let BokehValue::Object(obj) = ax {
                        obj.attributes.push(("visible".to_string(), BokehValue::Bool(false)));
                    }
                }
            }
        }
        if key == "center" {
            if let BokehValue::Array(items) = val {
                for item in items.iter_mut() {
                    if let BokehValue::Object(obj) = item {
                        if obj.name == "Grid" {
                            obj.attributes.push(("grid_line_color".to_string(), BokehValue::Null));
                        }
                    }
                }
            }
        }
    }
}
