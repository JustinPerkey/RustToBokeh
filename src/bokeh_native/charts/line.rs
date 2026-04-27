//! Multi-line chart builder.

use polars::prelude::*;

use crate::charts::charts::line::LineConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::palette::resolve_palette;
use super::super::source::build_column_data_source;
use super::{add_legend, add_renderers, make_hover_tool, set_axis_labels};

pub fn build_line(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &LineConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
    range_tool_x_range_id: Option<&str>,
) -> Result<BokehObject, ChartError> {
    let colors = resolve_palette(cfg.palette.as_ref(), cfg.y_cols.len());
    let line_width = cfg.line_width.unwrap_or(2.5);
    let point_size = cfg.point_size.unwrap_or(7.0);

    let (x_range, x_axis_type) = resolve_x_axis(cfg, df, range_tool_x_range_id);

    let mut default_cols: Vec<&str> = vec![cfg.x_col.as_str()];
    default_cols.extend(cfg.y_cols.iter().map(|s| s.as_str()));
    let ht = make_hover_tool(id_gen, cfg.tooltips.as_ref(), &default_cols);

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

    let cds = build_column_data_source(id_gen, df);
    let cds_id = cds.id.clone();
    let cds_value = cds.into_value();

    let mut legend_items: Vec<BokehValue> = Vec::new();
    for (i, y_col) in cfg.y_cols.iter().enumerate() {
        let cds_ref = if i == 0 {
            cds_value.clone()
        } else {
            BokehValue::ref_of(&cds_id)
        };
        let (line_r, circle_r, legend_item) = build_line_series(
            id_gen,
            &cfg.x_col,
            y_col,
            &colors[i],
            line_width,
            point_size,
            cds_ref,
            BokehValue::ref_of(&cds_id),
            filter_ref.clone(),
        );
        legend_items.push(legend_item);
        add_renderers(&mut figure, vec![line_r, circle_r]);
    }

    let legend = BokehObject::new("Legend", id_gen.next())
        .attr("items", BokehValue::Array(legend_items))
        .attr("location", BokehValue::Str("top_right".into()))
        .attr("click_policy", BokehValue::Str("hide".into()));
    add_legend(&mut figure, legend);

    set_axis_labels(&mut figure, "", &cfg.y_label);
    Ok(figure)
}

/// Resolve x-axis range kind and axis type based on config + data.
fn resolve_x_axis(
    cfg: &LineConfig,
    df: &DataFrame,
    range_tool_x_range_id: Option<&str>,
) -> (XRangeKind, AxisType) {
    let is_datetime = cfg.x_axis.as_ref().and_then(|a| a.time_scale.as_ref()).is_some();
    let x_col_dtype = df
        .column(&cfg.x_col)
        .map(|c| c.dtype().clone())
        .unwrap_or(DataType::Float64);
    let is_categorical = matches!(
        x_col_dtype,
        DataType::String | DataType::Categorical(_, _) | DataType::Enum(_, _)
    );

    let x_range = if let Some(rt_id) = range_tool_x_range_id {
        XRangeKind::ExistingId(rt_id.to_string())
    } else if is_categorical {
        let x_series = df.column(&cfg.x_col).unwrap();
        let x_cast = x_series.cast(&DataType::String).unwrap();
        let factors: Vec<BokehValue> = x_cast
            .str()
            .unwrap()
            .into_iter()
            .map(|v| BokehValue::Str(v.unwrap_or("").to_string()))
            .collect();
        XRangeKind::Factor(factors)
    } else {
        XRangeKind::DataRange
    };

    let axis_type = if is_datetime {
        AxisType::Datetime
    } else if is_categorical {
        AxisType::Categorical
    } else {
        AxisType::Linear
    };

    (x_range, axis_type)
}

