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
use super::super::source::{get_f64_column, get_str_column};
use super::{add_legend, add_renderers, make_hover_tool};

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
    let total: f64 = values.iter().sum();
    let show_legend = cfg.show_legend.unwrap_or(true);

    // Compute start/end angles
    let mut start_angles = Vec::with_capacity(n);
    let mut end_angles = Vec::with_capacity(n);
    let mut cumulative = -PI / 2.0; // start from top (12 o'clock)
    for &v in &values {
        let angle = if total > 0.0 { v / total * 2.0 * PI } else { 0.0 };
        start_angles.push(cumulative);
        end_angles.push(cumulative + angle);
        cumulative += angle;
    }

    let ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &[cfg.label_col.as_str(), cfg.value_col.as_str()],
    );

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        XRangeKind::Numeric { start: -1.1, end: 1.1 },
        YRangeKind::Numeric { start: -1.1, end: 1.1 },
        AxisBuilder::x(AxisType::Linear),
        AxisBuilder::y(AxisType::Linear),
        Some(ht),
    );

    // Build a shared CDS with all data (one row per slice)
    let cds_data: Vec<(String, BokehValue)> = vec![
        (cfg.label_col.clone(), BokehValue::Array(labels.iter().map(|s| BokehValue::Str(s.clone())).collect())),
        (cfg.value_col.clone(), BokehValue::Array(values.iter().map(|&v| BokehValue::Float(v)).collect())),
        ("start_angle".into(), BokehValue::Array(start_angles.iter().map(|&a| BokehValue::Float(a)).collect())),
        ("end_angle".into(), BokehValue::Array(end_angles.iter().map(|&a| BokehValue::Float(a)).collect())),
        ("color".into(), BokehValue::Array(colors.iter().map(|c| BokehValue::Str(c.clone())).collect())),
    ];

    let cds_id = id_gen.next();
    let sel_id = id_gen.next();
    let policy_id = id_gen.next();
    let cds = BokehObject::new("ColumnDataSource", cds_id.clone())
        .attr(
            "selected",
            BokehObject::new("Selection", sel_id)
                .attr("indices", BokehValue::Array(vec![]))
                .attr("line_indices", BokehValue::Array(vec![]))
                .into_value(),
        )
        .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id).into_value())
        .attr("data", BokehValue::Map(cds_data));

    let inner_r = cfg.inner_radius.unwrap_or(0.0);
    let outer_r = 0.9_f64;

    let mut renderers: Vec<BokehObject> = Vec::new();
    let mut legend_items: Vec<BokehValue> = Vec::new();

    for i in 0..n {
        // Each slice gets its own CDSView with IndexFilter(indices=[i])
        let filter_id = id_gen.next();
        let filter = BokehObject::new("IndexFilter", filter_id.clone())
            .attr("indices", BokehValue::Array(vec![BokehValue::Int(i as i64)]));

        let glyph_id = id_gen.next();
        let glyph = BokehObject::new("AnnularWedge", glyph_id)
            .attr("x", BokehValue::value_of(BokehValue::Float(0.0)))
            .attr("y", BokehValue::value_of(BokehValue::Float(0.0)))
            .attr("outer_radius", BokehValue::value_of(BokehValue::Float(outer_r)))
            .attr("inner_radius", BokehValue::value_of(BokehValue::Float(inner_r)))
            .attr("start_angle", BokehValue::field("start_angle"))
            .attr("end_angle", BokehValue::field("end_angle"))
            .attr("fill_color", BokehValue::field("color"))
            .attr("line_color", BokehValue::value_of(BokehValue::Str("white".into())));

        let nonsel_id = id_gen.next();
        let nonsel = BokehObject::new("AnnularWedge", nonsel_id)
            .attr("x", BokehValue::value_of(BokehValue::Float(0.0)))
            .attr("y", BokehValue::value_of(BokehValue::Float(0.0)))
            .attr("outer_radius", BokehValue::value_of(BokehValue::Float(outer_r)))
            .attr("inner_radius", BokehValue::value_of(BokehValue::Float(inner_r)))
            .attr("start_angle", BokehValue::field("start_angle"))
            .attr("end_angle", BokehValue::field("end_angle"))
            .attr("fill_color", BokehValue::field("color"))
            .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.3)))
            .attr("line_color", BokehValue::value_of(BokehValue::Str("white".into())));

        // First slice embeds the CDS inline; rest use reference
        let cds_ref = if i == 0 {
            cds.clone().into_value()
        } else {
            BokehValue::ref_of(&cds_id)
        };

        let renderer = build_glyph_renderer(
            id_gen,
            cds_ref,
            glyph,
            Some(nonsel),
            Some(filter.into_value()),
        );
        let renderer_id = renderer.id.clone();
        renderers.push(renderer);

        if show_legend {
            let item_id = id_gen.next();
            let item = BokehObject::new("LegendItem", item_id)
                .attr("label", BokehValue::value_of(BokehValue::Str(labels[i].clone())))
                .attr("renderers", BokehValue::Array(vec![BokehValue::ref_of(&renderer_id)]));
            legend_items.push(item.into_value());
        }
    }

    add_renderers(&mut figure, renderers);

    if show_legend && !legend_items.is_empty() {
        let side = cfg.legend_side.as_deref().unwrap_or("right");
        let legend_id = id_gen.next();
        let legend = BokehObject::new("Legend", legend_id)
            .attr("items", BokehValue::Array(legend_items))
            .attr("location", BokehValue::Str(side.into()));
        add_legend(&mut figure, legend);
    }

    // Hide axes for pie charts
    hide_axes(&mut figure);

    Ok(figure)
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
