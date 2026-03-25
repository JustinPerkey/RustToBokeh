use super::time_scale::{DateStep, TimeScale};

/// Configuration for a single interactive filter widget.
///
/// Each variant maps to a specific Bokeh widget and filter model:
///
/// | Variant | Widget | Bokeh filter |
/// |---|---|---|
/// | [`Range`](FilterConfig::Range) | `RangeSlider` | `BooleanFilter` |
/// | [`Select`](FilterConfig::Select) | `Select` (dropdown with "All") | `BooleanFilter` |
/// | [`Group`](FilterConfig::Group) | `Select` (dropdown) | `GroupFilter` |
/// | [`Threshold`](FilterConfig::Threshold) | `Switch` (toggle) | `BooleanFilter` |
/// | [`TopN`](FilterConfig::TopN) | `Slider` | `IndexFilter` |
///
/// Multiple filters targeting the same `source_key` are combined
/// automatically via Bokeh's `IntersectionFilter`.
pub enum FilterConfig {
    /// A range slider that filters rows where the column value falls within
    /// `[min, max]`. The slider moves in increments of `step`.
    Range { min: f64, max: f64, step: f64 },
    /// A dropdown selector with an "All" option. Selecting a specific value
    /// filters rows to those matching that value; "All" shows everything.
    Select { options: Vec<String> },
    /// A dropdown selector that uses Bokeh's `GroupFilter` to show only rows
    /// belonging to the selected group. Unlike [`Select`](FilterConfig::Select),
    /// there is no "All" option.
    Group { options: Vec<String> },
    /// A toggle switch that, when enabled, filters rows based on a threshold
    /// comparison. If `above` is `true`, keeps rows where the column value
    /// is greater than `value`; if `false`, keeps rows below the threshold.
    Threshold { value: f64, above: bool },
    /// A slider that limits the display to the top (or bottom) N rows, sorted
    /// by the filter's column. Uses Bokeh's `IndexFilter` to select row
    /// indices after sorting.
    TopN { max_n: usize, descending: bool },
    /// A date-range slider that filters rows where a datetime column (stored
    /// as milliseconds since the Unix epoch) falls within the selected
    /// `[min_ms, max_ms]` interval.
    ///
    /// The [`DateStep`] controls how far the slider jumps per tick; use
    /// [`DateStep::Custom`] for a non-standard interval.  The [`TimeScale`]
    /// controls how the date labels on the slider handles are formatted.
    DateRange {
        /// Lower bound in milliseconds since Unix epoch.
        min_ms: f64,
        /// Upper bound in milliseconds since Unix epoch.
        max_ms: f64,
        /// Step size for the slider (e.g. `DateStep::Day`).
        step: DateStep,
        /// Display resolution for the slider handle labels.
        scale: TimeScale,
    },
}

/// A declarative filter definition attached to a page.
///
/// Each `FilterSpec` targets a specific data source and column, producing a
/// Bokeh widget with a `CustomJS` callback that updates the filter in real
/// time. Charts on the same page that share the filter's `source_key` and
/// have been marked as [`filtered`](ChartSpecBuilder::filtered) will respond
/// to changes.
///
/// Use the factory methods ([`range`](FilterSpec::range),
/// [`select`](FilterSpec::select), [`group`](FilterSpec::group),
/// [`threshold`](FilterSpec::threshold), [`top_n`](FilterSpec::top_n)) to
/// construct instances.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// // Only show rows where revenue is between 40 and 320
/// let f = FilterSpec::range("my_data", "revenue", "Revenue Range", 40.0, 320.0, 10.0);
/// ```
pub struct FilterSpec {
    /// Key identifying which data source this filter applies to.
    pub source_key: String,
    /// Column name in the data source to filter on.
    pub column: String,
    /// Human-readable label displayed next to the widget.
    pub label: String,
    /// The filter type and its parameters.
    pub config: FilterConfig,
}

// ── FilterSpec factory methods ───────────────────────────────────────────────

