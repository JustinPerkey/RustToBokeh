//! Box plot chart builder.

use polars::prelude::DataFrame;

use crate::charts::charts::box_plot::BoxPlotConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, build_hover_tool, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::palette::resolve_palette;
use super::super::source::{build_cds_from_entries, get_f64_column, get_str_column};
use super::{add_renderers, make_hover_tool, set_axis_labels};

const CAP_HW: f64 = 0.3;
const BOX_W: f64 = 0.6;

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

    // Build CDS and renderers first so we know the box renderer ID, allowing
    // the HoverTool to be scoped to that renderer only (avoids "???" on whisker
    // and median CDSes whose columns differ from the tooltip fields).
    let whisker_cds = build_whisker_cds(id_gen, &categories, &q1_vals, &q3_vals, &lower_vals, &upper_vals);
    let whisker_cds_id = whisker_cds.id.clone();
    let box_cds = build_box_cds(id_gen, &cfg.category_col, &categories, &q1_vals, &q2_vals, &q3_vals, &lower_vals, &upper_vals, &colors);

    let whisker_rs = build_whisker_renderers(id_gen, whisker_cds, &whisker_cds_id, filter_ref.clone());
    let box_renderer = build_box_renderer(id_gen, box_cds, &cfg.category_col, alpha, filter_ref.clone());
    let box_renderer_id = box_renderer.id.clone();
    let med_renderer = build_median_renderer(id_gen, &categories, &q2_vals, filter_ref.clone());

    let mut ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &[cfg.category_col.as_str(), cfg.q1_col.as_str(), cfg.q2_col.as_str(), cfg.q3_col.as_str()],
    );
    // Scope hover to the IQR box renderer; whisker/median CDSes use different columns.
    for (k, v) in &mut ht.attributes {
        if k == "renderers" {
            *v = BokehValue::Array(vec![BokehValue::ref_of(&box_renderer_id)]);
            break;
        }
    }

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

    let mut all_renderers = whisker_rs;
    all_renderers.push(box_renderer);
    all_renderers.push(med_renderer);
    add_renderers(&mut figure, all_renderers);

    if let Some(outlier_df) = outlier_df {
        if let Some((out_renderer, value_col)) = build_outlier_renderer(id_gen, outlier_df, cfg) {
            let out_renderer_id = out_renderer.id.clone();
            add_renderers(&mut figure, vec![out_renderer]);
            let out_hover = build_outlier_hover(id_gen, &cfg.category_col, &value_col, &out_renderer_id);
            add_tool_to_toolbar(&mut figure, out_hover);
        }
    }

    set_axis_labels(&mut figure, "", &cfg.y_label);
    Ok(figure)
}

fn build_outlier_hover(
    id_gen: &mut IdGen,
    category_col: &str,
    value_label: &str,
    renderer_id: &str,
) -> BokehObject {
    let cat_tip = format!("@{{{category_col}}}");
    // CDS field for outlier value is hardcoded as "value" in build_outlier_renderer.
    let val_tip = "@{value}".to_string();
    let mut ht = build_hover_tool(
        id_gen,
        &[(category_col, cat_tip.as_str()), (value_label, val_tip.as_str())],
        &[],
    );
    for (k, v) in &mut ht.attributes {
        if k == "renderers" {
            *v = BokehValue::Array(vec![BokehValue::ref_of(renderer_id)]);
            break;
        }
    }
    ht
}

fn add_tool_to_toolbar(figure: &mut BokehObject, tool: BokehObject) {
    for (k, v) in &mut figure.attributes {
        if k == "toolbar" {
            if let BokehValue::Object(tb) = v {
                for (tk, tv) in &mut tb.attributes {
                    if tk == "tools" {
                        if let BokehValue::Array(arr) = tv {
                            arr.push(tool.into_value());
                            return;
                        }
                    }
                }
            }
        }
    }
}

