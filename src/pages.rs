//! Page layout types for multi-page dashboards.
//!
//! A [`Page`] groups one or more [`PageModule`](crate::modules::PageModule)s
//! and optional [`FilterSpec`](crate::charts::FilterSpec)s into a single HTML
//! file. Modules may be charts, paragraphs, or data tables — all positioned
//! in a shared CSS grid. The dashboard renderer produces one HTML file per
//! page and automatically generates a navigation bar linking all pages together.

use crate::charts::{ChartSpec, FilterSpec, MAX_GRID_COLS};
use crate::error::ChartError;
use crate::modules::{PageModule, ParagraphSpec, TableSpec};

// ── Page ─────────────────────────────────────────────────────────────────────

/// A single page in a multi-page dashboard.
///
/// Each page is rendered as a self-contained HTML file containing its modules
/// arranged in a CSS grid layout, optional filter widgets, and a navigation
/// bar linking to all other pages in the dashboard.
///
/// Construct pages using [`PageBuilder`].
pub struct Page {
    /// URL-safe identifier used as the HTML filename (e.g. `"revenue-overview"`
    /// produces `revenue-overview.html`).
    pub slug: String,
    /// Title displayed at the top of the page.
    pub title: String,
    /// Short label shown in the navigation bar.
    pub nav_label: String,
    /// Number of columns in the CSS grid layout.
    pub grid_cols: usize,
    /// Content modules (charts, paragraphs, tables) to render on this page.
    pub modules: Vec<PageModule>,
    /// Interactive filters attached to this page. Filters affect chart modules
    /// that share their `source_key` and have been marked as
    /// [`filtered`](crate::charts::ChartSpecBuilder::filtered).
    pub filters: Vec<FilterSpec>,
    /// Optional category label used to group this page in the navigation menu.
    ///
    /// Pages with the same `category` string are grouped together under that
    /// heading in the navigation bar (horizontal) or sidebar (vertical).
    /// Pages with `None` are shown ungrouped.
    pub category: Option<String>,
}

// ── Page builder ─────────────────────────────────────────────────────────────

/// Fluent builder for constructing [`Page`] instances.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let page = PageBuilder::new("overview", "Dashboard Overview", "Overview", 2)
///     .paragraph(
///         ParagraphSpec::new("Monthly performance summary for Q4.")
///             .title("About This Page")
///             .at(0, 0, 2)
///             .build()
///     )
///     .chart(ChartSpecBuilder::bar("Monthly Revenue", "revenue_data",
///         GroupedBarConfig::builder()
///             .x("month").group("category").value("amount").y_label("USD")
///             .build()?
///     ).at(1, 0, 2).build())
///     .filter(FilterSpec::range("revenue_data", "amount", "Amount", 0.0, 1000.0, 10.0))
///     .build()?;
/// ```
pub struct PageBuilder {
    slug: String,
    title: String,
    nav_label: String,
    grid_cols: usize,
    modules: Vec<PageModule>,
    filters: Vec<FilterSpec>,
    category: Option<String>,
}

