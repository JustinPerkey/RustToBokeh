//! Density (violin/sina) chart builder with pure-Rust KDE.

use std::collections::HashMap;
use polars::prelude::DataFrame;

use crate::charts::charts::density::DensityConfig;
use crate::charts::ChartSpec;
use crate::error::ChartError;

use super::super::figure::{build_figure, build_glyph_renderer, AxisBuilder, AxisType, FigureOutput, XRangeKind, YRangeKind};
use super::super::id_gen::IdGen;
use super::super::model::{BokehObject, BokehValue};
use super::super::palette::resolve_palette;
use super::super::source::{get_f64_column, get_str_column};
use super::{add_renderers, make_hover_tool, set_axis_labels};

const KDE_GRID_POINTS: usize = 80;

pub fn build_density(
    id_gen: &mut IdGen,
    spec: &ChartSpec,
    cfg: &DensityConfig,
    df: &DataFrame,
    filter_ref: Option<BokehValue>,
) -> Result<BokehObject, ChartError> {
    let cat_vals = get_str_column(df, &cfg.category_col).map_err(ChartError::NativeRender)?;
    let num_vals = get_f64_column(df, &cfg.value_col).map_err(ChartError::NativeRender)?;

    // Group values by category (preserve insertion order)
    let mut cat_order: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
    for (cat, val) in cat_vals.iter().zip(num_vals.iter()) {
        if seen.insert(cat.clone()) {
            cat_order.push(cat.clone());
        }
        groups.entry(cat.clone()).or_default().push(*val);
    }

    let n_cats = cat_order.len();
    let threshold = cfg.point_threshold.unwrap_or(50);
    let alpha = cfg.alpha.unwrap_or(0.65);

    let colors = if let Some(p) = &cfg.palette {
        resolve_palette(Some(p), n_cats)
    } else if let Some(c) = &cfg.color {
        vec![c.clone(); n_cats]
    } else {
        resolve_palette(None, n_cats)
    };

    let ht = make_hover_tool(
        id_gen,
        None,
        &[cfg.category_col.as_str(), cfg.value_col.as_str()],
    );

    let factors: Vec<BokehValue> = cat_order.iter().map(|s| BokehValue::Str(s.clone())).collect();

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

    // Compute global y range for KDE grid
    let all_vals: Vec<f64> = num_vals.iter().filter(|v| !v.is_nan()).copied().collect();
    if all_vals.is_empty() {
        return Ok(figure);
    }
    let y_min = all_vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = all_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_range = (y_max - y_min).max(1e-10);
    let y_lo = y_min - 0.05 * y_range;
    let y_hi = y_max + 0.05 * y_range;
    let y_grid = linspace(y_lo, y_hi, KDE_GRID_POINTS);

    // Compute per-category KDE, find global max density
    let mut cat_kdes: Vec<Vec<f64>> = Vec::new();
    let mut global_max = 0.0_f64;
    for cat in &cat_order {
        let vals = groups.get(cat).map(|v| v.as_slice()).unwrap_or(&[]);
        let kde = gaussian_kde(vals, &y_grid, None);
        let max = kde.iter().cloned().fold(0.0_f64, f64::max);
        if max > global_max { global_max = max; }
        cat_kdes.push(kde);
    }
    // Width multiplier: half-width of the violin in FactorRange units
    const VIOLIN_W: f64 = 0.4;

    for (i, cat) in cat_order.iter().enumerate() {
        let vals = groups.get(cat).map(|v| v.as_slice()).unwrap_or(&[]);
        let color = &colors[i];
        let kde = &cat_kdes[i];

        let use_sina = vals.len() <= threshold as usize;

        if use_sina {
            // Sina plot: scatter each point, jitter within KDE envelope
            let max_kde = kde.iter().cloned().fold(0.0_f64, f64::max).max(1e-12);
            let mut rng = LcgRng::from_str(cat);
            let xs: Vec<BokehValue> = vals.iter().map(|&v| {
                let kde_at_v = interp_kde(kde, &y_grid, v);
                let half_w = VIOLIN_W * (kde_at_v / max_kde);
                let jitter = rng.next_f64() * 2.0 * half_w - half_w;
                BokehValue::Array(vec![BokehValue::Str(cat.clone()), BokehValue::Float(jitter)])
            }).collect();
            let ys: Vec<BokehValue> = vals.iter().map(|&v| BokehValue::Float(v)).collect();

            let cds_id = id_gen.next();
            let sel_id = id_gen.next();
            let policy_id = id_gen.next();
            let cds = BokehObject::new("ColumnDataSource", cds_id)
                .attr(
                    "selected",
                    BokehObject::new("Selection", sel_id)
                        .attr("indices", BokehValue::Array(vec![]))
                        .attr("line_indices", BokehValue::Array(vec![]))
                        .into_value(),
                )
                .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id).into_value())
                .attr("data", BokehValue::Map(vec![
                    ("x".into(), BokehValue::Array(xs)),
                    ("y".into(), BokehValue::Array(ys)),
                ]));

            let glyph_id = id_gen.next();
            let glyph = BokehObject::new("Scatter", glyph_id)
                .attr("x", BokehValue::field("x"))
                .attr("y", BokehValue::field("y"))
                .attr("size", BokehValue::value_of(BokehValue::Float(6.0)))
                .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.clone())))
                .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(alpha)))
                .attr("line_color", BokehValue::value_of(BokehValue::Null))
                .attr("marker", BokehValue::value_of(BokehValue::Str("circle".into())));

            let nonsel_id = id_gen.next();
            let nonsel = BokehObject::new("Scatter", nonsel_id)
                .attr("x", BokehValue::field("x"))
                .attr("y", BokehValue::field("y"))
                .attr("size", BokehValue::value_of(BokehValue::Float(6.0)))
                .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)))
                .attr("marker", BokehValue::value_of(BokehValue::Str("circle".into())));

            let renderer = build_glyph_renderer(id_gen, cds.into_value(), glyph, Some(nonsel), filter_ref.clone());
            add_renderers(&mut figure, vec![renderer]);
        } else {
            // Violin: mirrored KDE polygon
            let max_kde = kde.iter().cloned().fold(0.0_f64, f64::max).max(1e-12);

            // Right side: (cat, +normalized_kde) at each y point
            // Left side: (cat, -normalized_kde) at each y point (reversed)
            let mut poly_x: Vec<BokehValue> = Vec::new();
            let mut poly_y: Vec<BokehValue> = Vec::new();

            for j in 0..KDE_GRID_POINTS {
                let offset = VIOLIN_W * kde[j] / max_kde;
                poly_x.push(BokehValue::Array(vec![BokehValue::Str(cat.clone()), BokehValue::Float(offset)]));
                poly_y.push(BokehValue::Float(y_grid[j]));
            }
            for j in (0..KDE_GRID_POINTS).rev() {
                let offset = -VIOLIN_W * kde[j] / max_kde;
                poly_x.push(BokehValue::Array(vec![BokehValue::Str(cat.clone()), BokehValue::Float(offset)]));
                poly_y.push(BokehValue::Float(y_grid[j]));
            }

            let cds_id = id_gen.next();
            let sel_id = id_gen.next();
            let policy_id = id_gen.next();
            let cds = BokehObject::new("ColumnDataSource", cds_id)
                .attr(
                    "selected",
                    BokehObject::new("Selection", sel_id)
                        .attr("indices", BokehValue::Array(vec![]))
                        .attr("line_indices", BokehValue::Array(vec![]))
                        .into_value(),
                )
                .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id).into_value())
                .attr("data", BokehValue::Map(vec![
                    ("x".into(), BokehValue::Array(poly_x)),
                    ("y".into(), BokehValue::Array(poly_y)),
                ]));

            let glyph_id = id_gen.next();
            let glyph = BokehObject::new("Patch", glyph_id)
                .attr("x", BokehValue::field("x"))
                .attr("y", BokehValue::field("y"))
                .attr("fill_color", BokehValue::value_of(BokehValue::Str(color.clone())))
                .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(alpha)))
                .attr("line_color", BokehValue::value_of(BokehValue::Str(color.clone())));

            let nonsel_id = id_gen.next();
            let nonsel = BokehObject::new("Patch", nonsel_id)
                .attr("x", BokehValue::field("x"))
                .attr("y", BokehValue::field("y"))
                .attr("fill_alpha", BokehValue::value_of(BokehValue::Float(0.1)));

            let renderer = build_glyph_renderer(id_gen, cds.into_value(), glyph, Some(nonsel), filter_ref.clone());
            add_renderers(&mut figure, vec![renderer]);

            // Median line
            let median_val = median(vals);
            let med_x = vec![
                BokehValue::Array(vec![BokehValue::Str(cat.clone()), BokehValue::Float(-0.2)]),
                BokehValue::Array(vec![BokehValue::Str(cat.clone()), BokehValue::Float(0.2)]),
            ];
            let med_y = vec![BokehValue::Float(median_val), BokehValue::Float(median_val)];

            let med_cds_id = id_gen.next();
            let sel_id2 = id_gen.next();
            let policy_id2 = id_gen.next();
            let med_cds = BokehObject::new("ColumnDataSource", med_cds_id)
                .attr(
                    "selected",
                    BokehObject::new("Selection", sel_id2)
                        .attr("indices", BokehValue::Array(vec![]))
                        .attr("line_indices", BokehValue::Array(vec![]))
                        .into_value(),
                )
                .attr("selection_policy", BokehObject::new("UnionRenderers", policy_id2).into_value())
                .attr("data", BokehValue::Map(vec![
                    ("x".into(), BokehValue::Array(med_x)),
                    ("y".into(), BokehValue::Array(med_y)),
                ]));

            let med_glyph_id = id_gen.next();
            let med_glyph = BokehObject::new("Line", med_glyph_id)
                .attr("x", BokehValue::field("x"))
                .attr("y", BokehValue::field("y"))
                .attr("line_color", BokehValue::value_of(BokehValue::Str("white".into())))
                .attr("line_width", BokehValue::value_of(BokehValue::Float(2.0)));

            let med_nonsel_id = id_gen.next();
            let med_nonsel = BokehObject::new("Line", med_nonsel_id)
                .attr("x", BokehValue::field("x"))
                .attr("y", BokehValue::field("y"))
                .attr("line_alpha", BokehValue::value_of(BokehValue::Float(0.1)));

            let med_renderer = build_glyph_renderer(id_gen, med_cds.into_value(), med_glyph, Some(med_nonsel), None);
            add_renderers(&mut figure, vec![med_renderer]);
        }
    }

    set_axis_labels(&mut figure, "", &cfg.y_label);
    Ok(figure)
}

