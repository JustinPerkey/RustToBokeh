//! High-level [`Dashboard`] builder — collects `DataFrames` and pages, then
//! renders everything via either the native path or the Python path.

use polars::prelude::DataFrame;

use crate::bokeh_native::{self, BokehResources};
use crate::error::ChartError;
use crate::pages::Page;
use crate::{serialize_df, NavStyle};

/// High-level dashboard builder that collects `DataFrames` and pages, then
/// renders everything in one call.
///
/// # Workflow
///
/// 1. Create a dashboard with [`Dashboard::new`].
/// 2. Optionally set the output directory with [`output_dir`](Dashboard::output_dir)
///    (defaults to `"output"`).
/// 3. Register `DataFrames` with [`add_df`](Dashboard::add_df). Each `DataFrame`
///    is serialized immediately and stored under the given key.
/// 4. Add pages with [`add_page`](Dashboard::add_page). Charts on each page
///    reference `DataFrames` by their registered key.
/// 5. Call [`render`](Dashboard::render) or [`render_native`](Dashboard::render_native)
///    to produce the HTML files.
///
/// # Example
///
/// ```ignore
/// use rust_to_bokeh::prelude::*;
/// use polars::prelude::*;
///
/// let mut df = df!["x" => [1, 2, 3], "y" => [4, 5, 6]].unwrap();
///
/// let mut dash = Dashboard::new();
/// dash.add_df("my_data", &mut df)?;
/// dash.add_page(
///     PageBuilder::new("overview", "Overview", "Overview", 2)
///         .chart(ChartSpecBuilder::scatter("X vs Y", "my_data",
///             ScatterConfig::builder()
///                 .x("x").y("y").x_label("X").y_label("Y")
///                 .build()?
///         ).at(0, 0, 2).build())
///         .build()?,
/// );
/// dash.render()?;
/// ```
pub struct Dashboard {
    pub(crate) frames: Vec<(String, Vec<u8>)>,
    pub(crate) pages: Vec<Page>,
    pub(crate) output_dir: String,
    pub(crate) title: String,
    pub(crate) nav_style: NavStyle,
}

