//! Box plot chart builder.

use polars::prelude::DataFrame;

use crate::charts::charts::box_plot::BoxPlotConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::palette::resolve_palette;
use super::super::source::{get_f64_column, get_str_column};
use super::{add_renderers, make_hover_tool, set_axis_labels};

pub fn build_box_plot(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &BoxPlotConfig,
    df: &DataFrame,
    outlier_df: Option<&DataFrame>,
    filter_ref: Option<BokehValue>,
) -> Result<BokehObject, ChartError> {
    let categories = get_str_column(df, &cfg.category_col).map_err(ChartError::NativeRender)?;
    let q1_vals = get_f64_column(df, &cfg.q1_col).map_err(ChartError::NativeRender)?;
    let q2_vals = get_f64_column(df, &cfg.q2_col).map_err(ChartError::NativeRender)?;
    let q3_vals = get_f64_column(df, &cfg.q3_col).map_err(ChartError::NativeRender)?;
    let lower_vals = get_f64_column(df, &cfg.lower_col).map_err(ChartError::NativeRender)?;
    let upper_vals = get_f64_column(df, &cfg.upper_col).map_err(ChartError::NativeRender)?;

    let n = categories.len();
    let colors = if let Some(palette) = &cfg.palette {
        resolve_palette(Some(palette), n)
    } else if let Some(col) = &cfg.color {
        vec![col.clone(); n]
    } else {
        resolve_palette(None, n)
    };
    let alpha = cfg.alpha.unwrap_or(0.7);

    // FactorRange from categories
    let factors: Vec<BokehValue> = categories.iter().map(|s| BokehValue::Str(s.clone())).collect();

    let ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &[cfg.category_col.as_str(), cfg.q1_col.as_str(), cfg.q2_col.as_str(), cfg.q3_col.as_str()],
    );

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        XRangeKind::Factor(factors),
        YRangeKind::DataRange,
        AxisBuilder::x(AxisType::Categorical),
        AxisBuilder::y(AxisType::Linear).config(cfg.y_axis.as_ref()),
        Some(ht),
    );

    const CAP_HW: f64 = 0.3;
    const BOX_W: f64 = 0.6;

    // --- Upper whisker stem: [cat, q3] → [cat, upper]
    let _upper_x0: Vec<BokehValue> = categories.iter().map(|c| BokehValue::Str(c.clone())).collect();
    let upper_y0: Vec<BokehValue> = q3_vals.iter().map(|&v| BokehValue::Float(v)).collect();
    let upper_y1: Vec<BokehValue> = upper_vals.iter().map(|&v| BokehValue::Float(v)).collect();

    // --- Lower whisker stem: [cat, lower] → [cat, q1]
    let lower_y0: Vec<BokehValue> = lower_vals.iter().map(|&v| BokehValue::Float(v)).collect();
    let lower_y1: Vec<BokehValue> = q1_vals.iter().map(|&v| BokehValue::Float(v)).collect();

    // --- Upper whisker cap: (cat, -(CAP_HW)) → (cat, +(CAP_HW))
    // In FactorRange: x offset tuples [category, offset_fraction]
    let cap_upper_x0: Vec<BokehValue> = categories.iter()
        .map(|c| BokehValue::Array(vec![BokehValue::Str(c.clone()), BokehValue::Float(-CAP_HW)]))
        .collect();
    let cap_upper_x1: Vec<BokehValue> = categories.iter()
        .map(|c| BokehValue::Array(vec![BokehValue::Str(c.clone()), BokehValue::Float(CAP_HW)]))
        .collect();
    let cap_lower_x0 = cap_upper_x0.clone();
    let cap_lower_x1 = cap_upper_x1.clone();

    // Build CDS for whiskers (segment glyphs)
    let whisker_cds = build_whisker_cds(
        id_gen,
        &categories,
        &upper_y0, &upper_y1,
        &lower_y0, &lower_y1,
        &upper_vals,
        &lower_vals,
        cap_upper_x0, cap_upper_x1,
        cap_lower_x0, cap_lower_x1,
    );
    let whisker_cds_id = whisker_cds.id.clone();

    // Build CDS for boxes (VBar glyph)
    let box_colors: Vec<BokehValue> = colors.iter().map(|c| BokehValue::Str(c.clone())).collect();
    let box_cds_id = id_gen.next();
    let sel_id = id_gen.next();
    let policy_id = id_gen.next();
    let box_cds = BokehObject::new("ColumnDataSource", box_cds_id.clone())
        .attr(
            "selected",
            BokehObject::new("Selection", sel_id)
                .attr("indices", BokehValue::Array(vec![]))
                .attr("line_indices", BokehValue::Array(vec![]))
                .into_value(),
        )
        .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id).into_value())
        .attr("data", BokehValue::Map(vec![
            (cfg.category_col.clone(), BokehValue::Array(categories.iter().map(|s| BokehValue::Str(s.clone())).collect())),
            ("q1".into(), BokehValue::Array(q1_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
            ("q2".into(), BokehValue::Array(q2_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
            ("q3".into(), BokehValue::Array(q3_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
            ("lower".into(), BokehValue::Array(lower_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
            ("upper".into(), BokehValue::Array(upper_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
            ("_fill_color".into(), BokehValue::Array(box_colors)),
        ]));

    // Upper whisker segment
    let u_whisker = build_segment_glyph(id_gen, "x", "upper_y0", "x", "upper_y1", "#666666");
    let u_nonsel = build_segment_glyph_nonsel(id_gen, "x", "upper_y0", "x", "upper_y1");
    let u_renderer = build_glyph_renderer(id_gen, whisker_cds.clone().into_value(), u_whisker, Some(u_nonsel), filter_ref.clone());

    // Lower whisker segment
    let l_whisker = build_segment_glyph(id_gen, "x", "lower_y0", "x", "lower_y1", "#666666");
    let l_nonsel = build_segment_glyph_nonsel(id_gen, "x", "lower_y0", "x", "lower_y1");
    let l_renderer = build_glyph_renderer(id_gen, BokehValue::ref_of(&whisker_cds_id), l_whisker, Some(l_nonsel), filter_ref.clone());

    // Upper cap segment
    let uc_whisker = build_segment_glyph(id_gen, "cap_upper_x0", "upper_val", "cap_upper_x1", "upper_val", "#666666");
    let uc_nonsel = build_segment_glyph_nonsel(id_gen, "cap_upper_x0", "upper_val", "cap_upper_x1", "upper_val");
    let uc_renderer = build_glyph_renderer(id_gen, BokehValue::ref_of(&whisker_cds_id), uc_whisker, Some(uc_nonsel), filter_ref.clone());

    // Lower cap segment
    let lc_whisker = build_segment_glyph(id_gen, "cap_lower_x0", "lower_val", "cap_lower_x1", "lower_val", "#666666");
    let lc_nonsel = build_segment_glyph_nonsel(id_gen, "cap_lower_x0", "lower_val", "cap_lower_x1", "lower_val");
    let lc_renderer = build_glyph_renderer(id_gen, BokehValue::ref_of(&whisker_cds_id), lc_whisker, Some(lc_nonsel), filter_ref.clone());

    // IQR box (VBar from q1 to q3)
    let box_glyph_id = id_gen.next();
    let box_glyph = BokehObject::new("VBar", box_glyph_id)
        .attr("x", BokehValue::field(&cfg.category_col))
        .attr("top", BokehValue::field("q3"))
        .attr("bottom", BokehValue::field("q1"))
        .attr("width", BokehValue::value_of(BokehValue::Float(BOX_W)))
        .attr("fill_color", BokehValue::field("_fill_color"))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(alpha)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("#333333".into())));

    let box_nonsel_id = id_gen.next();
    let box_nonsel = BokehObject::new("VBar", box_nonsel_id)
        .attr("x", BokehValue::field(&cfg.category_col))
        .attr("top", BokehValue::field("q3"))
        .attr("bottom", BokehValue::field("q1"))
        .attr("width", BokehValue::value_of(BokehValue::Float(BOX_W)))
        .attr("fill_color", BokehValue::field("_fill_color"))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)));

    let box_renderer = build_glyph_renderer(id_gen, box_cds.into_value(), box_glyph, Some(box_nonsel), filter_ref.clone());

    // Median segment (horizontal line at q2)
    let median_x0: Vec<BokehValue> = categories.iter()
        .map(|c| BokehValue::Array(vec![BokehValue::Str(c.clone()), BokehValue::Float(-BOX_W / 2.0)]))
        .collect();
    let median_x1: Vec<BokehValue> = categories.iter()
        .map(|c| BokehValue::Array(vec![BokehValue::Str(c.clone()), BokehValue::Float(BOX_W / 2.0)]))
        .collect();

    let median_cds_id = id_gen.next();
    let sel_id2 = id_gen.next();
    let policy_id2 = id_gen.next();
    let median_cds = BokehObject::new("ColumnDataSource", median_cds_id.clone())
        .attr(
            "selected",
            BokehObject::new("Selection", sel_id2)
                .attr("indices", BokehValue::Array(vec![]))
                .attr("line_indices", BokehValue::Array(vec![]))
                .into_value(),
        )
        .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id2).into_value())
        .attr("data", BokehValue::Map(vec![
            ("median_x0".into(), BokehValue::Array(median_x0)),
            ("median_x1".into(), BokehValue::Array(median_x1)),
            ("median_y".into(), BokehValue::Array(q2_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
        ]));

    let med_glyph = build_segment_glyph(id_gen, "median_x0", "median_y", "median_x1", "median_y", "#333333");
    let med_nonsel = build_segment_glyph_nonsel(id_gen, "median_x0", "median_y", "median_x1", "median_y");
    let med_renderer = build_glyph_renderer(id_gen, median_cds.into_value(), med_glyph, Some(med_nonsel), filter_ref.clone());

    add_renderers(&mut figure, vec![u_renderer, l_renderer, uc_renderer, lc_renderer, box_renderer, med_renderer]);

    // Outliers
    if let Some(outlier_df) = outlier_df {
        if let (Ok(out_cats), Ok(out_vals)) = (
            get_str_column(outlier_df, &cfg.category_col),
            get_f64_column(outlier_df, cfg.outlier_value_col.as_deref().unwrap_or("value")),
        ) {
            let out_cds_id = id_gen.next();
            let sel_id3 = id_gen.next();
            let policy_id3 = id_gen.next();
            let out_cds = BokehObject::new("ColumnDataSource", out_cds_id)
                .attr(
                    "selected",
                    BokehObject::new("Selection", sel_id3)
                        .attr("indices", BokehValue::Array(vec![]))
                        .attr("line_indices", BokehValue::Array(vec![]))
                        .into_value(),
                )
                .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id3).into_value())
                .attr("data", BokehValue::Map(vec![
                    (cfg.category_col.clone(), BokehValue::Array(out_cats.iter().map(|s| BokehValue::Str(s.clone())).collect())),
                    ("value".into(), BokehValue::Array(out_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
                ]));

            let out_glyph_id = id_gen.next();
            let out_glyph = BokehObject::new("Scatter", out_glyph_id)
                .attr("x", BokehValue::field(&cfg.category_col))
                .attr("y", BokehValue::field("value"))
                .attr("size", BokehValue::value_of(BokehValue::Float(5.0)))
                .attr("fill_color", BokehValue::value_of(BokehValue::Str("#666666".into())))
                .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.6)))
                .attr("line_color", BokehValue::value_of(BokehValue::Null))
                .attr("marker", BokehValue::value_of(BokehValue::Str("circle".into())));

            let out_nonsel_id = id_gen.next();
            let out_nonsel = BokehObject::new("Scatter", out_nonsel_id)
                .attr("x", BokehValue::field(&cfg.category_col))
                .attr("y", BokehValue::field("value"))
                .attr("size", BokehValue::value_of(BokehValue::Float(5.0)))
                .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
                .attr("marker", BokehValue::value_of(BokehValue::Str("circle".into())));

            let out_renderer = build_glyph_renderer(id_gen, out_cds.into_value(), out_glyph, Some(out_nonsel), None);
            add_renderers(&mut figure, vec![out_renderer]);
        }
    }

    set_axis_labels(&mut figure, "", &cfg.y_label);
    Ok(figure)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use crate::charts::{ChartConfig, ChartSpec, GridCell};
    use crate::charts::charts::box_plot::BoxPlotConfig;

    fn stats_df() -> DataFrame {
        df![
            "category" => ["Eng", "Sales"],
            "q1"       => [60.0, 50.0],
            "q2"       => [75.0, 65.0],
            "q3"       => [90.0, 80.0],
            "lower"    => [45.0, 35.0],
            "upper"    => [110.0, 100.0],
        ].unwrap()
    }

    fn outlier_df() -> DataFrame {
        df![
            "category" => ["Eng", "Sales"],
            "value"    => [130.0, 20.0],
        ].unwrap()
    }

    fn test_spec(title: &str) -> ChartSpec {
        ChartSpec {
            title: title.into(),
            source_key: "test".into(),
            config: ChartConfig::BoxPlot(
                BoxPlotConfig::builder()
                    .category("category").q1("q1").q2("q2").q3("q3")
                    .lower("lower").upper("upper").y_label("Value")
                    .build().unwrap(),
            ),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
            width: None,
            height: None,
        }
    }

    #[test]
    fn box_plot_produces_figure_with_multiple_renderers() {
        let df = stats_df();
        let mut id_gen = IdGen::new();
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Val")
            .build().unwrap();
        let spec = test_spec("Box");
        let fig = build_box_plot(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();

        assert_eq!(fig.name, "Figure");
        if let Some(BokehValue::Array(arr)) = find_attr_test(&fig, "renderers") {
            // upper whisker + lower whisker + upper cap + lower cap + box + median = 6
            assert_eq!(arr.len(), 6);
        }
    }

    #[test]
    fn box_plot_has_vbar_glyph_for_box() {
        let df = stats_df();
        let mut id_gen = IdGen::new();
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Val")
            .build().unwrap();
        let spec = test_spec("VBar");
        let fig = build_box_plot(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("VBar"));
        assert!(json.contains("Segment"));
    }

    #[test]
    fn box_plot_uses_factor_range() {
        let df = stats_df();
        let mut id_gen = IdGen::new();
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Val")
            .build().unwrap();
        let spec = test_spec("Factors");
        let fig = build_box_plot(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("FactorRange"));
    }

    #[test]
    fn box_plot_with_outliers_adds_scatter_renderer() {
        let df = stats_df();
        let out_df = outlier_df();
        let mut id_gen = IdGen::new();
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Val")
            .build().unwrap();
        let spec = test_spec("Outliers");
        let fig = build_box_plot(&mut id_gen, &spec, &cfg, &df, Some(&out_df), None).unwrap();

        if let Some(BokehValue::Array(arr)) = find_attr_test(&fig, "renderers") {
            // 6 base + 1 outlier scatter = 7
            assert_eq!(arr.len(), 7);
        }
        let json = serde_json::to_string(&fig).unwrap();
        // Outlier renderer uses Scatter glyph
        assert!(json.contains("\"Scatter\"") || json.contains("\"name\":\"Scatter\""));
    }

    #[test]
    fn box_plot_with_filter_ref() {
        let df = stats_df();
        let mut id_gen = IdGen::new();
        let cfg = BoxPlotConfig::builder()
            .category("category").q1("q1").q2("q2").q3("q3")
            .lower("lower").upper("upper").y_label("Val")
            .build().unwrap();
        let spec = test_spec("Filtered");
        let filter = BokehObject::new("BooleanFilter", "bf1".into())
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); 2]));
        let fig = build_box_plot(&mut id_gen, &spec, &cfg, &df, None, Some(filter.into_value())).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("BooleanFilter"));
    }
}