impl FilterSpec {
    /// Create a range slider filter.
    ///
    /// Produces a `RangeSlider` widget that filters rows where the column
    /// value falls within the selected `[min, max]` range. The slider moves
    /// in increments of `step`.
    #[must_use]
    pub fn range(
        source_key: &str,
        column: &str,
        label: &str,
        min: f64,
        max: f64,
        step: f64,
    ) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::Range { min, max, step },
        }
    }

    /// Create a dropdown select filter with an "All" option.
    ///
    /// The dropdown lists each value in `options` plus an "All" entry at the
    /// top. Selecting "All" removes the filter; selecting a specific value
    /// keeps only rows matching that value.
    pub fn select(source_key: &str, column: &str, label: &str, options: Vec<&str>) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::Select {
                options: options.into_iter().map(Into::into).collect(),
            },
        }
    }

    /// Create a group filter (dropdown without an "All" option).
    ///
    /// Uses Bokeh's `GroupFilter` to show only rows belonging to the selected
    /// group. The first option is selected by default.
    pub fn group(source_key: &str, column: &str, label: &str, options: Vec<&str>) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::Group {
                options: options.into_iter().map(Into::into).collect(),
            },
        }
    }

    /// Create a threshold toggle filter.
    ///
    /// Produces a `Switch` widget. When toggled on, rows are filtered based
    /// on whether the column value is above (`above = true`) or below
    /// (`above = false`) the given `value`.
    #[must_use]
    pub fn threshold(source_key: &str, column: &str, label: &str, value: f64, above: bool) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::Threshold { value, above },
        }
    }

    /// Create a top-N slider filter.
    ///
    /// Produces a `Slider` widget that limits display to the top (or bottom)
    /// N rows sorted by the filter's column. `max_n` sets the slider's upper
    /// bound. If `descending` is `true`, the highest values are kept; if
    /// `false`, the lowest.
    #[must_use]
    pub fn top_n(
        source_key: &str,
        column: &str,
        label: &str,
        max_n: usize,
        descending: bool,
    ) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::TopN { max_n, descending },
        }
    }

    /// Create a date-range slider filter.
    ///
    /// Produces a Bokeh `DateRangeSlider` widget. Data values in `column`
    /// must be stored as **milliseconds since the Unix epoch** (e.g. the
    /// output of `datetime.timestamp() * 1000` in Python or
    /// `SystemTime::UNIX_EPOCH.elapsed()?.as_millis()` in Rust).
    ///
    /// * `min_ms` / `max_ms` — slider bounds in milliseconds.
    /// * `step` — how far the slider moves per tick (e.g. `DateStep::Day`);
    ///   use `DateStep::Custom(ms)` for a non-standard interval.
    /// * `scale` — controls how handle labels are formatted on the widget.
    #[must_use]
    pub fn date_range(
        source_key: &str,
        column: &str,
        label: &str,
        min_ms: f64,
        max_ms: f64,
        step: DateStep,
        scale: TimeScale,
    ) -> Self {
        Self {
            source_key: source_key.into(),
            column: column.into(),
            label: label.into(),
            config: FilterConfig::DateRange { min_ms, max_ms, step, scale },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── FilterSpec factory methods ────────────────────────────────────────────

    #[test]
    fn filter_spec_range_stores_config() {
        let f = FilterSpec::range("src", "col", "Label", 10.0, 100.0, 5.0);
        assert_eq!(f.source_key, "src");
        assert_eq!(f.column, "col");
        assert_eq!(f.label, "Label");
        match f.config {
            FilterConfig::Range { min, max, step } => {
                assert_eq!(min, 10.0);
                assert_eq!(max, 100.0);
                assert_eq!(step, 5.0);
            }
            _ => panic!("expected Range config"),
        }
    }

    #[test]
    fn filter_spec_select_stores_config() {
        let f = FilterSpec::select("src", "col", "Label", vec!["A", "B", "C"]);
        match f.config {
            FilterConfig::Select { options } => {
                assert_eq!(options, vec!["A", "B", "C"]);
            }
            _ => panic!("expected Select config"),
        }
    }

    #[test]
    fn filter_spec_group_stores_config() {
        let f = FilterSpec::group("src", "col", "Label", vec!["X", "Y"]);
        match f.config {
            FilterConfig::Group { options } => {
                assert_eq!(options, vec!["X", "Y"]);
            }
            _ => panic!("expected Group config"),
        }
    }

    #[test]
    fn filter_spec_threshold_stores_config() {
        let f = FilterSpec::threshold("src", "col", "Label", 50.0, true);
        match f.config {
            FilterConfig::Threshold { value, above } => {
                assert_eq!(value, 50.0);
                assert!(above);
            }
            _ => panic!("expected Threshold config"),
        }
    }

    #[test]
    fn filter_spec_top_n_stores_config() {
        let f = FilterSpec::top_n("src", "col", "Label", 10, false);
        match f.config {
            FilterConfig::TopN { max_n, descending } => {
                assert_eq!(max_n, 10);
                assert!(!descending);
            }
            _ => panic!("expected TopN config"),
        }
    }
}
