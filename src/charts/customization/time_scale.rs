/// Time unit scale used for formatting datetime values on axes and tooltips.
///
/// Each variant maps to a [`DatetimeTickFormatter`](https://docs.bokeh.org/en/latest/docs/reference/models/formatters.html#bokeh.models.formatters.DatetimeTickFormatter)
/// format string and a corresponding tooltip strftime pattern.
///
/// # Format strings produced
///
/// | Variant | Axis tick format | Tooltip format |
/// |---|---|---|
/// | `Milliseconds` | `%H:%M:%S.%3N` | `%H:%M:%S.%3N` |
/// | `Seconds` | `%H:%M:%S` | `%H:%M:%S` |
/// | `Minutes` | `%H:%M` | `%H:%M` |
/// | `Hours` | `%m/%d %H:%M` | `%m/%d %H:%M` |
/// | `Days` | `%Y-%m-%d` | `%Y-%m-%d` |
/// | `Months` | `%b %Y` | `%b %Y` |
/// | `Years` | `%Y` | `%Y` |
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let x = AxisConfig::builder()
///     .time_scale(TimeScale::Days)
///     .build();
/// ```
#[derive(Clone)]
pub enum TimeScale {
    /// Sub-second resolution: `%H:%M:%S.%3N`
    Milliseconds,
    /// Second resolution: `%H:%M:%S`
    Seconds,
    /// Minute resolution: `%H:%M`
    Minutes,
    /// Hour resolution: `%m/%d %H:%M`
    Hours,
    /// Day resolution: `%Y-%m-%d`
    Days,
    /// Month resolution: `%b %Y`
    Months,
    /// Year resolution: `%Y`
    Years,
}

impl TimeScale {
    /// Return the strftime format string used for this scale.
    #[must_use]
    pub fn format_str(&self) -> &'static str {
        match self {
            TimeScale::Milliseconds => "%H:%M:%S.%3N",
            TimeScale::Seconds => "%H:%M:%S",
            TimeScale::Minutes => "%H:%M",
            TimeScale::Hours => "%m/%d %H:%M",
            TimeScale::Days => "%Y-%m-%d",
            TimeScale::Months => "%b %Y",
            TimeScale::Years => "%Y",
        }
    }

    /// Return the string identifier passed to Python.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeScale::Milliseconds => "milliseconds",
            TimeScale::Seconds => "seconds",
            TimeScale::Minutes => "minutes",
            TimeScale::Hours => "hours",
            TimeScale::Days => "days",
            TimeScale::Months => "months",
            TimeScale::Years => "years",
        }
    }
}

/// Step size for a [`DateRange`](crate::charts::FilterConfig::DateRange) slider.
///
/// Each named variant corresponds to a standard calendar unit. Use
/// [`DateStep::Custom`] to supply an exact millisecond value when none of the
/// named units fit.
///
/// # Millisecond values
///
/// | Variant | Milliseconds |
/// |---|---|
/// | `Millisecond` | 1 |
/// | `Second` | 1 000 |
/// | `Minute` | 60 000 |
/// | `Hour` | 3 600 000 |
/// | `Day` | 86 400 000 |
/// | `Week` | 604 800 000 |
/// | `Month` | 2 592 000 000 (30 days) |
/// | `Year` | 31 536 000 000 (365 days) |
/// | `Custom(ms)` | user-supplied |
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let f = FilterSpec::date_range(
///     "events", "timestamp_ms", "Date Range",
///     1_704_067_200_000.0, 1_706_572_800_000.0,
///     DateStep::Day,
///     TimeScale::Days,
/// );
/// ```
#[derive(Clone)]
pub enum DateStep {
    /// 1 ms
    Millisecond,
    /// 1 000 ms
    Second,
    /// 60 000 ms
    Minute,
    /// 3 600 000 ms
    Hour,
    /// 86 400 000 ms
    Day,
    /// 604 800 000 ms (7 days)
    Week,
    /// 2 592 000 000 ms (30 days)
    Month,
    /// 31 536 000 000 ms (365 days)
    Year,
    /// Exact step in milliseconds.
    Custom(f64),
}

impl DateStep {
    /// Return the step size in milliseconds.
    #[must_use]
    pub fn as_ms(&self) -> f64 {
        match self {
            DateStep::Millisecond => 1.0,
            DateStep::Second => 1_000.0,
            DateStep::Minute => 60_000.0,
            DateStep::Hour => 3_600_000.0,
            DateStep::Day => 86_400_000.0,
            DateStep::Week => 604_800_000.0,
            DateStep::Month => 2_592_000_000.0,
            DateStep::Year => 31_536_000_000.0,
            DateStep::Custom(ms) => *ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_step_named_variants_as_ms() {
        assert_eq!(DateStep::Millisecond.as_ms(), 1.0);
        assert_eq!(DateStep::Second.as_ms(), 1_000.0);
        assert_eq!(DateStep::Minute.as_ms(), 60_000.0);
        assert_eq!(DateStep::Hour.as_ms(), 3_600_000.0);
        assert_eq!(DateStep::Day.as_ms(), 86_400_000.0);
        assert_eq!(DateStep::Week.as_ms(), 604_800_000.0);
        assert_eq!(DateStep::Month.as_ms(), 2_592_000_000.0);
        assert_eq!(DateStep::Year.as_ms(), 31_536_000_000.0);
    }

    #[test]
    fn date_step_custom_returns_supplied_value() {
        assert_eq!(DateStep::Custom(12345.0).as_ms(), 12345.0);
    }
}
