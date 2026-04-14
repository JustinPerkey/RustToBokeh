//! Grouped vertical bar chart builder.

use std::collections::HashMap;
use polars::prelude::DataFrame;

use crate::charts::charts::grouped_bar::GroupedBarConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::palette::resolve_palette;
use super::super::source::{build_cds_from_entries, get_f64_column, get_str_column};
use super::{add_legend, add_renderers, make_hover_tool, set_axis_labels};

pub fn build_grouped_bar(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &GroupedBarConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
) -> Result<BokehObject, ChartError> {
    let x_vals = get_str_column(df, &cfg.x_col).map_err(ChartError::NativeRender)?;
    let group_vals = get_str_column(df, &cfg.group_col).map_err(ChartError::NativeRender)?;
    let values = get_f64_column(df, &cfg.value_col).map_err(ChartError::NativeRender)?;

    let x_cats = unique_preserve_order(&x_vals);
    let groups = unique_preserve_order(&group_vals);
    let colors = resolve_palette(cfg.palette.as_ref(), groups.len());

    let group_color_map: HashMap<&str, &str> = groups
        .iter()
        .enumerate()
        .map(|(i, g)| (g.as_str(), colors[i].as_str()))
        .collect();

    let fill_colors: Vec<BokehValue> = group_vals
        .iter()
        .map(|g| BokehValue::Str(group_color_map.get(g.as_str()).copied().unwrap_or("#4C72B0").to_string()))
        .collect();

    let factor_tuples: Vec<BokehValue> = x_vals
        .iter()
        .zip(group_vals.iter())
        .map(|(x, g)| BokehValue::Array(vec![BokehValue::Str(x.clone()), BokehValue::Str(g.clone())]))
        .collect();

    // Range factors: all [x, group] combos in order
    let range_factors: Vec<BokehValue> = x_cats
        .iter()
        .flat_map(|x| {
            groups.iter().map(move |g| {
                BokehValue::Array(vec![BokehValue::Str(x.clone()), BokehValue::Str(g.clone())])
            })
        })
        .collect();

    let ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &[cfg.x_col.as_str(), cfg.group_col.as_str(), cfg.value_col.as_str()],
    );

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        XRangeKind::Factor(range_factors),
        YRangeKind::DataRange,
        AxisBuilder::x(AxisType::Categorical).config(cfg.x_axis.as_ref()),
        AxisBuilder::y(AxisType::Linear).config(cfg.y_axis.as_ref()),
        Some(ht),
    );

    let factor_col = format!("_factors_{}_{}", cfg.x_col, cfg.group_col);
    let cds = build_cds_from_entries(
        id_gen,
        vec![
            (cfg.x_col.clone(), BokehValue::Array(x_vals.iter().map(|s| BokehValue::Str(s.clone())).collect())),
            (cfg.group_col.clone(), BokehValue::Array(group_vals.iter().map(|s| BokehValue::Str(s.clone())).collect())),
            (cfg.value_col.clone(), BokehValue::Array(values.iter().map(|&v| BokehValue::Float(v)).collect())),
            (factor_col.clone(), BokehValue::Array(factor_tuples)),
            ("_fill_color".into(), BokehValue::Array(fill_colors)),
        ],
    );

    let bar_width = cfg.bar_width.unwrap_or(0.9);
    let renderer = build_bar_renderer(id_gen, cds, &factor_col, &cfg.value_col, bar_width, filter_ref);
    add_renderers(&mut figure, vec![renderer]);

    add_group_legend(id_gen, &mut figure, &groups);

    set_axis_labels(&mut figure, "", &cfg.y_label);
    Ok(figure)
}

fn unique_preserve_order(vals: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for v in vals {
        if seen.insert(v.clone()) {
            out.push(v.clone());
        }
    }
    out
}

