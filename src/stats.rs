//! Statistical helper functions for preparing data for histogram and box plot charts.
//!
//! These functions operate on Polars `DataFrames` and produce new `DataFrames` in the
//! format expected by [`HistogramConfig`](crate::charts::HistogramConfig) and
//! [`BoxPlotConfig`](crate::charts::BoxPlotConfig). Register their output with
//! [`Dashboard::add_df`](crate::Dashboard::add_df) before building chart specs that
//! reference them.

use polars::prelude::*;

use crate::error::ChartError;

/// Compute equal-width histogram statistics from a numeric DataFrame column.
///
/// Given a `DataFrame`, a column name, and the desired number of bins, this
/// function computes bin edges and returns a new `DataFrame` with five columns:
///
/// | Column  | Type | Description |
/// |---------|------|-------------|
/// | `left`  | f64  | Left edge of each bin |
/// | `right` | f64  | Right edge of each bin |
/// | `count` | f64  | Number of values that fall in each bin |
/// | `pdf`   | f64  | Probability density: `count / (n × bin_width)` |
/// | `cdf`   | f64  | Cumulative fraction of values up to each bin's right edge |
///
/// The result is intended to be registered with [`Dashboard::add_df`](crate::Dashboard::add_df)
/// and referenced by a [`ChartSpecBuilder::histogram`](crate::charts::ChartSpecBuilder::histogram)
/// spec. Use [`HistogramConfig`](crate::charts::HistogramConfig) with
/// [`HistogramDisplay`](crate::charts::HistogramDisplay) to choose which statistic
/// to render.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
/// use polars::prelude::*;
///
/// let raw = df!["salary" => [42.0f64, 65.0, 80.0, 95.0]].unwrap();
/// let mut hist = compute_histogram(&raw, "salary", 12)?;
/// dash.add_df("salary_hist", &mut hist)?;
/// ```
///
/// # Errors
///
/// Returns [`ChartError::Serialization`] if the column does not exist or
/// cannot be cast to `f64`.
pub fn compute_histogram(
    df: &DataFrame,
    column: &str,
    num_bins: usize,
) -> Result<DataFrame, ChartError> {
    let num_bins = num_bins.max(1);
    let series = df.column(column)?;
    let cast = series.cast(&DataType::Float64)?;
    let ca = cast.f64()?;
    let values: Vec<f64> = ca.iter().filter_map(|v| v).collect();

    if values.is_empty() {
        return Ok(df![
            "left"  => Vec::<f64>::new(),
            "right" => Vec::<f64>::new(),
            "count" => Vec::<f64>::new(),
            "pdf"   => Vec::<f64>::new(),
            "cdf"   => Vec::<f64>::new(),
        ]?);
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    // Guard against all-identical values to avoid zero-width bins.
    let (bin_min, bin_max) = if (max - min).abs() < f64::EPSILON {
        (min - 0.5, max + 0.5)
    } else {
        (min, max)
    };

    let bin_width = (bin_max - bin_min) / num_bins as f64;
    let mut counts = vec![0u64; num_bins];
    for &v in &values {
        let idx = ((v - bin_min) / bin_width).floor() as usize;
        counts[idx.min(num_bins - 1)] += 1;
    }

    let total = values.len() as f64;
    let left: Vec<f64> = (0..num_bins).map(|i| bin_min + i as f64 * bin_width).collect();
    let right: Vec<f64> = (0..num_bins).map(|i| bin_min + (i + 1) as f64 * bin_width).collect();
    let count_vals: Vec<f64> = counts.iter().map(|&c| c as f64).collect();
    let pdf: Vec<f64> = counts.iter().map(|&c| c as f64 / (total * bin_width)).collect();
    let mut cum = 0.0_f64;
    let cdf: Vec<f64> = counts
        .iter()
        .map(|&c| {
            cum += c as f64 / total;
            cum
        })
        .collect();

    Ok(df![
        "left"  => left,
        "right" => right,
        "count" => count_vals,
        "pdf"   => pdf,
        "cdf"   => cdf,
    ]?)
}

/// Compute per-category box plot statistics from a raw category + value DataFrame.
///
/// Given a `DataFrame` with a categorical column and a numeric value column,
/// this function groups by category (preserving first-appearance order) and
/// returns a new `DataFrame` with six columns:
///
/// | Column     | Type | Description |
/// |------------|------|-------------|
/// | `category` | Utf8 | Category label (same values as `category_col`) |
/// | `q1`       | f64  | 25th percentile |
/// | `q2`       | f64  | 50th percentile (median) |
/// | `q3`       | f64  | 75th percentile |
/// | `lower`    | f64  | Lower whisker: min observed value ≥ Q1 − 1.5 × IQR |
/// | `upper`    | f64  | Upper whisker: max observed value ≤ Q3 + 1.5 × IQR |
///
/// The result is intended to be registered with [`Dashboard::add_df`](crate::Dashboard::add_df)
/// and referenced by a [`ChartSpecBuilder::box_plot`](crate::charts::ChartSpecBuilder::box_plot)
/// spec using [`BoxPlotConfig`](crate::charts::BoxPlotConfig).
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
/// use polars::prelude::*;
///
/// let raw = df![
///     "department" => ["Eng", "Sales", "Eng", "Sales"],
///     "salary"     => [95.0f64, 70.0, 105.0, 80.0],
/// ].unwrap();
/// let mut stats = compute_box_stats(&raw, "department", "salary")?;
/// dash.add_df("salary_box", &mut stats)?;
/// ```
///
/// # Errors
///
/// Returns [`ChartError::Serialization`] if the columns do not exist or the
/// value column cannot be cast to `f64`.
pub fn compute_box_stats(
    df: &DataFrame,
    category_col: &str,
    value_col: &str,
) -> Result<DataFrame, ChartError> {
    fn quantile_linear(sorted: &[f64], q: f64) -> f64 {
        if sorted.len() == 1 {
            return sorted[0];
        }
        let idx = q * (sorted.len() - 1) as f64;
        let lo = idx.floor() as usize;
        let hi = idx.ceil() as usize;
        if lo == hi {
            sorted[lo]
        } else {
            let frac = idx - lo as f64;
            sorted[lo] * (1.0 - frac) + sorted[hi] * frac
        }
    }

    let cat_series = df.column(category_col)?;
    let val_series = df.column(value_col)?;
    let val_f64 = val_series.cast(&DataType::Float64)?;
    let cat_str_ca = cat_series.str()?;

    // Collect unique categories in first-appearance order.
    let mut seen = std::collections::HashSet::new();
    let mut unique_cats: Vec<String> = Vec::new();
    for opt_s in cat_str_ca.iter() {
        if let Some(s) = opt_s {
            if seen.insert(s.to_string()) {
                unique_cats.push(s.to_string());
            }
        }
    }

    let mut out_cats:  Vec<String> = Vec::new();
    let mut out_q1:    Vec<f64>    = Vec::new();
    let mut out_q2:    Vec<f64>    = Vec::new();
    let mut out_q3:    Vec<f64>    = Vec::new();
    let mut out_lower: Vec<f64>    = Vec::new();
    let mut out_upper: Vec<f64>    = Vec::new();

    for cat in &unique_cats {
        let mask = cat_str_ca.equal(cat.as_str());
        let filtered = val_f64.filter(&mask)?;
        let filtered_ca = filtered.f64()?;
        let mut vals: Vec<f64> = filtered_ca.into_no_null_iter().collect();

        if vals.is_empty() {
            continue;
        }

        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let q1  = quantile_linear(&vals, 0.25);
        let q2  = quantile_linear(&vals, 0.50);
        let q3  = quantile_linear(&vals, 0.75);
        let iqr = q3 - q1;
        let lo_fence = q1 - 1.5 * iqr;
        let hi_fence = q3 + 1.5 * iqr;

        // Whisker endpoints: most extreme observed values within the fences.
        let lower = vals
            .iter()
            .cloned()
            .filter(|&v| v >= lo_fence)
            .fold(f64::INFINITY, f64::min);
        let upper = vals
            .iter()
            .cloned()
            .filter(|&v| v <= hi_fence)
            .fold(f64::NEG_INFINITY, f64::max);

        out_cats.push(cat.clone());
        out_q1.push(q1);
        out_q2.push(q2);
        out_q3.push(q3);
        out_lower.push(lower);
        out_upper.push(upper);
    }

    Ok(df![
        category_col => out_cats,
        "q1"         => out_q1,
        "q2"         => out_q2,
        "q3"         => out_q3,
        "lower"      => out_lower,
        "upper"      => out_upper,
    ]?)
}

/// Extract outlier rows from a raw DataFrame for use with box plots.
///
/// Returns a new `DataFrame` containing only the rows whose `value_col` value
/// falls **outside** the Tukey IQR fences (Q1 − 1.5·IQR or Q3 + 1.5·IQR) for
/// their respective category. The output columns are `category_col` and
/// `value_col` (with the same names passed in), so the resulting DataFrame can
/// be registered directly with [`Dashboard::add_df`](crate::Dashboard::add_df)
/// and referenced by [`BoxPlotConfig::outlier_source`](crate::BoxPlotConfig).
///
/// # Example
///
/// ```ignore
/// let raw = data::build_salary_raw();
/// let mut outliers = compute_box_outliers(&raw, "department", "salary_k")?;
/// dash.add_df("salary_outliers", &mut outliers)?;
/// ```
///
/// # Errors
///
/// Returns [`ChartError::Serialization`] if the columns do not exist or the
/// value column cannot be cast to `f64`.
pub fn compute_box_outliers(
    df: &DataFrame,
    category_col: &str,
    value_col: &str,
) -> Result<DataFrame, ChartError> {
    fn quantile_linear(sorted: &[f64], q: f64) -> f64 {
        if sorted.len() == 1 {
            return sorted[0];
        }
        let idx = q * (sorted.len() - 1) as f64;
        let lo = idx.floor() as usize;
        let hi = idx.ceil() as usize;
        if lo == hi {
            sorted[lo]
        } else {
            let frac = idx - lo as f64;
            sorted[lo] * (1.0 - frac) + sorted[hi] * frac
        }
    }

    let cat_series = df.column(category_col)?;
    let val_series = df.column(value_col)?;
    let val_f64 = val_series.cast(&DataType::Float64)?;
    let cat_str_ca = cat_series.str()?;

    // Collect unique categories in first-appearance order.
    let mut seen = std::collections::HashSet::new();
    let mut unique_cats: Vec<String> = Vec::new();
    for opt_s in cat_str_ca.iter() {
        if let Some(s) = opt_s {
            if seen.insert(s.to_string()) {
                unique_cats.push(s.to_string());
            }
        }
    }

    let mut out_cats: Vec<String> = Vec::new();
    let mut out_vals: Vec<f64>    = Vec::new();

    for cat in &unique_cats {
        let mask = cat_str_ca.equal(cat.as_str());
        let filtered = val_f64.filter(&mask)?;
        let filtered_ca = filtered.f64()?;
        let mut vals: Vec<f64> = filtered_ca.into_no_null_iter().collect();

        if vals.is_empty() {
            continue;
        }

        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let q1  = quantile_linear(&vals, 0.25);
        let q3  = quantile_linear(&vals, 0.75);
        let iqr = q3 - q1;
        let lo_fence = q1 - 1.5 * iqr;
        let hi_fence = q3 + 1.5 * iqr;

        for v in vals {
            if v < lo_fence || v > hi_fence {
                out_cats.push(cat.clone());
                out_vals.push(v);
            }
        }
    }

    Ok(df![
        category_col => out_cats,
        value_col    => out_vals,
    ]?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── compute_histogram ─────────────────────────────────────────────────────

    #[test]
    fn histogram_basic_columns() {
        let df = df!["v" => [1.0f64, 2.0, 3.0, 4.0, 5.0]].unwrap();
        let result = compute_histogram(&df, "v", 5).unwrap();
        assert_eq!(result.width(), 5);
        let names: Vec<&str> = result.get_column_names().iter().map(|s| s.as_str()).collect();
        assert!(names.contains(&"left"));
        assert!(names.contains(&"right"));
        assert!(names.contains(&"count"));
        assert!(names.contains(&"pdf"));
        assert!(names.contains(&"cdf"));
    }

    #[test]
    fn histogram_row_count_matches_bins() {
        let df = df!["v" => [1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0]].unwrap();
        let result = compute_histogram(&df, "v", 3).unwrap();
        assert_eq!(result.height(), 3);
    }

    #[test]
    fn histogram_empty_input() {
        let df = df!["v" => Vec::<f64>::new()].unwrap();
        let result = compute_histogram(&df, "v", 5).unwrap();
        assert_eq!(result.height(), 0);
    }

    #[test]
    fn histogram_all_same_value() {
        let df = df!["v" => [7.0f64, 7.0, 7.0]].unwrap();
        let result = compute_histogram(&df, "v", 4).unwrap();
        // Should not panic with zero-width bins
        assert_eq!(result.height(), 4);
    }

    #[test]
    fn histogram_cdf_ends_at_one() {
        let df = df!["v" => [1.0f64, 2.0, 3.0, 4.0]].unwrap();
        let result = compute_histogram(&df, "v", 4).unwrap();
        let cdf = result.column("cdf").unwrap();
        let last = cdf.f64().unwrap().get(result.height() - 1).unwrap();
        assert!((last - 1.0).abs() < 1e-10);
    }

    #[test]
    fn histogram_missing_column_returns_error() {
        let df = df!["v" => [1.0f64]].unwrap();
        assert!(compute_histogram(&df, "nonexistent", 5).is_err());
    }

    // ── compute_box_stats ─────────────────────────────────────────────────────

    #[test]
    fn box_stats_columns_present() {
        let df = df![
            "cat" => ["A", "A", "B", "B"],
            "val" => [1.0f64, 3.0, 2.0, 4.0],
        ].unwrap();
        let result = compute_box_stats(&df, "cat", "val").unwrap();
        let names: Vec<&str> = result.get_column_names().iter().map(|s| s.as_str()).collect();
        assert!(names.contains(&"cat"));
        assert!(names.contains(&"q1"));
        assert!(names.contains(&"q2"));
        assert!(names.contains(&"q3"));
        assert!(names.contains(&"lower"));
        assert!(names.contains(&"upper"));
    }

    #[test]
    fn box_stats_row_count_is_unique_categories() {
        let df = df![
            "cat" => ["A", "A", "B", "B", "C"],
            "val" => [1.0f64, 2.0, 3.0, 4.0, 5.0],
        ].unwrap();
        let result = compute_box_stats(&df, "cat", "val").unwrap();
        assert_eq!(result.height(), 3);
    }

    #[test]
    fn box_stats_missing_column_returns_error() {
        let df = df!["cat" => ["A"], "val" => [1.0f64]].unwrap();
        assert!(compute_box_stats(&df, "cat", "nonexistent").is_err());
    }

    // ── compute_box_outliers ──────────────────────────────────────────────────

    #[test]
    fn box_outliers_excludes_inliers() {
        // Tight cluster — no outliers expected
        let df = df![
            "cat" => ["A", "A", "A", "A", "A"],
            "val" => [10.0f64, 11.0, 12.0, 10.5, 11.5],
        ].unwrap();
        let result = compute_box_outliers(&df, "cat", "val").unwrap();
        assert_eq!(result.height(), 0);
    }

    #[test]
    fn box_outliers_includes_extremes() {
        // One extreme outlier
        let df = df![
            "cat" => ["A", "A", "A", "A", "A", "A"],
            "val" => [10.0f64, 10.1, 10.2, 10.3, 10.4, 100.0],
        ].unwrap();
        let result = compute_box_outliers(&df, "cat", "val").unwrap();
        assert!(result.height() >= 1);
    }

    #[test]
    fn box_outliers_output_column_names() {
        let df = df![
            "dept" => ["Eng", "Eng", "Eng"],
            "salary" => [80.0f64, 82.0, 300.0],
        ].unwrap();
        let result = compute_box_outliers(&df, "dept", "salary").unwrap();
        let names: Vec<&str> = result.get_column_names().iter().map(|s| s.as_str()).collect();
        assert!(names.contains(&"dept"));
        assert!(names.contains(&"salary"));
    }
}