impl Dashboard {
    /// Create an empty dashboard with the default output directory (`"output"`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            pages: Vec::new(),
            output_dir: "output".into(),
            title: String::new(),
            nav_style: NavStyle::Horizontal,
        }
    }

    /// Set the report title displayed in the navigation bar on every page.
    #[must_use]
    pub fn title(mut self, title: &str) -> Self {
        self.title = title.into();
        self
    }

    /// Set the navigation bar orientation. Defaults to [`NavStyle::Horizontal`].
    #[must_use]
    pub fn nav_style(mut self, style: NavStyle) -> Self {
        self.nav_style = style;
        self
    }

    /// Set the output directory for generated HTML files. Defaults to `"output"`.
    #[must_use]
    pub fn output_dir(mut self, dir: &str) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Register a `DataFrame` under the given key. Serialized to Arrow IPC bytes immediately.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::Serialization`] if the `DataFrame` cannot be serialized.
    pub fn add_df(&mut self, key: &str, df: &mut DataFrame) -> Result<&mut Self, ChartError> {
        self.frames.push((key.into(), serialize_df(df)?));
        Ok(self)
    }

    /// Add a pre-built [`Page`] to the dashboard. Pages render in insertion order.
    pub fn add_page(&mut self, page: Page) -> &mut Self {
        self.pages.push(page);
        self
    }

    /// Render all pages to HTML via the embedded Python renderer.
    ///
    /// Requires the `python` feature.
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::Python`] if the Python script raises an exception.
    #[cfg(feature = "python")]
    pub fn render(&self) -> Result<(), ChartError> {
        let refs: Vec<(&str, Vec<u8>)> = self
            .frames
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        crate::render::render_dashboard(
            &refs,
            &self.pages,
            &self.output_dir,
            &self.title,
            self.nav_style.as_str(),
        )
    }

    /// Render all pages to HTML using pure Rust — no Python required.
    ///
    /// `resources` controls how Bokeh JS/CSS is delivered:
    /// - [`BokehResources::Cdn`] — load from cdn.bokeh.org
    /// - [`BokehResources::Inline`] — embed inline (requires `bokeh-inline` feature)
    ///
    /// # Errors
    ///
    /// Returns [`ChartError::NativeRender`] on chart build failure or I/O failure.
    pub fn render_native(&self, resources: BokehResources) -> Result<(), ChartError> {
        let refs: Vec<(&str, Vec<u8>)> = self
            .frames
            .iter()
            .map(|(k, v)| (k.as_str(), v.clone()))
            .collect();
        bokeh_native::render_native_dashboard(
            &refs,
            &self.pages,
            &self.output_dir,
            &self.title,
            self.nav_style,
            resources,
        )
    }
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn dashboard_new_defaults() {
        let dash = Dashboard::new();
        assert_eq!(dash.output_dir, "output");
        assert_eq!(dash.title, "");
        assert_eq!(dash.nav_style, NavStyle::Horizontal);
        assert!(dash.frames.is_empty());
        assert!(dash.pages.is_empty());
    }

    #[test]
    fn dashboard_default_matches_new() {
        let a = Dashboard::new();
        let b = Dashboard::default();
        assert_eq!(a.output_dir, b.output_dir);
        assert_eq!(a.title, b.title);
    }

    #[test]
    fn dashboard_title_sets_title() {
        let dash = Dashboard::new().title("My Report");
        assert_eq!(dash.title, "My Report");
    }

    #[test]
    fn dashboard_output_dir_sets_dir() {
        let dash = Dashboard::new().output_dir("/tmp/test-output");
        assert_eq!(dash.output_dir, "/tmp/test-output");
    }

    #[test]
    fn dashboard_nav_style_sets_style() {
        let dash = Dashboard::new().nav_style(NavStyle::Vertical);
        assert_eq!(dash.nav_style, NavStyle::Vertical);
    }

    #[test]
    fn dashboard_add_df_stores_frame() {
        let mut df = df![
            "a" => [1i64, 2],
        ]
        .unwrap();
        let mut dash = Dashboard::new();
        dash.add_df("my_data", &mut df).unwrap();
        assert_eq!(dash.frames.len(), 1);
        assert_eq!(dash.frames[0].0, "my_data");
        assert!(!dash.frames[0].1.is_empty());
    }

    #[test]
    fn dashboard_add_df_multiple_keys() {
        let mut df1 = df!["a" => [1i64]].unwrap();
        let mut df2 = df!["b" => [2i64]].unwrap();
        let mut dash = Dashboard::new();
        dash.add_df("first", &mut df1).unwrap();
        dash.add_df("second", &mut df2).unwrap();
        assert_eq!(dash.frames.len(), 2);
        assert_eq!(dash.frames[0].0, "first");
        assert_eq!(dash.frames[1].0, "second");
    }

    #[test]
    fn dashboard_add_df_returns_self_for_chaining() {
        let mut df = df!["a" => [1i64]].unwrap();
        let mut dash = Dashboard::new();
        dash.add_df("k1", &mut df)
            .unwrap()
            .add_df("k2", &mut df)
            .unwrap();
        assert_eq!(dash.frames.len(), 2);
    }

    #[test]
    fn dashboard_add_page_stores_page() {
        use crate::charts::{ChartSpecBuilder, HBarConfig};
        use crate::pages::PageBuilder;

        let cfg = HBarConfig::builder()
            .category("c")
            .value("v")
            .x_label("X")
            .build()
            .unwrap();
        let page = PageBuilder::new("overview", "Overview", "Ov", 1)
            .chart(
                ChartSpecBuilder::hbar("Chart", "data", cfg)
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap();

        let mut dash = Dashboard::new();
        dash.add_page(page);
        assert_eq!(dash.pages.len(), 1);
        assert_eq!(dash.pages[0].slug, "overview");
    }

    #[test]
    fn dashboard_add_page_multiple() {
        use crate::charts::{ChartSpecBuilder, HBarConfig};
        use crate::pages::PageBuilder;

        let make_page = |slug: &str| {
            let cfg = HBarConfig::builder()
                .category("c")
                .value("v")
                .x_label("X")
                .build()
                .unwrap();
            PageBuilder::new(slug, "Title", "Label", 1)
                .chart(ChartSpecBuilder::hbar("C", "d", cfg).at(0, 0, 1).build())
                .build()
                .unwrap()
        };

        let mut dash = Dashboard::new();
        dash.add_page(make_page("page-one"));
        dash.add_page(make_page("page-two"));
        assert_eq!(dash.pages.len(), 2);
    }

    #[test]
    fn dashboard_output_dir_used_in_render_config() {
        let dash = Dashboard::new()
            .output_dir("/custom/path")
            .title("Test")
            .nav_style(NavStyle::Vertical);
        assert_eq!(dash.output_dir, "/custom/path");
        assert_eq!(dash.title, "Test");
        assert_eq!(dash.nav_style, NavStyle::Vertical);
    }
}
