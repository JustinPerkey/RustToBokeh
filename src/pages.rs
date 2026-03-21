use crate::charts::{ChartSpec, FilterSpec};

// ── Page ─────────────────────────────────────────────────────────────────────

pub struct Page {
    pub slug: String,
    pub title: String,
    pub nav_label: String,
    pub grid_cols: usize,
    pub specs: Vec<ChartSpec>,
    pub filters: Vec<FilterSpec>,
}

// ── Page builder ─────────────────────────────────────────────────────────────

pub struct PageBuilder {
    slug: String,
    title: String,
    nav_label: String,
    grid_cols: usize,
    specs: Vec<ChartSpec>,
    filters: Vec<FilterSpec>,
}

impl PageBuilder {
    pub fn new(slug: &str, title: &str, nav_label: &str, grid_cols: usize) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            nav_label: nav_label.into(),
            grid_cols,
            specs: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn chart(mut self, spec: ChartSpec) -> Self {
        self.specs.push(spec);
        self
    }

    pub fn filter(mut self, filter: FilterSpec) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn build(self) -> Page {
        Page {
            slug: self.slug,
            title: self.title,
            nav_label: self.nav_label,
            grid_cols: self.grid_cols,
            specs: self.specs,
            filters: self.filters,
        }
    }
}
