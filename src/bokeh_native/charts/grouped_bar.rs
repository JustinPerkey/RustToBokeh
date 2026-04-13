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
use super::super::source::{get_f64_column, get_str_column};
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

    // Collect unique x categories (preserve order)
    let mut x_cats: Vec<String> = Vec::new();
    let mut seen_x = std::collections::HashSet::new();
    for x in &x_vals {
        if seen_x.insert(x.clone()) {
            x_cats.push(x.clone());
        }
    }

    // Collect unique groups (preserve order)
    let mut groups: Vec<String> = Vec::new();
    let mut seen_g = std::collections::HashSet::new();
    for g in &group_vals {
        if seen_g.insert(g.clone()) {
            groups.push(g.clone());
        }
    }

    let n_groups = groups.len();
    let colors = resolve_palette(cfg.palette.as_ref(), n_groups);

    // Build factor tuples: [x, group] for each row
    let factor_tuples: Vec<BokehValue> = x_vals
        .iter()
        .zip(group_vals.iter())
        .map(|(x, g)| {
            BokehValue::Array(vec![BokehValue::Str(x.clone()), BokehValue::Str(g.clone())])
        })
        .collect();

    // Pre-compute _fill_color per row (simpler than CategoricalColorMapper)
    let group_color_map: HashMap<&str, &str> = groups
        .iter()
        .enumerate()
        .map(|(i, g)| (g.as_str(), colors[i].as_str()))
        .collect();

    let fill_colors: Vec<BokehValue> = group_vals
        .iter()
        .map(|g| {
            let c = group_color_map.get(g.as_str()).copied().unwrap_or("#4C72B0");
            BokehValue::Str(c.to_string())
        })
        .collect();

    // FactorRange factors: all [x, group] combos in order
    let range_factors: Vec<BokehValue> = x_cats.iter().flat_map(|x| {
        groups.iter().map(move |g| {
            BokehValue::Array(vec![BokehValue::Str(x.clone()), BokehValue::Str(g.clone())])
        })
    }).collect();

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

    let bar_width = cfg.bar_width.unwrap_or(0.9);

    // Build column name for the factor tuples
    let factor_col = format!("_factors_{}_{}", cfg.x_col, cfg.group_col);

    // Build CDS manually with the extra columns
    let cds_id = id_gen.next();
    let sel_id = id_gen.next();
    let policy_id = id_gen.next();
    let data_entries: Vec<(String, BokehValue)> = vec![
        (cfg.x_col.clone(), BokehValue::Array(x_vals.iter().map(|s| BokehValue::Str(s.clone())).collect())),
        (cfg.group_col.clone(), BokehValue::Array(group_vals.iter().map(|s| BokehValue::Str(s.clone())).collect())),
        (cfg.value_col.clone(), BokehValue::Array(values.iter().map(|&v| BokehValue::Float(v)).collect())),
        (factor_col.clone(), BokehValue::Array(factor_tuples)),
        ("_fill_color".into(), BokehValue::Array(fill_colors)),
    ];

    let cds = BokehObject::new("ColumnDataSource", cds_id.clone())
        .attr(
            "selected",
            BokehObject::new("Selection", sel_id)
                .attr("indices", BokehValue::Array(vec![]))
                .attr("line_indices", BokehValue::Array(vec![]))
                .into_value(),
        )
        .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id).into_value())
        .attr("data", BokehValue::Map(data_entries));

    let glyph_id = id_gen.next();
    let glyph = BokehObject::new("VBar", glyph_id)
        .attr("x", BokehValue::field(&factor_col))
        .attr("top", BokehValue::field(&cfg.value_col))
        .attr("bottom", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("width", BokehValue::value_of(BokehValue::Float(bar_width)))
        .attr("fill_color", BokehValue::field("_fill_color"))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())));

    let nonsel_id = id_gen.next();
    let nonsel = BokehObject::new("VBar", nonsel_id)
        .attr("x", BokehValue::field(&factor_col))
        .attr("top", BokehValue::field(&cfg.value_col))
        .attr("bottom", BokehValue::value_of(BokehValue::Float(0.0)))
        .attr("width", BokehValue::value_of(BokehValue::Float(bar_width)))
        .attr("fill_color", BokehValue::field("_fill_color"))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())));

    let renderer = build_glyph_renderer(id_gen, cds.into_value(), glyph, Some(nonsel), filter_ref);
    add_renderers(&mut figure, vec![renderer]);

    // Legend (one item per group)
    let legend_items: Vec<BokehValue> = groups.iter().zip(colors.iter()).map(|(g, _color)| {
        // Build a tiny colored square as a legend marker using a dummy glyph
        let item_id = id_gen.next();
        BokehObject::new("LegendItem", item_id)
            .attr("label", BokehValue::value_of(BokehValue::Str(g.clone())))
            .into_value()
    }).collect();

    if !legend_items.is_empty() {
        let legend_id = id_gen.next();
        let legend = BokehObject::new("Legend", legend_id)
            .attr("items", BokehValue::Array(legend_items))
            .attr("location", BokehValue::Str("top_right".into()))
            .attr("click_policy", BokehValue::Str("hide".into()));
        add_legend(&mut figure, legend);
    }

    set_axis_labels(&mut figure, "", &cfg.y_label);
    Ok(figure)
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