fn build_bar_renderer(
    id_gen: &mut IdGen,
    cds: BokehObject,
    factor_col: &str,
    value_col: &str,
    bar_width: f64,
    filter_ref: Option<BokehValue>,
) -> BokehObject {
    let glyph = BokehObject::new("VBar", id_gen.next())
        .attr("x", BokehValue::field(factor_col))
        .attr("top", BokehValue::field(value_col))
        .attr("bottom", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("width", BokehValue::value_of(BokehValue::Float(bar_width)))
        .attr("fill_color", BokehValue::field("_fill_color"))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())));

    let nonsel = BokehObject::new("VBar", id_gen.next())
        .attr("x", BokehValue::field(factor_col))
        .attr("top", BokehValue::field(value_col))
        .attr("bottom", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("width", BokehValue::value_of(BokehValue::Float(bar_width)))
        .attr("fill_color", BokehValue::field("_fill_color"))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())));

    build_glyph_renderer(id_gen, cds.into_value(), glyph, Some(nonsel), filter_ref)
}

fn add_group_legend(id_gen: &mut IdGen, figure: &mut BokehObject, groups: &[String]) {
    if groups.is_empty() {
        return;
    }
    let items: Vec<BokehValue> = groups
        .iter()
        .map(|g| {
            BokehObject::new("LegendItem", id_gen.next())
                .attr("label", BokehValue::value_of(BokehValue::Str(g.clone())))
                .into_value()
        })
        .collect();

    let legend = BokehObject::new("Legend", id_gen.next())
        .attr("items", BokehValue::Array(items))
        .attr("location", BokehValue::Str("top_right".into()))
        .attr("click_policy", BokehValue::Str("hide".into()));
    add_legend(figure, legend);
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use crate::charts::{ChartConfig, ChartSpec, GridCell};

    fn find_attr<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
        obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn test_df() -> DataFrame {
        df![
            "quarter" => ["Q1", "Q1", "Q2", "Q2"],
            "product" => ["A", "B", "A", "B"],
            "revenue" => [100.0, 80.0, 120.0, 90.0],
        ].unwrap()
    }

    fn test_spec(title: &str) -> ChartSpec {
        ChartSpec {
            title: title.into(),
            source_key: "test".into(),
            config: ChartConfig::GroupedBar(
                GroupedBarConfig::builder()
                    .x("quarter").group("product").value("revenue").y_label("Revenue")
                    .build().unwrap(),
            ),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
            width: None,
            height: None,
        }
    }

    #[test]
    fn grouped_bar_produces_figure_with_vbar_glyph() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = GroupedBarConfig::builder()
            .x("quarter").group("product").value("revenue").y_label("Rev")
            .build().unwrap();
        let spec = test_spec("Grouped");
        let fig = build_grouped_bar(&mut id_gen, &spec, &cfg, &df, None).unwrap();

        assert_eq!(fig.name, "Figure");
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            assert_eq!(arr.len(), 1);
            if let BokehValue::Object(r) = &arr[0] {
                if let Some(BokehValue::Object(g)) = find_attr(r, "glyph") {
                    assert_eq!(g.name, "VBar");
                }
            }
        }
    }

    #[test]
    fn grouped_bar_uses_factor_range_with_tuples() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = GroupedBarConfig::builder()
            .x("quarter").group("product").value("revenue").y_label("Rev")
            .build().unwrap();
        let spec = test_spec("Factors");
        let fig = build_grouped_bar(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("FactorRange"));
        assert!(json.contains("Q1"));
        assert!(json.contains("Q2"));
    }

    #[test]
    fn grouped_bar_has_legend() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = GroupedBarConfig::builder()
            .x("quarter").group("product").value("revenue").y_label("Rev")
            .build().unwrap();
        let spec = test_spec("Legend");
        let fig = build_grouped_bar(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("Legend"));
        assert!(json.contains("LegendItem"));
    }

    #[test]
    fn grouped_bar_cds_has_fill_color_column() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = GroupedBarConfig::builder()
            .x("quarter").group("product").value("revenue").y_label("Rev")
            .build().unwrap();
        let spec = test_spec("FillColor");
        let fig = build_grouped_bar(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("_fill_color"));
    }

    #[test]
    fn grouped_bar_with_filter_ref() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = GroupedBarConfig::builder()
            .x("quarter").group("product").value("revenue").y_label("Rev")
            .build().unwrap();
        let spec = test_spec("Filtered");
        let filter = BokehObject::new("BooleanFilter", "bf1".into())
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); 4]));
        let fig = build_grouped_bar(&mut id_gen, &spec, &cfg, &df, Some(filter.into_value())).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("BooleanFilter"));
    }
}
