//! Page module types for mixed-content dashboards.
//!
//! A [`PageModule`] is a single content block on a dashboard page. Pages may
//! contain any mix of charts, paragraphs, and data tables, each positioned in
//! the CSS grid via a [`GridCell`](crate::charts::GridCell).
//!
//! # Module types
//!
//! | Type | Description |
//! |---|---|
//! | [`PageModule::Chart`] | Interactive Bokeh chart (wraps [`ChartSpec`](crate::charts::ChartSpec)) |
//! | [`PageModule::Paragraph`] | Styled text block with optional heading |
//! | [`PageModule::Table`] | Formatted data table rendered from a registered DataFrame |

use crate::charts::{ChartSpec, GridCell};

// ── PageModule ────────────────────────────────────────────────────────────────

/// A single content module on a dashboard page.
///
/// Pages may mix any combination of module types in their grid layout.
/// Each module specifies its own grid position via the [`GridCell`](crate::charts::GridCell)
/// embedded in the inner spec.
///
/// Construct pages using [`PageBuilder`](crate::pages::PageBuilder), which
/// provides `.chart()`, `.paragraph()`, and `.table()` methods that
/// automatically wrap specs into the correct variant.
pub enum PageModule {
    /// An interactive Bokeh chart.
    Chart(ChartSpec),
    /// A styled text block with optional heading.
    Paragraph(ParagraphSpec),
    /// A formatted data table rendered from a registered DataFrame.
    Table(TableSpec),
}

// ── ParagraphSpec ─────────────────────────────────────────────────────────────

/// A text content block rendered as styled paragraphs.
///
/// The `text` field may contain multiple paragraphs separated by double
/// newlines (`"\n\n"`). Each paragraph is wrapped in a `<p>` element.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let para = ParagraphSpec::new(
///     "This dashboard shows monthly revenue trends.\n\n\
///      Data is sourced from the internal finance system and \
///      refreshed each quarter."
/// )
/// .title("About This Report")
/// .at(0, 0, 2)
/// .build();
/// ```
pub struct ParagraphSpec {
    /// Optional heading displayed above the text.
    pub title: Option<String>,
    /// Body text. Separate paragraphs with `"\n\n"`.
    pub text: String,
    /// Position in the page grid.
    pub grid: GridCell,
}

/// Fluent builder for [`ParagraphSpec`].
///
/// Create with [`ParagraphSpec::new`].
pub struct ParagraphSpecBuilder {
    title: Option<String>,
    text: String,
    grid: GridCell,
}

impl ParagraphSpec {
    /// Create a builder for a paragraph module with the given body text.
    ///
    /// The text may contain multiple paragraphs separated by `"\n\n"`.
    pub fn new(text: &str) -> ParagraphSpecBuilder {
        ParagraphSpecBuilder {
            title: None,
            text: text.into(),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
        }
    }
}

impl ParagraphSpecBuilder {
    /// Set an optional heading displayed above the paragraph text.
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the grid position and column span.
    ///
    /// `row` and `col` are zero-based. `span` controls how many grid columns
    /// this module occupies.
    pub fn at(mut self, row: usize, col: usize, span: usize) -> Self {
        self.grid = GridCell { row, col, col_span: span };
        self
    }

    /// Consume the builder and produce a [`ParagraphSpec`].
    pub fn build(self) -> ParagraphSpec {
        ParagraphSpec {
            title: self.title,
            text: self.text,
            grid: self.grid,
        }
    }
}

// ── ColumnFormat ──────────────────────────────────────────────────────────────

/// Formatting rule applied to each cell value in a [`TableColumn`].
pub enum ColumnFormat {
    /// Render the value as a plain string (no special formatting).
    Text,
    /// Render as a fixed-point decimal with the given number of decimal places.
    ///
    /// Example with `decimals = 2`: `1234.5` → `"1234.50"`
    Number { decimals: u32 },
    /// Render as a currency value with a symbol prefix and thousands separator.
    ///
    /// Example with `symbol = "$"`, `decimals = 2`: `1234.5` → `"$1,234.50"`
    Currency { symbol: String, decimals: u32 },
    /// Render as a percentage with the given number of decimal places.
    ///
    /// Example with `decimals = 1`: `42.567` → `"42.6%"`
    Percent { decimals: u32 },
}

// ── TableColumn ───────────────────────────────────────────────────────────────

