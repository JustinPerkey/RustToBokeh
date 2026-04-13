//! Chart builders for native Bokeh rendering.

pub mod box_plot;
pub mod density;
pub mod grouped_bar;
pub mod hbar;
pub mod histogram;
pub mod line;
pub mod pie;
pub mod scatter;

use crate::charts::{ChartConfig, ChartSpec, TooltipField, TooltipFormat, TooltipSpec};
use crate::error::ChartError;
use polars::prelude::DataFrame;
use std::collections::HashMap;

use super::document::BokehDocument;
use super::figure::build_hover_tool;
use super::id_gen::IdGen;
use super::model::{BokehObject, BokehValue};

/// Context passed to every chart builder.
pub struct ChartContext<'a> {
    pub id_gen: &'a mut IdGen,
    pub doc: &'a mut BokehDocument,
    pub frames: &'a HashMap<String, DataFrame>,
    /// CDSView filter reference for filtered charts (None if not filtered).
    pub filter_ref: Option<BokehValue>,
    /// Shared Range1d ID for RangeTool synchronisation (None if not used).
    pub range_tool_x_range_id: Option<String>,
}

/// Build a chart figure `BokehObject` without adding it to any document.
///
/// This lower-level function lets callers inspect the figure (e.g. to extract
/// the CDS ID for filter wiring) before deciding how to embed it.
pub fn build_chart_obj(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    frames: &HashMap<String, DataFrame>,
    filter_ref: Option<BokehValue>,
    range_tool_x_range_id: Option<&str>,
) -> Result<BokehObject, ChartError> {
    let df = frames.get(&spec.source_key).ok_or_else(|| {
        ChartError::NativeRender(format!(
            "source_key '{}' not registered",
            spec.source_key
        ))
    })?;

    match &spec.config {
        ChartConfig::HBar(c) => hbar::build_hbar(id_gen, spec, c, df, filter_ref),
        ChartConfig::Scatter(c) => scatter::build_scatter(
            id_gen, spec, c, df, filter_ref, range_tool_x_range_id,
        ),
        ChartConfig::Histogram(c) => histogram::build_histogram(id_gen, spec, c, df, filter_ref),
        ChartConfig::Line(c) => line::build_line(
            id_gen, spec, c, df, filter_ref, range_tool_x_range_id,
        ),
        ChartConfig::Pie(c) => pie::build_pie(id_gen, spec, c, df),
        ChartConfig::GroupedBar(c) => grouped_bar::build_grouped_bar(id_gen, spec, c, df, filter_ref),
        ChartConfig::BoxPlot(c) => {
            let outlier_df = c.outlier_source_key.as_ref().and_then(|k| frames.get(k));
            box_plot::build_box_plot(id_gen, spec, c, df, outlier_df, filter_ref)
        }
        ChartConfig::Density(c) => density::build_density(id_gen, spec, c, df, filter_ref),
    }
}

/// Build a chart figure for the given `ChartSpec` and add it to the document.
///
/// Returns the HTML div UUID for embedding.
pub fn build_chart(
    ctx: &mut ChartContext<'_>,
    spec: &ChartSpec,
) -> Result<String, ChartError> {
    let fig = build_chart_obj(
        ctx.id_gen,
        spec,
        ctx.frames,
        ctx.filter_ref.clone(),
        ctx.range_tool_x_range_id.as_deref(),
    )?;
    Ok(ctx.doc.add_root(fig))
}

// ── Tooltip helpers ─────────────────────────────────────────────────────────

/// Convert a `TooltipSpec` to `(tooltips, formatters)` for `build_hover_tool`.
pub fn tooltip_arrays(
    spec: &TooltipSpec,
) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let mut tooltips = Vec::new();
    let mut formatters = Vec::new();

    for field in &spec.fields {
        let (fmt_str, fmt_type) = format_tooltip_field(field);
        tooltips.push((field.label.clone(), fmt_str.clone()));
        if let Some(ft) = fmt_type {
            formatters.push((format!("@{{{}}}", field.column), ft));
        }
    }
    (tooltips, formatters)
}