/// Build (line_renderer, circle_renderer, legend_item) for one y-series.
#[allow(clippy::too_many_arguments)]
fn build_line_series(
    id_gen: &mut IdGen,
    x_col: &str,
    y_col: &str,
    color: &str,
    line_width: f64,
    point_size: f64,
    line_cds_ref: BokehValue,
    circle_cds_ref: BokehValue,
    filter_ref: Option<BokehValue>,
) -> (BokehObject, BokehObject, BokehValue) {
    let line_glyph = BokehObject::new("Line", id_gen.next())
        .attr("x", BokehValue::field(x_col))
        .attr("y", BokehValue::field(y_col))
        .attr("line_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("line_width", BokehValue::value_of(BokehValue::Float(line_width)));

    let line_nonsel = BokehObject::new("Line", id_gen.next())
        .attr("x", BokehValue::field(x_col))
        .attr("y", BokehValue::field(y_col))
        .attr("line_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("line_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("line_width", BokehValue::value_of(BokehValue::Float(line_width)));

    let line_renderer = build_glyph_renderer(id_gen, line_cds_ref, line_glyph, Some(line_nonsel), filter_ref.clone());
    let line_renderer_id = line_renderer.id.clone();

    let circle_glyph = BokehObject::new("Scatter", id_gen.next())
        .attr("x", BokehValue::field(x_col))
        .attr("y", BokehValue::field(y_col))
        .attr("size", BokehValue::value_of(BokehValue::Float(point_size)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
        .attr("marker", BokehValue::value_of(BokehValue::Str("circle".to_string())));

    let circle_nonsel = BokehObject::new("Scatter", id_gen.next())
        .attr("x", BokehValue::field(x_col))
        .attr("y", BokehValue::field(y_col))
        .attr("size", BokehValue::value_of(BokehValue::Float(point_size)))
        .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
        .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
        .attr("line_color", BokehValue::value_of(BokehValue::Str("white".to_string())))
        .attr("marker", BokehValue::value_of(BokehValue::Str("circle".to_string())));

    let circle_renderer = build_glyph_renderer(id_gen, circle_cds_ref, circle_glyph, Some(circle_nonsel), filter_ref);
    let circle_renderer_id = circle_renderer.id.clone();

    let legend_item = BokehObject::new("LegendItem", id_gen.next())
        .attr("label", BokehValue::value_of(BokehValue::Str(y_col.to_string())))
        .attr(
            "renderers",
            BokehValue::Array(vec![
                BokehValue::ref_of(&line_renderer_id),
                BokehValue::ref_of(&circle_renderer_id),
            ]),
        )
        .into_value();

    (line_renderer, circle_renderer, legend_item)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::{ChartConfig, ChartSpec, GridCell};

    fn find_attr<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
        obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn test_df() -> DataFrame {
        df!["month" => ["Jan", "Feb", "Mar"], "rev" => [10.0, 20.0, 30.0], "exp" => [5.0, 10.0, 15.0]].unwrap()
    }

    fn numeric_df() -> DataFrame {
        df!["x" => [1.0f64, 2.0, 3.0], "a" => [10.0, 20.0, 30.0], "b" => [5.0, 15.0, 25.0]].unwrap()
    }

    fn test_spec(title: &str) -> ChartSpec {
        ChartSpec {
            title: title.into(),
            source_key: "test".into(),
            config: ChartConfig::Line(
                LineConfig::builder().x("x").y_cols(&["a"]).y_label("Y").build().unwrap(),
            ),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
            width: None,
            height: None,
        }
    }

    #[test]
    fn line_single_series_has_line_and_scatter_renderers() {
        let df = numeric_df();
        let mut id_gen = IdGen::new();
        let cfg = LineConfig::builder().x("x").y_cols(&["a"]).y_label("Y").build().unwrap();
        let spec = test_spec("Single");
        let fig = build_line(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();

        assert_eq!(fig.name, "Figure");
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            // 1 series = 1 Line renderer + 1 Scatter (circle marker) renderer
            assert_eq!(arr.len(), 2);
            let names: Vec<&str> = arr.iter().filter_map(|v| {
                if let BokehValue::Object(r) = v {
                    find_attr(r, "glyph").and_then(|g| {
                        if let BokehValue::Object(go) = g { Some(go.name) } else { None }
                    })
                } else { None }
            }).collect();
            assert!(names.contains(&"Line"), "should have Line glyph");
            assert!(names.contains(&"Scatter"), "should have Scatter circle marker");
        }
    }

    #[test]
    fn line_multi_series_has_correct_renderer_count() {
        let df = numeric_df();
        let mut id_gen = IdGen::new();
        let cfg = LineConfig::builder().x("x").y_cols(&["a", "b"]).y_label("Y").build().unwrap();
        let spec = test_spec("Multi");
        let fig = build_line(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();

        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            // 2 series × 2 renderers = 4
            assert_eq!(arr.len(), 4);
        }
    }

    #[test]
    fn line_has_legend() {
        let df = numeric_df();
        let mut id_gen = IdGen::new();
        let cfg = LineConfig::builder().x("x").y_cols(&["a", "b"]).y_label("Y").build().unwrap();
        let spec = test_spec("Legend");
        let fig = build_line(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("Legend"));
        assert!(json.contains("LegendItem"));
        assert!(json.contains("hide"), "click_policy should be hide");
    }

    #[test]
    fn line_categorical_x_uses_factor_range() {
        let df = test_df();
        let mut id_gen = IdGen::new();
        let cfg = LineConfig::builder().x("month").y_cols(&["rev"]).y_label("Y").build().unwrap();
        let spec = test_spec("Categorical");
        let fig = build_line(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("FactorRange"));
        assert!(json.contains("Jan"));
    }

    #[test]
    fn line_with_filter_ref() {
        let df = numeric_df();
        let mut id_gen = IdGen::new();
        let cfg = LineConfig::builder().x("x").y_cols(&["a"]).y_label("Y").build().unwrap();
        let spec = test_spec("Filtered");
        let filter = BokehObject::new("BooleanFilter", "bf1".into())
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); 3]));
        let fig = build_line(&mut id_gen, &spec, &cfg, &df, Some(filter.into_value()), None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("BooleanFilter"));
    }

    #[test]
    fn line_with_range_tool_x_range() {
        let df = numeric_df();
        let mut id_gen = IdGen::new();
        let cfg = LineConfig::builder().x("x").y_cols(&["a"]).y_label("Y").build().unwrap();
        let spec = test_spec("RT");
        let fig = build_line(&mut id_gen, &spec, &cfg, &df, None, Some("shared_range")).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("shared_range"));
    }

    #[test]
    fn line_cds_embeds_data() {
        let df = numeric_df();
        let mut id_gen = IdGen::new();
        let cfg = LineConfig::builder().x("x").y_cols(&["a"]).y_label("Y").build().unwrap();
        let spec = test_spec("CDS");
        let fig = build_line(&mut id_gen, &spec, &cfg, &df, None, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("ColumnDataSource"));
    }
}