/// A single column definition for a [`TableSpec`].
///
/// Use the factory methods ([`text`](Self::text), [`number`](Self::number),
/// [`currency`](Self::currency), [`percent`](Self::percent)) to create
/// column definitions with the appropriate format.
pub struct TableColumn {
    /// Column name in the source DataFrame.
    pub key: String,
    /// Header label displayed in the table.
    pub label: String,
    /// How to format values in this column.
    pub format: ColumnFormat,
}

impl TableColumn {
    /// Plain text column — values rendered with string conversion.
    pub fn text(key: &str, label: &str) -> Self {
        Self { key: key.into(), label: label.into(), format: ColumnFormat::Text }
    }

    /// Fixed-point number column.
    ///
    /// # Example
    ///
    /// ```ignore
    /// TableColumn::number("score", "Score", 2)  // 3.14159 → "3.14"
    /// ```
    pub fn number(key: &str, label: &str, decimals: u32) -> Self {
        Self { key: key.into(), label: label.into(), format: ColumnFormat::Number { decimals } }
    }

    /// Currency column with a prefix symbol and thousands separator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// TableColumn::currency("revenue", "Revenue", "$", 0)  // 1234567 → "$1,234,567"
    /// ```
    pub fn currency(key: &str, label: &str, symbol: &str, decimals: u32) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            format: ColumnFormat::Currency { symbol: symbol.into(), decimals },
        }
    }

    /// Percentage column.
    ///
    /// # Example
    ///
    /// ```ignore
    /// TableColumn::percent("margin", "Margin %", 1)  // 28.456 → "28.5%"
    /// ```
    pub fn percent(key: &str, label: &str, decimals: u32) -> Self {
        Self { key: key.into(), label: label.into(), format: ColumnFormat::Percent { decimals } }
    }
}

// ── TableSpec ─────────────────────────────────────────────────────────────────

/// A formatted data table rendered from a registered DataFrame.
///
/// The table displays selected columns in the order they are added, with
/// per-column formatting applied to each cell value.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
///
/// let table = TableSpec::new("Monthly Revenue", "monthly_revenue")
///     .column(TableColumn::text("month", "Month"))
///     .column(TableColumn::text("category", "Category"))
///     .column(TableColumn::currency("value", "Amount (k)", "$", 1))
///     .at(1, 0, 1)
///     .build();
/// ```
pub struct TableSpec {
    /// Heading displayed above the table.
    pub title: String,
    /// Key into the frames dictionary identifying which DataFrame to display.
    /// Must match a key registered with [`Dashboard::add_df`](crate::Dashboard::add_df).
    pub source_key: String,
    /// Columns to include in the table, in display order.
    pub columns: Vec<TableColumn>,
    /// Position in the page grid.
    pub grid: GridCell,
}

/// Fluent builder for [`TableSpec`].
///
/// Create with [`TableSpec::new`].
pub struct TableSpecBuilder {
    title: String,
    source_key: String,
    columns: Vec<TableColumn>,
    grid: GridCell,
}

impl TableSpec {
    /// Create a builder for a table module.
    ///
    /// # Arguments
    ///
    /// * `title` — Heading displayed above the table.
    /// * `source_key` — Key of the DataFrame registered with
    ///   [`Dashboard::add_df`](crate::Dashboard::add_df).
    pub fn new(title: &str, source_key: &str) -> TableSpecBuilder {
        TableSpecBuilder {
            title: title.into(),
            source_key: source_key.into(),
            columns: Vec::new(),
            grid: GridCell { row: 0, col: 0, col_span: 1 },
        }
    }
}

impl TableSpecBuilder {
    /// Add a column to the table.
    ///
    /// Columns are displayed in the order they are added.
    pub fn column(mut self, col: TableColumn) -> Self {
        self.columns.push(col);
        self
    }

    /// Set the grid position and column span.
    ///
    /// `row` and `col` are zero-based. `span` controls how many grid columns
    /// this module occupies.
    pub fn at(mut self, row: usize, col: usize, span: usize) -> Self {
        self.grid = GridCell { row, col, col_span: span };
        self
    }

    /// Consume the builder and produce a [`TableSpec`].
    pub fn build(self) -> TableSpec {
        TableSpec {
            title: self.title,
            source_key: self.source_key,
            columns: self.columns,
            grid: self.grid,
        }
    }
}