impl PageBuilder {
    /// Create a new page builder.
    ///
    /// # Arguments
    ///
    /// * `slug` — URL-safe identifier used as the output filename (without
    ///   `.html` extension). Use lowercase with hyphens (e.g.
    ///   `"revenue-overview"`).
    /// * `title` — Full title displayed at the top of the page.
    /// * `nav_label` — Short label for the navigation bar.
    /// * `grid_cols` — Number of columns in the page's CSS grid layout (1–[`MAX_GRID_COLS`]).
    ///   Modules are positioned within this grid via their `.at()` method.
    pub fn new(slug: &str, title: &str, nav_label: &str, grid_cols: usize) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            nav_label: nav_label.into(),
            grid_cols,
            modules: Vec::new(),
            filters: Vec::new(),
            category: None,
        }
    }

    /// Assign this page to a navigation category group.
    ///
    /// Pages sharing the same category string are grouped together under that
    /// heading in the navigation bar. This is optional — pages without a
    /// category are shown at the top of the navigation ungrouped.
    pub fn category(mut self, cat: &str) -> Self {
        self.category = Some(cat.into());
        self
    }

    /// Add a chart to this page.
    ///
    /// The spec is wrapped in [`PageModule::Chart`](crate::modules::PageModule::Chart).
    pub fn chart(mut self, spec: ChartSpec) -> Self {
        self.modules.push(PageModule::Chart(spec));
        self
    }

    /// Add a paragraph text block to this page.
    ///
    /// The spec is wrapped in [`PageModule::Paragraph`](crate::modules::PageModule::Paragraph).
    pub fn paragraph(mut self, spec: ParagraphSpec) -> Self {
        self.modules.push(PageModule::Paragraph(spec));
        self
    }

    /// Add a formatted data table to this page.
    ///
    /// The spec is wrapped in [`PageModule::Table`](crate::modules::PageModule::Table).
    /// The table's `source_key` must reference a DataFrame registered with
    /// [`Dashboard::add_df`](crate::Dashboard::add_df).
    pub fn table(mut self, spec: TableSpec) -> Self {
        self.modules.push(PageModule::Table(spec));
        self
    }

    /// Add an interactive filter widget to this page.
    ///
    /// The filter applies to all chart modules on this page that share the
    /// filter's `source_key` and have been marked as
    /// [`filtered`](crate::charts::ChartSpecBuilder::filtered). Multiple
    /// filters on the same source are combined via Bokeh's
    /// `IntersectionFilter`.
    pub fn filter(mut self, filter: FilterSpec) -> Self {
        self.filters.push(filter);
        self
    }

    /// Consume the builder and produce a [`Page`], validating the grid layout.
    ///
    /// # Validation rules
    ///
    /// - `grid_cols` must be between 1 and [`MAX_GRID_COLS`] (inclusive).
    /// - Every module's `col_span` must be at least 1.
    /// - Every module's column index must be less than `grid_cols`.
    /// - Every module's `col + col_span` must not exceed `grid_cols`.
    /// - No two modules in the same row may occupy overlapping columns.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::GridValidation`] if any rule is violated.
    pub fn build(self) -> Result<Page, ChartError> {
        // Validate grid column count.
        if self.grid_cols == 0 || self.grid_cols > MAX_GRID_COLS {
            return Err(ChartError::GridValidation(format!(
                "grid_cols must be between 1 and {MAX_GRID_COLS}, got {}",
                self.grid_cols
            )));
        }

        // Collect (row, col_start, col_end_exclusive) for every module.
        let mut cells: Vec<(usize, usize, usize)> = Vec::with_capacity(self.modules.len());

        for module in &self.modules {
            let (row, col, span) = match module {
                PageModule::Chart(s)     => (s.grid.row, s.grid.col, s.grid.col_span),
                PageModule::Paragraph(s) => (s.grid.row, s.grid.col, s.grid.col_span),
                PageModule::Table(s)     => (s.grid.row, s.grid.col, s.grid.col_span),
            };

            if span == 0 {
                return Err(ChartError::GridValidation(
                    "col_span must be at least 1".into(),
                ));
            }

            if col >= self.grid_cols {
                return Err(ChartError::GridValidation(format!(
                    "column index {col} is out of bounds for a {}-column grid",
                    self.grid_cols
                )));
            }

            if col + span > self.grid_cols {
                return Err(ChartError::GridValidation(format!(
                    "element at row {row}, col {col} with span {span} overflows \
                     the {}-column grid (col + span = {})",
                    self.grid_cols,
                    col + span,
                )));
            }

            cells.push((row, col, col + span));
        }

        // Check for overlapping modules within the same row.
        for i in 0..cells.len() {
            let (row_i, start_i, end_i) = cells[i];
            for (row_j, start_j, end_j) in cells.iter().skip(i + 1) {
                if row_i == *row_j && start_i < *end_j && end_i > *start_j {
                    return Err(ChartError::GridValidation(format!(
                        "modules overlap in row {row_i}: columns [{start_i}, {end_i}) \
                         and [{start_j}, {end_j}) intersect"
                    )));
                }
            }
        }

        Ok(Page {
            slug: self.slug,
            title: self.title,
            nav_label: self.nav_label,
            grid_cols: self.grid_cols,
            modules: self.modules,
            filters: self.filters,
            category: self.category,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charts::{ChartSpecBuilder, HBarConfig, FilterSpec};
    use crate::modules::{ParagraphSpec, TableSpec, TableColumn};

    fn hbar_spec(row: usize, col: usize, span: usize) -> ChartSpec {
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X").build().unwrap();
        ChartSpecBuilder::hbar("Chart", "data", cfg).at(row, col, span).build()
    }

    // ── Successful builds ─────────────────────────────────────────────────────

    #[test]
    fn build_single_chart_page() {
        let page = PageBuilder::new("overview", "Overview", "Ov", 2)
            .chart(hbar_spec(0, 0, 2))
            .build()
            .unwrap();
        assert_eq!(page.slug, "overview");
        assert_eq!(page.title, "Overview");
        assert_eq!(page.nav_label, "Ov");
        assert_eq!(page.grid_cols, 2);
        assert_eq!(page.modules.len(), 1);
        assert!(page.filters.is_empty());
        assert!(page.category.is_none());
    }

    #[test]
    fn build_page_with_category() {
        let page = PageBuilder::new("p", "P", "P", 1)
            .category("Finance")
            .chart(hbar_spec(0, 0, 1))
            .build()
            .unwrap();
        assert_eq!(page.category, Some("Finance".to_string()));
    }

    #[test]
    fn build_page_with_filter() {
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(hbar_spec(0, 0, 1))
            .filter(FilterSpec::range("data", "v", "Val", 0.0, 100.0, 1.0))
            .build()
            .unwrap();
        assert_eq!(page.filters.len(), 1);
    }

    #[test]
    fn build_page_two_charts_non_overlapping() {
        let page = PageBuilder::new("p", "P", "P", 2)
            .chart(hbar_spec(0, 0, 1))
            .chart(hbar_spec(0, 1, 1))
            .build()
            .unwrap();
        assert_eq!(page.modules.len(), 2);
    }

    #[test]
    fn build_page_charts_in_different_rows() {
        let page = PageBuilder::new("p", "P", "P", 1)
            .chart(hbar_spec(0, 0, 1))
            .chart(hbar_spec(1, 0, 1))
            .build()
            .unwrap();
        assert_eq!(page.modules.len(), 2);
    }

    #[test]
    fn build_page_with_paragraph() {
        let para = ParagraphSpec::new("Hello world").at(0, 0, 1).build();
        let page = PageBuilder::new("p", "P", "P", 1)
            .paragraph(para)
            .build()
            .unwrap();
        assert_eq!(page.modules.len(), 1);
    }

    #[test]
    fn build_page_with_table() {
        let tbl = TableSpec::new("My Table", "data")
            .column(TableColumn::text("name", "Name"))
            .at(0, 0, 1)
            .build();
        let page = PageBuilder::new("p", "P", "P", 1)
            .table(tbl)
            .build()
            .unwrap();
        assert_eq!(page.modules.len(), 1);
    }

    // ── Grid validation errors ────────────────────────────────────────────────

    #[test]
    fn grid_cols_zero_fails() {
        assert!(matches!(
            PageBuilder::new("p", "P", "P", 0).build(),
            Err(ChartError::GridValidation(_))
        ));
    }

    #[test]
    fn grid_cols_exceeds_max_fails() {
        assert!(matches!(
            PageBuilder::new("p", "P", "P", MAX_GRID_COLS + 1).build(),
            Err(ChartError::GridValidation(_))
        ));
    }

    #[test]
    fn grid_cols_at_max_succeeds() {
        let page = PageBuilder::new("p", "P", "P", MAX_GRID_COLS)
            .chart(hbar_spec(0, 0, MAX_GRID_COLS))
            .build()
            .unwrap();
        assert_eq!(page.grid_cols, MAX_GRID_COLS);
    }

    #[test]
    fn col_span_zero_fails() {
        let cfg = HBarConfig::builder()
            .category("c").value("v").x_label("X").build().unwrap();
        let spec = ChartSpecBuilder::hbar("C", "d", cfg).at(0, 0, 0).build();
        assert!(matches!(
            PageBuilder::new("p", "P", "P", 2).chart(spec).build(),
            Err(ChartError::GridValidation(_))
        ));
    }

    #[test]
    fn col_index_out_of_bounds_fails() {
        // col=2 is out of bounds for a 2-column grid (valid cols are 0, 1)
        assert!(matches!(
            PageBuilder::new("p", "P", "P", 2).chart(hbar_spec(0, 2, 1)).build(),
            Err(ChartError::GridValidation(_))
        ));
    }

    #[test]
    fn col_plus_span_overflow_fails() {
        // col=1, span=2 → col+span=3 overflows a 2-column grid
        assert!(matches!(
            PageBuilder::new("p", "P", "P", 2).chart(hbar_spec(0, 1, 2)).build(),
            Err(ChartError::GridValidation(_))
        ));
    }

    #[test]
    fn overlapping_modules_same_row_fails() {
        // Both at row 0, col 0, span 2 — they overlap
        assert!(matches!(
            PageBuilder::new("p", "P", "P", 2)
                .chart(hbar_spec(0, 0, 2))
                .chart(hbar_spec(0, 0, 1))
                .build(),
            Err(ChartError::GridValidation(_))
        ));
    }

    #[test]
    fn partial_overlap_same_row_fails() {
        // col 0 span 2 occupies [0,2) and col 1 span 1 occupies [1,2) → overlap
        assert!(matches!(
            PageBuilder::new("p", "P", "P", 3)
                .chart(hbar_spec(0, 0, 2))
                .chart(hbar_spec(0, 1, 1))
                .build(),
            Err(ChartError::GridValidation(_))
        ));
    }

    #[test]
    fn adjacent_modules_same_row_succeeds() {
        // col 0 span 1 and col 1 span 1 are adjacent, not overlapping
        let page = PageBuilder::new("p", "P", "P", 2)
            .chart(hbar_spec(0, 0, 1))
            .chart(hbar_spec(0, 1, 1))
            .build()
            .unwrap();
        assert_eq!(page.modules.len(), 2);
    }
}