// ── Pure-Rust KDE helpers ────────────────────────────────────────────────────

fn mean(vals: &[f64]) -> f64 {
    if vals.is_empty() { return 0.0; }
    vals.iter().sum::<f64>() / vals.len() as f64
}

fn stddev(vals: &[f64]) -> f64 {
    if vals.len() < 2 { return 1.0; }
    let m = mean(vals);
    let var = vals.iter().map(|&v| (v - m) * (v - m)).sum::<f64>() / (vals.len() - 1) as f64;
    var.sqrt()
}

fn gaussian_kde(values: &[f64], grid: &[f64], bw: Option<f64>) -> Vec<f64> {
    if values.is_empty() {
        return vec![0.0; grid.len()];
    }
    let n = values.len() as f64;
    let std = stddev(values).max(1e-10);
    let h = bw.unwrap_or_else(|| (1.06 * std * n.powf(-0.2)).max(1e-6));
    let norm = 1.0 / (n * h * (2.0 * std::f64::consts::PI).sqrt());
    grid.iter().map(|&y| {
        norm * values.iter().map(|&v| {
            let z = (y - v) / h;
            (-0.5 * z * z).exp()
        }).sum::<f64>()
    }).collect()
}

/// Linearly interpolate KDE density at a given y value.
fn interp_kde(kde: &[f64], grid: &[f64], y: f64) -> f64 {
    if grid.is_empty() { return 0.0; }
    let n = grid.len();
    if y <= grid[0] { return kde[0]; }
    if y >= grid[n - 1] { return kde[n - 1]; }
    let idx = grid.partition_point(|&g| g < y).saturating_sub(1);
    let i1 = (idx + 1).min(n - 1);
    let t = (y - grid[idx]) / (grid[i1] - grid[idx]).max(1e-12);
    kde[idx] * (1.0 - t) + kde[i1] * t
}

