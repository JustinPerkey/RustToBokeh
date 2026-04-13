//! Horizontal bar chart builder.

use polars::prelude::DataFrame;

use crate::charts::charts::HBarConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::source::{build_column_data_source, get_str_column};
use super::{add_renderers, make_hover_tool, set_axis_labels};

pub fn build_hbar(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &HBarConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
) -> Result<BokehObject, ChartError> {
    let categories = get_str_column(df, &cfg.category_col)
        .map_err(ChartError::NativeRender)?;

    // Categories go on the y-axis (reversed so top category is first)
    let factor_values: Vec<BokehValue> = categories
        .iter()
        .rev()
        .map(|s| BokehValue::Str(s.clone()))
        .collect();

    let ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &[cfg.category_col.as_str(), cfg.value_col.as_str()],
    );

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        XRangeKind::DataRange,
        YRangeKind::Factor(factor_values),
        AxisBuilder::x(AxisType::Linear).config(cfg.x_axis.as_ref()),
        AxisBuilder::y(AxisType::Categorical).config(cfg.y_axis.as_ref()),
        Some(ht),
    );

    let color = cfg.color.as_deref().unwrap_or("#4C72B0");

    // Build CDS
    let cds = build_column_data_source(id_gen, df);
    let cds_id = cds.id.clone();
    let cds_ref = cds.into_value();

    // HBar glyph
    let glyph_id = id_gen.next();
    let glyph = BokehObject::new("HBar", glyph_id)
        .attr("y", BokehValue::field(&cfg.category_col))
        .attr("right", BokehValue::field(&cfg.value_col))
        .attr("left", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("height", BokehValue::value_of(BokehValue::Float(0.7)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())));

    let nonsel_id = id_gen.next();
    let nonsel = BokehObject::new("HBar", nonsel_id)
        .attr("y", BokehValue::field(&cfg.category_col))
        .attr("right", BokehValue::field(&cfg.value_col))
        .attr("left", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("height", BokehValue::value_of(BokehValue::Float(0.7)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())));

    let renderer = build_glyph_renderer(id_gen, cds_ref, glyph, Some(nonsel), filter_ref);
    add_renderers(&mut figure, vec![renderer]);
    set_axis_labels(&mut figure, &cfg.x_label, "");

    // Apply grid config for x axis (show_grid false)
    if let Some(ax_cfg) = &cfg.x_axis {
        if !ax_cfg.show_grid {
            hide_x_grid(&mut figure);
        }
    }

    let _ = cds_id; // suppress unused warning
    Ok(figure)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use crate::charts::{ChartConfig, ChartSpec, GridCell};

    fn test_df() -> DataFrame {
        df!["dept" => ["Eng", "Sales", "HR"], "headcount" => [50.0, 30.0, 20.0]].unwrap()
    }

    fn test_spec(title: &str) -> ChartSpec {
        ChartSpec {
            title: title.into(),
            source_key: "test".into(),
            config: ChartConfig::HBar(
                HBarConfig::builder().category("dept").value("headcount").x_label("Count").build().unwrap(),
            ),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
            width: None,
            height: None,
        }
    }

    #[test]
    fn hbar_produces_figure_with_hbar_glyph() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = HBarConfig::builder().category("dept").value("headcount").x_label("Count").build().unwrap();
        let spec = test_spec("HBar");
        let fig = build_hbar(&mut id_gen, &spec, &cfg, &df, None).unwrap();

        assert_eq!(fig.name, "Figure");
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            assert_eq!(arr.len(), 1);
            if let BokehValue::Object(r) = &arr[0] {
                if let Some(BokehValue::Object(g)) = find_attr(r, "glyph") {
                    assert_eq!(g.name, "HBar");
                }
            }
        }
    }

    #[test]
    fn hbar_uses_factor_range_for_y_axis() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = HBarConfig::builder().category("dept").value("headcount").x_label("X").build().unwrap();
        let spec = test_spec("FactorY");
        let fig = build_hbar(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("FactorRange"));
        // Reversed categories
        assert!(json.contains("HR"));
        assert!(json.contains("Sales"));
        assert!(json.contains("Eng"));
    }

    #[test]
    fn hbar_custom_color() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = HBarConfig::builder()
            .category("dept").value("headcount").x_label("X")
            .color("#e74c3c")
            .build().unwrap();
        let spec = test_spec("Color");
        let fig = build_hbar(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("#e74c3c"));
    }

    #[test]
    fn hbar_with_filter_ref() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = HBarConfig::builder().category("dept").value("headcount").x_label("X").build().unwrap();
        let spec = test_spec("Filtered");
        let filter = BokehObject::new("BooleanFilter", "bf1".into())
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); 3]));
        let fig = build_hbar(&mut id_gen, &spec, &cfg, &df, Some(filter.into_value())).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("BooleanFilter"));
    }

    #[test]
    fn hbar_hide_x_grid_when_configured() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = HBarConfig::builder()
            .category("dept").value("headcount").x_label("X")
            .x_axis(crate::charts::AxisConfig::builder().show_grid(false).build())
            .build().unwrap();
        let spec = test_spec("NoGrid");
        let fig = build_hbar(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("grid_line_color"));
    }
}

#[cfg(test)]
fn find_attr<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
    obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn hide_x_grid(fig: &mut BokehObject) {
    for (key, val) in &mut fig.attributes {
        if key == "center" {
            if let BokehValue::Array(items) = val {
                for item in items.iter_mut() {
                    if let BokehValue::Object(obj) = item {
                        if obj.name == "Grid" {
                            let is_x_grid = obj.attributes.iter().any(|(k, v)| {
                                k == "dimension" && matches!(v, BokehValue::Int(0))
                            });
                            if is_x_grid {
                                obj.attributes.push(("grid_line_color".to_string(), BokehValue::Null));
                            }
                        }
                    }
                }
            }
        }
    }
}