fn build_box_cds(
    id_gen: &mut IdGen,
    category_col: &str,
    categories: &[String],
    q1_vals: &[f64],
    q2_vals: &[f64],
    q3_vals: &[f64],
    lower_vals: &[f64],
    upper_vals: &[f64],
    colors: &[String],
) -> BokehObject {
    let to_floats = |v: &[f64]| BokehValue::Array(v.iter().map(|&x| BokehValue::Float(x)).collect());
    build_cds_from_entries(
        id_gen,
        vec![
            (category_col.into(), BokehValue::Array(categories.iter().map(|s| BokehValue::Str(s.clone())).collect())),
            ("q1".into(), to_floats(q1_vals)),
            ("q2".into(), to_floats(q2_vals)),
            ("q3".into(), to_floats(q3_vals)),
            ("lower".into(), to_floats(lower_vals)),
            ("upper".into(), to_floats(upper_vals)),
            ("_fill_color".into(), BokehValue::Array(colors.iter().map(|c| BokehValue::Str(c.clone())).collect())),
        ],
    )
}

fn build_whisker_renderers(
    id_gen: &mut IdGen,
    whisker_cds: BokehObject,
    whisker_cds_id: &str,
    filter_ref: Option<BokehValue>,
) -> Vec<BokehObject> {
    let stem_color = "#666666";
    let pairs: [(&str, &str, &str, &str); 4] = [
        ("x", "upper_y0", "x", "upper_y1"),
        ("x", "lower_y0", "x", "lower_y1"),
        ("cap_upper_x0", "upper_val", "cap_upper_x1", "upper_val"),
        ("cap_lower_x0", "lower_val", "cap_lower_x1", "lower_val"),
    ];

    pairs
        .iter()
        .enumerate()
        .map(|(i, (x0, y0, x1, y1))| {
            let glyph = build_segment_glyph(id_gen, x0, y0, x1, y1, stem_color);
            let nonsel = build_segment_glyph_nonsel(id_gen, x0, y0, x1, y1);
            let cds_ref = if i == 0 {
                whisker_cds.clone().into_value()
            } else {
                BokehValue::ref_of(whisker_cds_id)
            };
            build_glyph_renderer(id_gen, cds_ref, glyph, Some(nonsel), filter_ref.clone())
        })
        .collect()
}

fn build_box_renderer(
    id_gen: &mut IdGen,
    box_cds: BokehObject,
    category_col: &str,
    alpha: f64,
    filter_ref: Option<BokehValue>,
) -> BokehObject {
    let glyph = BokehObject::new("VBar", id_gen.next())
        .attr("x", BokehValue::field(category_col))
        .attr("top", BokehValue::field("q3"))
        .attr("bottom", BokehValue::field("q1"))
        .attr("width", BokehValue::value_of(BokehValue::Float(BOX_W)))
        .attr("fill_color", BokehValue::field("_fill_color"))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(alpha)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("#333333".into())));

    let nonsel = BokehObject::new("VBar", id_gen.next())
        .attr("x", BokehValue::field(category_col))
        .attr("top", BokehValue::field("q3"))
        .attr("bottom", BokehValue::field("q1"))
        .attr("width", BokehValue::value_of(BokehValue::Float(BOX_W)))
        .attr("fill_color", BokehValue::field("_fill_color"))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)));

    build_glyph_renderer(id_gen, box_cds.into_value(), glyph, Some(nonsel), filter_ref)
}