fn format_tooltip_field(f: &TooltipField) -> (String, Option<String>) {
    let col = &f.column;
    match &f.format {
        TooltipFormat::Text => (format!("@{{{col}}}"), None),
        TooltipFormat::Number(dec) => {
            let d = dec.unwrap_or(2) as usize;
            let zeros = "0".repeat(d);
            (format!("@{{{col}}}{{0.{zeros}}}"), None)
        }
        TooltipFormat::Percent(dec) => {
            let d = dec.unwrap_or(1) as usize;
            let zeros = "0".repeat(d);
            (format!("@{{{col}}}{{0.{zeros}%}}"), None)
        }
        TooltipFormat::Currency => (format!("@{{{col}}}{{$0,0}}"), None),
        TooltipFormat::DateTime(scale) => {
            let fmt = time_scale_strftime(scale);
            (format!("@{{{col}}}{{{fmt}}}"), Some("datetime".to_string()))
        }
    }
}

fn time_scale_strftime(scale: &crate::charts::TimeScale) -> &'static str {
    use crate::charts::TimeScale;
    match scale {
        TimeScale::Milliseconds => "%H:%M:%S.%3N",
        TimeScale::Seconds      => "%H:%M:%S",
        TimeScale::Minutes      => "%H:%M",
        TimeScale::Hours        => "%m/%d %H:%M",
        TimeScale::Days         => "%Y-%m-%d",
        TimeScale::Months       => "%b %Y",
        TimeScale::Years        => "%Y",
    }
}

/// Build a default hover tool from column names.
pub fn default_hover_tool(id_gen: &mut IdGen, cols: &[&str]) -> BokehObject {
    let tips: Vec<(&str, String)> = cols
        .iter()
        .map(|c| (*c, format!("@{{{c}}}")))
        .collect();
    let tip_refs: Vec<(&str, &str)> = tips.iter().map(|(l, v)| (*l, v.as_str())).collect();
    build_hover_tool(id_gen, &tip_refs, &[])
}

/// Build a hover tool from a `TooltipSpec` or fall back to default column names.
pub fn make_hover_tool(
    id_gen: &mut IdGen,
    tt: Option<&TooltipSpec>,
    default_cols: &[&str],
) -> BokehObject {
    if let Some(spec) = tt {
        let (tooltips, formatters) = tooltip_arrays(spec);
        let t_refs: Vec<(&str, &str)> = tooltips.iter().map(|(l, v)| (l.as_str(), v.as_str())).collect();
        let f_refs: Vec<(&str, &str)> = formatters.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        build_hover_tool(id_gen, &t_refs, &f_refs)
    } else {
        default_hover_tool(id_gen, default_cols)
    }
}

/// Add an axis label to the figure's x and y axes.
pub fn set_axis_labels(fig: &mut BokehObject, x_label: &str, y_label: &str) {
    // Find `below` (x-axis) and `left` (y-axis), add axis_label attribute
    for (key, val) in &mut fig.attributes {
        if key == "below" {
            if let BokehValue::Array(axes) = val {
                for ax in axes {
                    if let BokehValue::Object(obj) = ax {
                        if !x_label.is_empty() {
                            obj.attributes.push(("axis_label".to_string(), BokehValue::Str(x_label.to_string())));
                        }
                    }
                }
            }
        }
        if key == "left" {
            if let BokehValue::Array(axes) = val {
                for ax in axes {
                    if let BokehValue::Object(obj) = ax {
                        if !y_label.is_empty() {
                            obj.attributes.push(("axis_label".to_string(), BokehValue::Str(y_label.to_string())));
                        }
                    }
                }
            }
        }
    }
}

/// Add one or more glyph renderers to a Figure's `renderers` list.
pub fn add_renderers(fig: &mut BokehObject, renderers: Vec<BokehObject>) {
    for (key, val) in &mut fig.attributes {
        if key == "renderers" {
            if let BokehValue::Array(arr) = val {
                for r in renderers {
                    arr.push(r.into_value());
                }
                return;
            }
        }
    }
}

