//! Histogram chart builder.

use polars::prelude::DataFrame;

use crate::charts::charts::histogram::{HistogramConfig, HistogramDisplay};
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::source::{build_column_data_source, get_f64_column};
use super::{add_renderers, make_hover_tool, set_axis_labels};

pub fn build_histogram(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &HistogramConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
) -> Result<BokehObject, ChartError> {
    let display = cfg.display.as_ref().cloned().unwrap_or(HistogramDisplay::Count);
    let y_col = display.as_str(); // "count", "pdf", or "cdf"

    let y_label = cfg.y_label.as_deref().unwrap_or(match &display {
        HistogramDisplay::Count => "Count",
        HistogramDisplay::Pdf => "Density",
        HistogramDisplay::Cdf => "Cumulative Fraction",
    });

    let color = cfg.color.as_deref().unwrap_or("#4C72B0");
    let line_color = cfg.line_color.as_deref().unwrap_or("white");
    let alpha = cfg.alpha.unwrap_or(0.8);

    let ht = make_hover_tool(
        id_gen,
        cfg.tooltips.as_ref(),
        &["left", "right", y_col],
    );

    let FigureOutput { mut figure, .. } = build_figure(
        id_gen,
        &spec.title,
        spec.height.unwrap_or(400),
        spec.width,
        XRangeKind::DataRange,
        YRangeKind::DataRange,
        AxisBuilder::x(AxisType::Linear).config(cfg.x_axis.as_ref()),
        AxisBuilder::y(AxisType::Linear).config(cfg.y_axis.as_ref()),
        Some(ht),
    );

    let cds = build_column_data_source(id_gen, df);
    let cds_ref = cds.into_value();

    if matches!(display, HistogramDisplay::Cdf) {
        // CDF: render as a step line using Line glyph
        // Pre-compute stepped x (right edges) and y values
        let right = get_f64_column(df, "right").map_err(ChartError::NativeRender)?;
        let cdf = get_f64_column(df, "cdf").map_err(ChartError::NativeRender)?;

        let mut step_x: Vec<BokehValue> = Vec::new();
        let mut step_y: Vec<BokehValue> = Vec::new();
        // Start at 0
        if let Some(first_left) = get_f64_column(df, "left").ok().and_then(|v| v.into_iter().next()) {
            step_x.push(BokehValue::Float(first_left));
            step_y.push(BokehValue::Float(0.0));
        }
        for (x, y) in right.iter().zip(cdf.iter()) {
            step_x.push(BokehValue::Float(*x));
            step_y.push(BokehValue::Float(*y));
        }

        // Build a separate CDS for the step line
        let step_cds_id = id_gen.next();
        let sel_id = id_gen.next();
        let policy_id = id_gen.next();
        let step_cds = BokehObject::new("ColumnDataSource", step_cds_id)
            .attr(
                "selected",
                BokehObject::new("Selection", sel_id)
                    .attr("indices", BokehValue::Array(vec![]))
                    .attr("line_indices", BokehValue::Array(vec![]))
                    .into_value(),
            )
            .attr(
                "selection_policy",
                BokehObject::new("UnionRenderers", policy_id).into_value(),
            )
            .attr(
                "data",
                BokehValue::Map(vec![
                    ("x".into(), BokehValue::Array(step_x)),
                    ("y".into(), BokehValue::Array(step_y)),
                ]),
            );

        let glyph_id = id_gen.next();
        let glyph = BokehObject::new("Line", glyph_id)
            .attr("x", BokehValue::field("x"))
            .attr("y", BokehValue::field("y"))
            .attr("line_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
            .attr("line_width", BokehValue::value_of(BokehValue::Float(2.0)));

        let nonsel_id = id_gen.next();
        let nonsel = BokehObject::new("Line", nonsel_id)
            .attr("x", BokehValue::field("x"))
            .attr("y", BokehValue::field("y"))
            .attr("line_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
            .attr("line_alpha", BokehValue::value_of(BokehValue::Float(0.1)));

        let renderer = build_glyph_renderer(id_gen, step_cds.into_value(), glyph, Some(nonsel), filter_ref);
        add_renderers(&mut figure, vec![renderer]);
    } else {
        // Count or PDF: render as quad bars
        let glyph_id = id_gen.next();
        let glyph = BokehObject::new("Quad", glyph_id)
            .attr("left", BokehValue::field("left"))
            .attr("right", BokehValue::field("right"))
            .attr("top", BokehValue::field(y_col))
            .attr("bottom", BokehValue::value_of(BokehValue::Float(0.0)))
            .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
            .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(alpha)))
            .attr("line_color", BokehValue::value_of(BokehValue::Str(line_color.to_string())));

        let nonsel_id = id_gen.next();
        let nonsel = BokehObject::new("Quad", nonsel_id)
            .attr("left", BokehValue::field("left"))
            .attr("right", BokehValue::field("right"))
            .attr("top", BokehValue::field(y_col))
            .attr("bottom", BokehValue::value_of(BokehValue::Float(0.0)))
            .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.to_string())))
            .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)));

        let renderer = build_glyph_renderer(id_gen, cds_ref, glyph, Some(nonsel), filter_ref);
        add_renderers(&mut figure, vec![renderer]);
    }

    set_axis_labels(&mut figure, &cfg.x_label, y_label);
    Ok(figure)
}
