// ── Chart configuration structs ──────────────────────────────────────────────

pub struct GroupedBarConfig {
    pub x_col: String,
    pub group_col: String,
    pub value_col: String,
    pub y_label: String,
}

pub struct GroupedBarConfigBuilder {
    x_col: Option<String>,
    group_col: Option<String>,
    value_col: Option<String>,
    y_label: Option<String>,
}

impl GroupedBarConfig {
    pub fn builder() -> GroupedBarConfigBuilder {
        GroupedBarConfigBuilder {
            x_col: None,
            group_col: None,
            value_col: None,
            y_label: None,
        }
    }
}

impl GroupedBarConfigBuilder {
    pub fn x(mut self, col: &str) -> Self { self.x_col = Some(col.into()); self }
    pub fn group(mut self, col: &str) -> Self { self.group_col = Some(col.into()); self }
    pub fn value(mut self, col: &str) -> Self { self.value_col = Some(col.into()); self }
    pub fn y_label(mut self, label: &str) -> Self { self.y_label = Some(label.into()); self }

    pub fn build(self) -> GroupedBarConfig {
        GroupedBarConfig {
            x_col: self.x_col.expect("x_col required"),
            group_col: self.group_col.expect("group_col required"),
            value_col: self.value_col.expect("value_col required"),
            y_label: self.y_label.expect("y_label required"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

pub struct LineConfig {
    pub x_col: String,
    pub y_cols: Vec<String>,
    pub y_label: String,
}

pub struct LineConfigBuilder {
    x_col: Option<String>,
    y_cols: Option<Vec<String>>,
    y_label: Option<String>,
}

impl LineConfig {
    pub fn builder() -> LineConfigBuilder {
        LineConfigBuilder {
            x_col: None,
            y_cols: None,
            y_label: None,
        }
    }
}

impl LineConfigBuilder {
    pub fn x(mut self, col: &str) -> Self { self.x_col = Some(col.into()); self }
    pub fn y_cols(mut self, cols: &[&str]) -> Self {
        self.y_cols = Some(cols.iter().map(|&s| s.into()).collect());
        self
    }
    pub fn y_label(mut self, label: &str) -> Self { self.y_label = Some(label.into()); self }

    pub fn build(self) -> LineConfig {
        LineConfig {
            x_col: self.x_col.expect("x_col required"),
            y_cols: self.y_cols.expect("y_cols required"),
            y_label: self.y_label.expect("y_label required"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

pub struct HBarConfig {
    pub category_col: String,
    pub value_col: String,
    pub x_label: String,
}

pub struct HBarConfigBuilder {
    category_col: Option<String>,
    value_col: Option<String>,
    x_label: Option<String>,
}

impl HBarConfig {
    pub fn builder() -> HBarConfigBuilder {
        HBarConfigBuilder {
            category_col: None,
            value_col: None,
            x_label: None,
        }
    }
}

impl HBarConfigBuilder {
    pub fn category(mut self, col: &str) -> Self { self.category_col = Some(col.into()); self }
    pub fn value(mut self, col: &str) -> Self { self.value_col = Some(col.into()); self }
    pub fn x_label(mut self, label: &str) -> Self { self.x_label = Some(label.into()); self }

    pub fn build(self) -> HBarConfig {
        HBarConfig {
            category_col: self.category_col.expect("category_col required"),
            value_col: self.value_col.expect("value_col required"),
            x_label: self.x_label.expect("x_label required"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────

pub struct ScatterConfig {
    pub x_col: String,
    pub y_col: String,
    pub x_label: String,
    pub y_label: String,
}

pub struct ScatterConfigBuilder {
    x_col: Option<String>,
    y_col: Option<String>,
    x_label: Option<String>,
    y_label: Option<String>,
}

impl ScatterConfig {
    pub fn builder() -> ScatterConfigBuilder {
        ScatterConfigBuilder {
            x_col: None,
            y_col: None,
            x_label: None,
            y_label: None,
        }
    }
}

impl ScatterConfigBuilder {
    pub fn x(mut self, col: &str) -> Self { self.x_col = Some(col.into()); self }
    pub fn y(mut self, col: &str) -> Self { self.y_col = Some(col.into()); self }
    pub fn x_label(mut self, label: &str) -> Self { self.x_label = Some(label.into()); self }
    pub fn y_label(mut self, label: &str) -> Self { self.y_label = Some(label.into()); self }

    pub fn build(self) -> ScatterConfig {
        ScatterConfig {
            x_col: self.x_col.expect("x_col required"),
            y_col: self.y_col.expect("y_col required"),
            x_label: self.x_label.expect("x_label required"),
            y_label: self.y_label.expect("y_label required"),
        }
    }
}

// ── Chart config enum ────────────────────────────────────────────────────────

pub enum ChartConfig {
    GroupedBar(GroupedBarConfig),
    Line(LineConfig),
    HBar(HBarConfig),
    Scatter(ScatterConfig),
}

impl ChartConfig {
    pub fn chart_type_str(&self) -> &'static str {
        match self {
            ChartConfig::GroupedBar(_) => "grouped_bar",
            ChartConfig::Line(_) => "line_multi",
            ChartConfig::HBar(_) => "hbar",
            ChartConfig::Scatter(_) => "scatter",
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
    pub source_key: String,
    pub config: ChartConfig,
    pub grid: GridCell,
    pub filtered: bool,
}

// ── Filter types ─────────────────────────────────────────────────────────────

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
    source_key: String,
    config: ChartConfig,
    grid: GridCell,
    filtered: bool,
}

impl ChartSpecBuilder {
    pub fn new(title: &str, source_key: &str, config: ChartConfig) -> Self {
        Self {
            title: title.into(),
            source_key: source_key.into(),
            config,
            grid: GridCell { row: 0, col: 0, col_span: 1 },
            filtered: false,
        }
    }

    pub fn bar(title: &str, key: &str, config: GroupedBarConfig) -> Self {
        Self::new(title, key, ChartConfig::GroupedBar(config))
    }

    pub fn line(title: &str, key: &str, config: LineConfig) -> Self {
        Self::new(title, key, ChartConfig::Line(config))
    }

    pub fn hbar(title: &str, key: &str, config: HBarConfig) -> Self {
        Self::new(title, key, ChartConfig::HBar(config))
    }

    pub fn scatter(title: &str, key: &str, config: ScatterConfig) -> Self {
        Self::new(title, key, ChartConfig::Scatter(config))
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