/// Add a Legend to the Figure's `center` list.
pub fn add_legend(fig: &mut BokehObject, legend: BokehObject) {
    for (key, val) in &mut fig.attributes {
        if key == "center" {
            if let BokehValue::Array(arr) = val {
                arr.push(legend.into_value());
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use std::collections::HashMap;
    use crate::charts::charts::{
        ChartSpecBuilder,
        scatter::ScatterConfig,
        line::LineConfig,
        hbar::HBarConfig,
        histogram::{HistogramConfig, HistogramDisplay},
        grouped_bar::GroupedBarConfig,
        pie::PieConfig,
        box_plot::BoxPlotConfig,
        density::DensityConfig,
    };

    fn find_attr<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
        obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    // ── build_chart_obj dispatch ────────────────────────────────────────────

    fn make_frames(key: &str, df: DataFrame) -> HashMap<String, DataFrame> {
        let mut m = HashMap::new();
        m.insert(key.into(), df);
        m
    }

    #[test]
    fn dispatch_scatter() {
        let frames = make_frames("s", df!["x" => [1.0], "y" => [2.0]].unwrap());
        let spec = ChartSpecBuilder::scatter("T", "s",
            ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let fig = build_chart_obj(&mut id_gen, &spec, &frames, None, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    #[test]
    fn dispatch_line() {
        let frames = make_frames("l", df!["x" => [1.0], "a" => [2.0]].unwrap());
        let spec = ChartSpecBuilder::line("T", "l",
            LineConfig::builder().x("x").y_cols(&["a"]).y_label("Y").build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let fig = build_chart_obj(&mut id_gen, &spec, &frames, None, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    #[test]
    fn dispatch_hbar() {
        let frames = make_frames("h", df!["c" => ["A"], "v" => [10.0]].unwrap());
        let spec = ChartSpecBuilder::hbar("T", "h",
            HBarConfig::builder().category("c").value("v").x_label("X").build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let fig = build_chart_obj(&mut id_gen, &spec, &frames, None, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    #[test]
    fn dispatch_histogram() {
        let frames = make_frames("hi", df![
            "left" => [0.0], "right" => [10.0], "count" => [5.0],
            "pdf" => [0.1], "cdf" => [1.0],
        ].unwrap());
        let spec = ChartSpecBuilder::histogram("T", "hi",
            HistogramConfig::builder().x_label("X").build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let fig = build_chart_obj(&mut id_gen, &spec, &frames, None, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    #[test]
    fn dispatch_grouped_bar() {
        let frames = make_frames("gb", df![
            "q" => ["Q1", "Q1"], "p" => ["A", "B"], "v" => [10.0, 20.0],
        ].unwrap());
        let spec = ChartSpecBuilder::bar("T", "gb",
            GroupedBarConfig::builder().x("q").group("p").value("v").y_label("Y").build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let fig = build_chart_obj(&mut id_gen, &spec, &frames, None, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    #[test]
    fn dispatch_pie() {
        let frames = make_frames("p", df!["l" => ["A", "B"], "v" => [30.0, 70.0]].unwrap());
        let spec = ChartSpecBuilder::pie("T", "p",
            PieConfig::builder().label("l").value("v").build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let fig = build_chart_obj(&mut id_gen, &spec, &frames, None, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    #[test]
    fn dispatch_box_plot() {
        let frames = make_frames("bp", df![
            "category" => ["A"], "q1" => [25.0], "q2" => [50.0],
            "q3" => [75.0], "lower" => [10.0], "upper" => [90.0],
        ].unwrap());
        let spec = ChartSpecBuilder::box_plot("T", "bp",
            BoxPlotConfig::builder()
                .category("category").q1("q1").q2("q2").q3("q3")
                .lower("lower").upper("upper").y_label("Y")
                .build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let fig = build_chart_obj(&mut id_gen, &spec, &frames, None, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    #[test]
    fn dispatch_density() {
        let frames = make_frames("d", df!["cat" => ["A", "B"], "val" => [10.0, 20.0]].unwrap());
        let spec = ChartSpecBuilder::density("T", "d",
            DensityConfig::builder().category("cat").value("val").y_label("Y").build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let fig = build_chart_obj(&mut id_gen, &spec, &frames, None, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    #[test]
    fn dispatch_missing_source_key_returns_error() {
        let frames: HashMap<String, DataFrame> = HashMap::new();
        let spec = ChartSpecBuilder::scatter("T", "missing",
            ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap()
        ).build();
        let mut id_gen = IdGen::new();
        let result = build_chart_obj(&mut id_gen, &spec, &frames, None, None);
        assert!(result.is_err());
    }

    // ── tooltip_arrays ──────────────────────────────────────────────────────

    #[test]
    fn tooltip_text_format() {
        let spec = TooltipSpec::builder()
            .field("name", "Name", TooltipFormat::Text)
            .build();
        let (tips, fmts) = tooltip_arrays(&spec);
        assert_eq!(tips.len(), 1);
        assert_eq!(tips[0].0, "Name");
        assert_eq!(tips[0].1, "@{name}");
        assert!(fmts.is_empty());
    }

    #[test]
    fn tooltip_number_format() {
        let spec = TooltipSpec::builder()
            .field("revenue", "Revenue", TooltipFormat::Number(Some(2)))
            .build();
        let (tips, _) = tooltip_arrays(&spec);
        assert_eq!(tips[0].1, "@{revenue}{0.00}");
    }

    #[test]
    fn tooltip_percent_format() {
        let spec = TooltipSpec::builder()
            .field("rate", "Rate", TooltipFormat::Percent(Some(1)))
            .build();
        let (tips, _) = tooltip_arrays(&spec);
        assert_eq!(tips[0].1, "@{rate}{0.0%}");
    }

    #[test]
    fn tooltip_currency_format() {
        let spec = TooltipSpec::builder()
            .field("price", "Price", TooltipFormat::Currency)
            .build();
        let (tips, _) = tooltip_arrays(&spec);
        assert_eq!(tips[0].1, "@{price}{$0,0}");
    }

    #[test]
    fn tooltip_datetime_format_and_formatter() {
        use crate::charts::TimeScale;
        let spec = TooltipSpec::builder()
            .field("ts", "Time", TooltipFormat::DateTime(TimeScale::Days))
            .build();
        let (tips, fmts) = tooltip_arrays(&spec);
        assert!(tips[0].1.contains("%Y-%m-%d"));
        assert_eq!(fmts.len(), 1);
        assert_eq!(fmts[0].1, "datetime");
    }

    // ── default_hover_tool ──────────────────────────────────────────────────

    #[test]
    fn default_hover_tool_creates_hovertool() {
        let mut id_gen = IdGen::new();
        let ht = default_hover_tool(&mut id_gen, &["x", "y"]);
        assert_eq!(ht.name, "HoverTool");
        let json = serde_json::to_string(&ht).unwrap();
        assert!(json.contains("@{x}"));
        assert!(json.contains("@{y}"));
    }

    // ── add_renderers ───────────────────────────────────────────────────────

    #[test]
    fn add_renderers_appends_to_empty() {
        let mut fig = BokehObject::new("Figure", "f1".into())
            .attr("renderers", BokehValue::Array(vec![]));
        let r = BokehObject::new("GlyphRenderer", "r1".into());
        add_renderers(&mut fig, vec![r]);
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            assert_eq!(arr.len(), 1);
        }
    }

    #[test]
    fn add_renderers_appends_multiple() {
        let mut fig = BokehObject::new("Figure", "f1".into())
            .attr("renderers", BokehValue::Array(vec![]));
        let r1 = BokehObject::new("GlyphRenderer", "r1".into());
        let r2 = BokehObject::new("GlyphRenderer", "r2".into());
        add_renderers(&mut fig, vec![r1, r2]);
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "renderers") {
            assert_eq!(arr.len(), 2);
        }
    }

    // ── set_axis_labels ─────────────────────────────────────────────────────

    #[test]
    fn set_axis_labels_adds_to_axes() {
        let x_axis = BokehObject::new("LinearAxis", "xa".into());
        let y_axis = BokehObject::new("LinearAxis", "ya".into());
        let mut fig = BokehObject::new("Figure", "f1".into())
            .attr("below", BokehValue::Array(vec![x_axis.into_value()]))
            .attr("left", BokehValue::Array(vec![y_axis.into_value()]));
        set_axis_labels(&mut fig, "X Label", "Y Label");
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("X Label"));
        assert!(json.contains("Y Label"));
    }

    // ── add_legend ──────────────────────────────────────────────────────────

    #[test]
    fn add_legend_appends_to_center() {
        let mut fig = BokehObject::new("Figure", "f1".into())
            .attr("center", BokehValue::Array(vec![]));
        let legend = BokehObject::new("Legend", "lg1".into());
        add_legend(&mut fig, legend);
        if let Some(BokehValue::Array(arr)) = find_attr(&fig, "center") {
            assert_eq!(arr.len(), 1);
            if let BokehValue::Object(o) = &arr[0] {
                assert_eq!(o.name, "Legend");
            }
        }
    }
}
