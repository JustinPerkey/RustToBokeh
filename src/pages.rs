//! Page layout types for multi-page dashboards.
//!
//! A [`Page`] groups one or more [`PageModule`](crate::modules::PageModule)s
//! and optional [`FilterSpec`](crate::charts::FilterSpec)s into a single HTML
//! file. Modules may be charts, paragraphs, or data tables — all positioned
//! in a shared CSS grid. The dashboard renderer produces one HTML file per
//! page and automatically generates a navigation bar linking all pages together.

use crate::charts::{ChartSpec, FilterSpec};
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
///     .build();
/// ```
pub struct PageBuilder {
    slug: String,
    title: String,
    nav_label: String,
    grid_cols: usize,
    modules: Vec<PageModule>,
    filters: Vec<FilterSpec>,
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
    /// * `grid_cols` — Number of columns in the page's CSS grid layout.
    ///   Modules are positioned within this grid via their `.at()` method.
    pub fn new(slug: &str, title: &str, nav_label: &str, grid_cols: usize) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            nav_label: nav_label.into(),
            grid_cols,
            modules: Vec::new(),
            filters: Vec::new(),
        }
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

    /// Consume the builder and produce a [`Page`].
    pub fn build(self) -> Page {
        Page {
            slug: self.slug,
            title: self.title,
            nav_label: self.nav_label,
            grid_cols: self.grid_cols,
            modules: self.modules,
            filters: self.filters,
        }
    }
}
