// ── Chart types ─────────────────────────────────────────────────────────────

pub enum ChartType {
    GroupedBar,
    LineMulti,
    HBar,
    ScatterPlot,
}

impl ChartType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChartType::GroupedBar => "grouped_bar",
            ChartType::LineMulti => "line_multi",
            ChartType::HBar => "hbar",
            ChartType::ScatterPlot => "scatter",
        }
    }
}

// ── Layout structs ──────────────────────────────────────────────────────────

pub struct GridCell {
    pub row: usize,
    pub col: usize,
    pub col_span: usize,
}

pub struct ChartSpec {
    pub title: String,
    pub chart_type: ChartType,
    pub source_key: String,
    pub config: Vec<(String, String)>,
    pub grid: GridCell,
    pub filtered: bool,
}

pub enum FilterConfig {
    Range { min: f64, max: f64, step: f64 },
    Select { options: Vec<String> },
    Group { options: Vec<String> },
    Threshold { value: f64, above: bool },
    TopN { max_n: usize, descending: bool },
}

pub struct FilterSpec {
    pub source_key: String,
    pub column: String,
    pub label: String,
    pub config: FilterConfig,
}

// ── ChartSpec builder ────────────────────────────────────────────────────────

pub struct ChartSpecBuilder {
    title: String,
    chart_type: ChartType,
    source_key: String,
    config: Vec<(String, String)>,
    grid: GridCell,
    filtered: bool,
}

impl ChartSpecBuilder {
    pub fn bar(title: &str, key: &str, x: &str, group: &str, val: &str, ylabel: &str) -> Self {
        Self {
            title: title.into(),
            chart_type: ChartType::GroupedBar,
            source_key: key.into(),
            config: vec![
                ("x_col".into(), x.into()),
                ("group_col".into(), group.into()),
                ("value_col".into(), val.into()),
                ("y_label".into(), ylabel.into()),
            ],
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
        }
    }

    pub fn line(title: &str, key: &str, x: &str, ycols: &str, ylabel: &str) -> Self {
        Self {
            title: title.into(),
            chart_type: ChartType::LineMulti,
            source_key: key.into(),
            config: vec![
                ("x_col".into(), x.into()),
                ("y_cols".into(), ycols.into()),
                ("y_label".into(), ylabel.into()),
            ],
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
        }
    }

    pub fn hbar(title: &str, key: &str, cat: &str, val: &str, xlabel: &str) -> Self {
        Self {
            title: title.into(),
            chart_type: ChartType::HBar,
            source_key: key.into(),
            config: vec![
                ("category_col".into(), cat.into()),
                ("value_col".into(), val.into()),
                ("x_label".into(), xlabel.into()),
            ],
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
        }
    }

    pub fn scatter(title: &str, key: &str, x: &str, y: &str, xlabel: &str, ylabel: &str) -> Self {
        Self {
            title: title.into(),
            chart_type: ChartType::ScatterPlot,
            source_key: key.into(),
            config: vec![
                ("x_col".into(), x.into()),
                ("y_col".into(), y.into()),
                ("x_label".into(), xlabel.into()),
                ("y_label".into(), ylabel.into()),
            ],
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
        }
    }

    /// Set the grid position and column span.
    pub fn at(mut self, row: usize, col: usize, span: usize) -> Self {
        self.grid = GridCell { row, col, col_span: span };
        self
    }

    /// Mark this chart as filtered (opts into CDSView-based filtering).
    pub fn filtered(mut self) -> Self {
        self.filtered = true;
        self
    }

    pub fn build(self) -> ChartSpec {
        ChartSpec {
            title: self.title,
            chart_type: self.chart_type,
            source_key: self.source_key,
            config: self.config,
            grid: self.grid,
            filtered: self.filtered,
        }
    }
}

// ── FilterSpec factory methods ───────────────────────────────────────────────

impl FilterSpec {
    pub fn range(source_key: &str, column: &str, label: &str, min: f64, max: f64, step: f64) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::Range { min, max, step } }
    }

    pub fn select(source_key: &str, column: &str, label: &str, options: Vec<&str>) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::Select { options: options.into_iter().map(Into::into).collect() } }
    }

    pub fn group(source_key: &str, column: &str, label: &str, options: Vec<&str>) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::Group { options: options.into_iter().map(Into::into).collect() } }
    }

    pub fn threshold(source_key: &str, column: &str, label: &str, value: f64, above: bool) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::Threshold { value, above } }
    }

    pub fn top_n(source_key: &str, column: &str, label: &str, max_n: usize, descending: bool) -> Self {
        Self { source_key: source_key.into(), column: column.into(), label: label.into(),
               config: FilterConfig::TopN { max_n, descending } }
    }
}