fn build_whisker_cds(
    id_gen: &mut IdGen,
    categories: &[String],
    upper_y0: &[BokehValue],
    upper_y1: &[BokehValue],
    lower_y0: &[BokehValue],
    lower_y1: &[BokehValue],
    upper_vals: &[f64],
    lower_vals: &[f64],
    cap_upper_x0: Vec<BokehValue>,
    cap_upper_x1: Vec<BokehValue>,
    cap_lower_x0: Vec<BokehValue>,
    cap_lower_x1: Vec<BokehValue>,
) -> BokehObject {
    let cds_id = id_gen.next();
    let sel_id = id_gen.next();
    let policy_id = id_gen.next();
    BokehObject::new("ColumnDataSource", cds_id)
        .attr(
            "selected",
            BokehObject::new("Selection", sel_id)
                .attr("indices", BokehValue::Array(vec![]))
                .attr("line_indices", BokehValue::Array(vec![]))
                .into_value(),
        )
        .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id).into_value())
        .attr("data", BokehValue::Map(vec![
            ("x".into(), BokehValue::Array(categories.iter().map(|s| BokehValue::Str(s.clone())).collect())),
            ("upper_y0".into(), BokehValue::Array(upper_y0.to_vec())),
            ("upper_y1".into(), BokehValue::Array(upper_y1.to_vec())),
            ("lower_y0".into(), BokehValue::Array(lower_y0.to_vec())),
            ("lower_y1".into(), BokehValue::Array(lower_y1.to_vec())),
            ("upper_val".into(), BokehValue::Array(upper_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
            ("lower_val".into(), BokehValue::Array(lower_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
            ("cap_upper_x0".into(), BokehValue::Array(cap_upper_x0)),
            ("cap_upper_x1".into(), BokehValue::Array(cap_upper_x1)),
            ("cap_lower_x0".into(), BokehValue::Array(cap_lower_x0)),
            ("cap_lower_x1".into(), BokehValue::Array(cap_lower_x1)),
        ]))
}

fn build_segment_glyph(id_gen: &mut IdGen, x0: &str, y0: &str, x1: &str, y1: &str, color: &str) -> BokehObject {
    BokehObject::new("Segment", id_gen.next())
        .attr("x0", BokehValue::field(x0))
        .attr("y0", BokehValue::field(y0))
        .attr("x1", BokehValue::field(x1))
        .attr("y1", BokehValue::field(y1))
        .attr("line_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("line_width", BokehValue::value_of(BokehValue::Float(1.5)))
}

#[cfg(test)]
fn find_attr_test<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
    obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn build_segment_glyph_nonsel(id_gen: &mut IdGen, x0: &str, y0: &str, x1: &str, y1: &str) -> BokehObject {
    BokehObject::new("Segment", id_gen.next())
        .attr("x0", BokehValue::field(x0))
        .attr("y0", BokehValue::field(y0))
        .attr("x1", BokehValue::field(x1))
        .attr("y1", BokehValue::field(y1))
        .attr("line_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
}
