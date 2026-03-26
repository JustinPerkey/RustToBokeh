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
/// | [`RangeTool`](FilterConfig::RangeTool) | Auto-generated overview chart | x-axis `Range1d` sync |
///
/// Multiple filters targeting the same `source_key` are combined
/// automatically via Bokeh's `IntersectionFilter`.
///
/// `RangeTool` is special: it does **not** use `CDSView`/row filtering.
/// Instead it synchronises the visible x-axis range of all line and scatter
/// charts on the page that share the same `source_key`, via a compact
/// navigator chart placed automatically below the grid.
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
    /// A Bokeh `RangeTool` navigator that synchronises the visible x-axis
    /// range of all line and scatter charts sharing the same `source_key`.
    ///
    /// A compact overview chart (height 130 px, no toolbar) is automatically
    /// generated and placed below the page grid. It renders `y_column` over
    /// the full x extent and attaches a draggable `RangeTool` overlay that
    /// updates the shared [`Range1d`] used by the detail charts.
    ///
    /// Unlike the other filter variants, `RangeTool` does not produce a
    /// `CDSView` — it only adjusts the visible x-axis window. Charts do not
    /// need to be marked with `.filtered()` to participate.
    ///
    /// The `source_key`'s x column is stored in `FilterSpec.column`.
    /// `start` / `end` set the initial visible window in the same units as
    /// the x column (milliseconds since epoch for datetime columns).
    /// Supply `time_scale` if the x column is a datetime so the overview
    /// chart uses a datetime axis.
    RangeTool {
        /// Column to plot on the y-axis of the auto-generated overview chart.
        y_column: String,
        /// Initial visible range start (same units as the x column).
        start: f64,
        /// Initial visible range end.
        end: f64,
        /// If `Some`, the overview chart uses a datetime x axis formatted at
        /// this resolution.
        time_scale: Option<TimeScale>,
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

    /// Create a `RangeTool` navigator.
    ///
    /// Produces a compact overview chart (height 130 px) placed automatically
    /// below the page grid.  The overview renders `y_column` over the full
    /// x extent and shows a draggable `RangeTool` overlay that synchronises
    /// the visible x-axis window of all line and scatter charts on the same
    /// page that share `source_key`.
    ///
    /// * `x_column` — the x-axis column in the data source (stored in
    ///   `FilterSpec.column`).
    /// * `y_column` — the column to draw in the overview mini-chart.
    /// * `label` — title shown on the overview chart.
    /// * `start` / `end` — initial visible x-axis window (same units as
    ///   `x_column`; use milliseconds since epoch for datetime data).
    /// * `time_scale` — pass `Some(TimeScale::Days)` (or another variant)
    ///   if `x_column` contains datetime values so the overview axis is
    ///   formatted correctly.  Pass `None` for numeric x axes.
    ///
    /// Charts do **not** need `.filtered()` to participate — the range tool
    /// zooms the axis, it does not hide rows via `CDSView`.
    #[must_use]
    pub fn range_tool(
        source_key: &str,
        x_column: &str,
        y_column: &str,
        label: &str,
        start: f64,
        end: f64,
        time_scale: Option<TimeScale>,
    ) -> Self {
        Self {
            source_key: source_key.into(),
            column: x_column.into(),
            label: label.into(),
            config: FilterConfig::RangeTool {
                y_column: y_column.into(),
                start,
                end,
                time_scale,
            },
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

    #[test]
    fn filter_spec_range_tool_stores_config() {
        let f = FilterSpec::range_tool(
            "sensor_events",
            "timestamp_ms",
            "temperature",
            "Navigator",
            1_000.0,
            9_000.0,
            Some(TimeScale::Days),
        );
        assert_eq!(f.source_key, "sensor_events");
        assert_eq!(f.column, "timestamp_ms");
        assert_eq!(f.label, "Navigator");
        match f.config {
            FilterConfig::RangeTool { y_column, start, end, time_scale } => {
                assert_eq!(y_column, "temperature");
                assert_eq!(start, 1_000.0);
                assert_eq!(end, 9_000.0);
                assert!(matches!(time_scale, Some(TimeScale::Days)));
            }
            _ => panic!("expected RangeTool config"),
        }
    }
}