fn linspace(lo: f64, hi: f64, n: usize) -> Vec<f64> {
    if n == 0 { return vec![]; }
    if n == 1 { return vec![lo]; }
    let step = (hi - lo) / (n - 1) as f64;
    (0..n).map(|i| lo + i as f64 * step).collect()
}

fn median(vals: &[f64]) -> f64 {
    if vals.is_empty() { return 0.0; }
    let mut sorted: Vec<f64> = vals.iter().copied().filter(|v| !v.is_nan()).collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    if n % 2 == 0 {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    } else {
        sorted[n / 2]
    }
}

/// Simple LCG pseudo-random number generator for reproducible jitter.
struct LcgRng(u64);

impl LcgRng {
    fn from_str(s: &str) -> Self {
        let seed = s.bytes().fold(12345u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        LcgRng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

#[cfg(test)]
fn find_attr_test<'a>(obj: &'a BokehObject, key: &str) -> Option<&'a BokehValue> {
    obj.attributes.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use crate::charts::{ChartConfig, ChartSpec, GridCell};
    use crate::charts::charts::density::DensityConfig;

    fn test_spec(title: &str) -> ChartSpec {
        ChartSpec {
            title: title.into(),
            source_key: "test".into(),
            config: ChartConfig::Density(
                DensityConfig::builder().category("cat").value("val").y_label("Y").build().unwrap(),
            ),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
            width: None,
            height: None,
        }
    }

    #[test]
    fn density_sina_for_few_points() {
        // 3 points per category < default threshold of 50 → sina (Scatter)
        let df = df![
            "cat" => ["A", "A", "A", "B", "B", "B"],
            "val" => [10.0, 20.0, 30.0, 15.0, 25.0, 35.0],
        ].unwrap();
        let mut id_gen = IdGen::new();
        let cfg = DensityConfig::builder().category("cat").value("val").y_label("Y").build().unwrap();
        let spec = test_spec("Sina");
        let fig = build_density(&mut id_gen, &spec, &cfg, &df, None).unwrap();

        assert_eq!(fig.name, "Figure");
        let json = serde_json::to_string(&fig).unwrap();
        // Sina uses Scatter glyphs, not Patch
        assert!(json.contains("Scatter"));
        assert!(!json.contains("Patch"));
    }

    #[test]
    fn density_violin_for_many_points() {
        // > threshold → violin (Patch glyph + median Line)
        let cats: Vec<&str> = (0..100).map(|_| "X").collect();
        let vals: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let df = df!["cat" => cats, "val" => vals].unwrap();
        let mut id_gen = IdGen::new();
        let cfg = DensityConfig::builder()
            .category("cat").value("val").y_label("Y")
            .point_threshold(50)
            .build().unwrap();
        let spec = test_spec("Violin");
        let fig = build_density(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("Patch"), "violin should use Patch glyph");
        // Median line
        assert!(json.contains("\"Line\"") || json.contains("\"name\":\"Line\""));
    }

    #[test]
    fn density_uses_factor_range() {
        let df = df!["cat" => ["A", "B"], "val" => [10.0, 20.0]].unwrap();
        let mut id_gen = IdGen::new();
        let cfg = DensityConfig::builder().category("cat").value("val").y_label("Y").build().unwrap();
        let spec = test_spec("Factors");
        let fig = build_density(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("FactorRange"));
    }

    #[test]
    fn density_with_filter_ref() {
        let df = df!["cat" => ["A", "B"], "val" => [10.0, 20.0]].unwrap();
        let mut id_gen = IdGen::new();
        let cfg = DensityConfig::builder().category("cat").value("val").y_label("Y").build().unwrap();
        let spec = test_spec("Filtered");
        let filter = BokehObject::new("BooleanFilter", "bf1".into())
            .attr("booleans", BokehValue::Array(vec![BokehValue::Bool(true); 2]));
        let fig = build_density(&mut id_gen, &spec, &cfg, &df, Some(filter.into_value())).unwrap();
        let json = serde_json::to_string(&fig).unwrap();
        assert!(json.contains("BooleanFilter"));
    }

    #[test]
    fn density_empty_data_returns_figure() {
        let df = df!["cat" => Vec::<&str>::new(), "val" => Vec::<f64>::new()].unwrap();
        let mut id_gen = IdGen::new();
        let cfg = DensityConfig::builder().category("cat").value("val").y_label("Y").build().unwrap();
        let spec = test_spec("Empty");
        let fig = build_density(&mut id_gen, &spec, &cfg, &df, None).unwrap();
        assert_eq!(fig.name, "Figure");
    }

    // ── Pure-Rust KDE helpers ────────────────────────────────────────────────

    #[test]
    fn gaussian_kde_sums_to_approx_one() {
        let vals: Vec<f64> = (0..100).map(|i| i as f64).collect();
        // Grid extends 3× the bandwidth beyond the data range to capture the tails.
        let grid = linspace(-50.0, 150.0, 400);
        let kde = gaussian_kde(&vals, &grid, None);
        // Trapezoidal integration
        let step = 200.0 / 399.0;
        let integral: f64 = kde.windows(2).map(|w| (w[0] + w[1]) * 0.5 * step).sum();
        assert!((integral - 1.0).abs() < 0.05, "integral={integral}");
    }

    #[test]
    fn lcg_rng_produces_values_in_range() {
        let mut rng = LcgRng::from_str("test");
        for _ in 0..100 {
            let v = rng.next_f64();
            assert!(v >= 0.0 && v < 1.0);
        }
    }
}