fn build_median_renderer(
    id_gen: &mut IdGen,
    categories: &[String],
    q2_vals: &[f64],
    filter_ref: Option<BokehValue>,
) -> BokehObject {
    let median_x0: Vec<BokehValue> = categories
        .iter()
        .map(|c| BokehValue::Array(vec![BokehValue::Str(c.clone()), BokehValue::Float(-BOX_W / 2.0)]))
        .collect();
    let median_x1: Vec<BokehValue> = categories
        .iter()
        .map(|c| BokehValue::Array(vec![BokehValue::Str(c.clone()), BokehValue::Float(BOX_W / 2.0)]))
        .collect();
    let median_cds = build_cds_from_entries(
        id_gen,
        vec![
            ("median_x0".into(), BokehValue::Array(median_x0)),
            ("median_x1".into(), BokehValue::Array(median_x1)),
            ("median_y".into(), BokehValue::Array(q2_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
        ],
    );

    let med_glyph = build_segment_glyph(id_gen, "median_x0", "median_y", "median_x1", "median_y", "#333333");
    let med_nonsel = build_segment_glyph_nonsel(id_gen, "median_x0", "median_y", "median_x1", "median_y");
    build_glyph_renderer(id_gen, median_cds.into_value(), med_glyph, Some(med_nonsel), filter_ref)
}

fn build_outlier_renderer(
    id_gen: &mut IdGen,
    outlier_df: &DataFrame,
    cfg: &BoxPlotConfig,
) -> Option<(BokehObject, String)> {
    let out_cats = get_str_column(outlier_df, &cfg.category_col).ok()?;
    let value_col = match cfg.outlier_value_col.as_deref() {
        Some(c) => c.to_string(),
        None => {
            // Pick the single non-category numeric column. Matches the shape
            // emitted by compute_box_outliers (category + value only).
            let candidates: Vec<String> = outlier_df
                .columns()
                .iter()
                .filter(|c| {
                    c.name().as_str() != cfg.category_col
                        && matches!(
                            c.dtype(),
                            polars::prelude::DataType::Int8
                                | polars::prelude::DataType::Int16
                                | polars::prelude::DataType::Int32
                                | polars::prelude::DataType::Int64
                                | polars::prelude::DataType::UInt8
                                | polars::prelude::DataType::UInt16
                                | polars::prelude::DataType::UInt32
                                | polars::prelude::DataType::UInt64
                                | polars::prelude::DataType::Float32
                                | polars::prelude::DataType::Float64
                        )
                })
                .map(|c| c.name().to_string())
                .collect();
            candidates.into_iter().next()?
        }
    };
    let out_vals = get_f64_column(outlier_df, &value_col).ok()?;

    let out_cds = build_cds_from_entries(
        id_gen,
        vec![
            (cfg.category_col.clone(), BokehValue::Array(out_cats.iter().map(|s| BokehValue::Str(s.clone())).collect())),
            ("value".into(), BokehValue::Array(out_vals.iter().map(|&v| BokehValue::Float(v)).collect())),
        ],
    );

    let glyph = BokehObject::new("Scatter", id_gen.next())
        .attr("x", BokehValue::field(&cfg.category_col))
        .attr("y", BokehValue::field("value"))
        .attr("size", BokehValue::value_of(BokehValue::Float(5.0)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str("#666666".into())))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.6)))
        .attr("line_color", BokehValue::value_of(BokehValue::Null))
        .attr("marker", BokehValue::value_of(BokehValue::Str("circle".into())));

    let nonsel = BokehObject::new("Scatter", id_gen.next())
        .attr("x", BokehValue::field(&cfg.category_col))
        .attr("y", BokehValue::field("value"))
        .attr("size", BokehValue::value_of(BokehValue::Float(5.0)))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("marker", BokehValue::value_of(BokehValue::Str("circle".into())));

    let renderer = build_glyph_renderer(id_gen, out_cds.into_value(), glyph, Some(nonsel), None);
    Some((renderer, value_col))
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
    q1_vals: &[f64],
    q3_vals: &[f64],
    lower_vals: &[f64],
    upper_vals: &[f64],
) -> BokehObject {
    let to_floats = |v: &[f64]| BokehValue::Array(v.iter().map(|&x| BokehValue::Float(x)).collect());
    let cat_with_offset = |off: f64| -> BokehValue {
        BokehValue::Array(
            categories
                .iter()
                .map(|c| BokehValue::Array(vec![BokehValue::Str(c.clone()), BokehValue::Float(off)]))
                .collect(),
        )
    };

    build_cds_from_entries(
        id_gen,
        vec![
            ("x".into(), BokehValue::Array(categories.iter().map(|s| BokehValue::Str(s.clone())).collect())),
            ("upper_y0".into(), to_floats(q3_vals)),
            ("upper_y1".into(), to_floats(upper_vals)),
            ("lower_y0".into(), to_floats(lower_vals)),
            ("lower_y1".into(), to_floats(q1_vals)),
            ("upper_val".into(), to_floats(upper_vals)),
            ("lower_val".into(), to_floats(lower_vals)),
            ("cap_upper_x0".into(), cat_with_offset(-CAP_HW)),
            ("cap_upper_x1".into(), cat_with_offset(CAP_HW)),
            ("cap_lower_x0".into(), cat_with_offset(-CAP_HW)),
            ("cap_lower_x1".into(), cat_with_offset(CAP_HW)),
        ],
    )
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
