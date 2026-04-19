//! Integration tests that exercise the full rendering pipeline and verify
//! that HTML output files are written to the expected locations.
//!
//! These tests require a Python interpreter with `bokeh`, `polars`, and
//! `jinja2` installed. Run `bash scripts/setup_vendor.sh` first to configure
//! the vendored Python, then execute:
//!
//! ```sh
//! cargo test --test dashboard_output
//! ```

use polars::prelude::*;
use rust_to_bokeh::prelude::*;
use tempfile::TempDir;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Construct a `DfHandle` for tests from a raw key.
fn h(s: &str) -> DfHandle { DfHandle::new(s) }

/// Build a minimal single-row DataFrame for testing.
fn make_simple_df() -> DataFrame {
    df![
        "category" => ["Alpha", "Beta", "Gamma"],
        "value"    => [10.0f64, 20.0, 30.0],
    ]
    .unwrap()
}

/// Build a time-series DataFrame for line/bar charts.
fn make_timeseries_df() -> DataFrame {
    df![
        "month"    => ["Jan", "Feb", "Mar", "Apr"],
        "series_a" => [100.0f64, 120.0, 90.0, 140.0],
        "series_b" => [80.0f64,  95.0, 110.0, 130.0],
    ]
    .unwrap()
}

/// Build a grouped DataFrame for grouped bar charts.
fn make_grouped_df() -> DataFrame {
    df![
        "month"    => ["Jan", "Jan", "Feb", "Feb"],
        "category" => ["A", "B", "A", "B"],
        "value"    => [50.0f64, 70.0, 60.0, 80.0],
    ]
    .unwrap()
}

// ── single-page renders ───────────────────────────────────────────────────────

#[test]
#[cfg(feature = "python")]
fn hbar_page_creates_html_file() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("simple", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("hbar-page", "HBar Page", "HBar", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Category Values",
                    &h("simple"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Value")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    let html = tmp.path().join("hbar-page.html");
    assert!(html.exists(), "expected hbar-page.html to be created");
}

#[test]
#[cfg(feature = "python")]
fn hbar_page_html_contains_bokeh_content() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out).title("Test Dashboard");
    dash.add_df("simple", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("content-check", "Content Check", "Check", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Values",
                    &h("simple"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    let html_path = tmp.path().join("content-check.html");
    let content = std::fs::read_to_string(&html_path).unwrap();

    // HTML should contain Bokeh CDN resources
    assert!(
        content.contains("bokeh"),
        "HTML should reference Bokeh resources"
    );
    // HTML should contain the page title
    assert!(
        content.contains("Content Check"),
        "HTML should contain the page title"
    );
}

#[test]
#[cfg(feature = "python")]
fn line_page_creates_html_file() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_timeseries_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("ts", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("line-page", "Line Page", "Lines", 1)
            .chart(
                ChartSpecBuilder::line(
                    "Series Over Time",
                    &h("ts"),
                    LineConfig::builder()
                        .x("month")
                        .y_cols(&["series_a", "series_b"])
                        .y_label("Value")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(tmp.path().join("line-page.html").exists());
}

#[test]
#[cfg(feature = "python")]
fn grouped_bar_page_creates_html_file() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_grouped_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("grouped", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("bar-page", "Bar Page", "Bars", 1)
            .chart(
                ChartSpecBuilder::bar(
                    "Grouped Bars",
                    &h("grouped"),
                    GroupedBarConfig::builder()
                        .x("month")
                        .group("category")
                        .value("value")
                        .y_label("USD")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(tmp.path().join("bar-page.html").exists());
}

#[test]
#[cfg(feature = "python")]
fn scatter_page_creates_html_file() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = df![
        "x_vals" => [1.0f64, 2.0, 3.0, 4.0],
        "y_vals" => [4.0f64, 3.0, 2.0, 1.0],
    ]
    .unwrap();

    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("scatter_data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("scatter-page", "Scatter Page", "Scatter", 1)
            .chart(
                ChartSpecBuilder::scatter(
                    "X vs Y",
                    &h("scatter_data"),
                    ScatterConfig::builder()
                        .x("x_vals")
                        .y("y_vals")
                        .x_label("X")
                        .y_label("Y")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(tmp.path().join("scatter-page.html").exists());
}

// ── multi-page renders ────────────────────────────────────────────────────────

#[test]
#[cfg(feature = "python")]
fn multi_page_dashboard_creates_one_file_per_page() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out).title("Multi-Page Test");
    dash.add_df("data", &mut df).unwrap();

    for slug in &["page-one", "page-two", "page-three"] {
        let cfg = HBarConfig::builder()
            .category("category")
            .value("value")
            .x_label("Val")
            .build()
            .unwrap();
        dash.add_page(
            PageBuilder::new(slug, slug, slug, 1)
                .chart(
                    ChartSpecBuilder::hbar("Chart", &h("data"), cfg)
                        .at(0, 0, 1)
                        .build(),
                )
                .build()
                .unwrap(),
        );
    }

    dash.render().unwrap();

    for slug in &["page-one", "page-two", "page-three"] {
        let html = tmp.path().join(format!("{slug}.html"));
        assert!(html.exists(), "{slug}.html should be created");
    }
}

#[test]
#[cfg(feature = "python")]
fn multi_page_nav_links_reference_other_pages() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();

    for (slug, title) in &[("alpha", "Alpha Page"), ("beta", "Beta Page")] {
        let cfg = HBarConfig::builder()
            .category("category")
            .value("value")
            .x_label("Val")
            .build()
            .unwrap();
        dash.add_page(
            PageBuilder::new(slug, title, title, 1)
                .chart(ChartSpecBuilder::hbar("C", &h("data"), cfg).at(0, 0, 1).build())
                .build()
                .unwrap(),
        );
    }

    dash.render().unwrap();

    // Each page should contain a link to the other page.
    let alpha_html = std::fs::read_to_string(tmp.path().join("alpha.html")).unwrap();
    let beta_html = std::fs::read_to_string(tmp.path().join("beta.html")).unwrap();

    assert!(
        alpha_html.contains("beta"),
        "alpha.html should link to beta page"
    );
    assert!(
        beta_html.contains("alpha"),
        "beta.html should link to alpha page"
    );
}

// ── output directory configuration ───────────────────────────────────────────

#[test]
#[cfg(feature = "python")]
fn output_dir_created_automatically() {
    let tmp = TempDir::new().unwrap();
    // Use a subdirectory that does not yet exist.
    let out = tmp.path().join("nested").join("output");
    let out_str = out.to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out_str);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("p", "P", "P", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "C",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("V")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(
        out.join("p.html").exists(),
        "HTML should be created in the nested output dir"
    );
}

#[test]
#[cfg(feature = "python")]
fn custom_output_dir_does_not_pollute_cwd() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("isolated", "Isolated", "Iso", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "C",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("V")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    // The file should be in the TempDir, not in the default "output/" folder.
    assert!(
        tmp.path().join("isolated.html").exists(),
        "HTML should be in the TempDir"
    );
    assert!(
        !std::path::Path::new("output/isolated.html").exists(),
        "HTML should NOT appear in the default 'output/' directory"
    );
}

// ── page with filters ─────────────────────────────────────────────────────────

#[test]
#[cfg(feature = "python")]
fn page_with_range_filter_creates_html() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("filtered", "Filtered", "Filtered", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Filtered Chart",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .filtered()
                .build(),
            )
            .filter(FilterSpec::range(
                &h("data"),
                "value",
                "Value Range",
                0.0,
                50.0,
                1.0,
            ))
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(tmp.path().join("filtered.html").exists());
}

#[test]
#[cfg(feature = "python")]
fn page_with_select_filter_creates_html() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("select-filter", "Select Filter", "Select", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .filtered()
                .build(),
            )
            .filter(FilterSpec::select(
                &h("data"),
                "category",
                "Category",
                vec!["Alpha", "Beta", "Gamma"],
            ))
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(tmp.path().join("select-filter.html").exists());
}

// ── nav style ─────────────────────────────────────────────────────────────────

#[test]
#[cfg(feature = "python")]
fn vertical_nav_style_creates_html_file() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new()
        .output_dir(&out)
        .nav_style(NavStyle::Vertical);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("vert-nav", "Vertical Nav", "VNav", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(tmp.path().join("vert-nav.html").exists());
}

// ── page with mixed modules ───────────────────────────────────────────────────

#[test]
#[cfg(feature = "python")]
fn page_with_paragraph_and_chart_creates_html() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("mixed", "Mixed Page", "Mixed", 1)
            .paragraph(
                ParagraphSpec::new("This is a test paragraph.")
                    .title("About")
                    .at(0, 0, 1)
                    .build(),
            )
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(1, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    let html_path = tmp.path().join("mixed.html");
    assert!(html_path.exists());

    let content = std::fs::read_to_string(&html_path).unwrap();
    assert!(content.contains("This is a test paragraph."));
}

#[test]
#[cfg(feature = "python")]
fn page_with_table_module_creates_html() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("table-page", "Table Page", "Table", 1)
            .table(
                TableSpec::new("Simple Table", &h("data"))
                    .column(TableColumn::text("category", "Category"))
                    .column(TableColumn::number("value", "Value", 1))
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(tmp.path().join("table-page.html").exists());
}

// ── dashboard title ───────────────────────────────────────────────────────────

#[test]
#[cfg(feature = "python")]
fn dashboard_title_appears_in_html() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new()
        .output_dir(&out)
        .title("Acme Corp Dashboard");
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("titled", "Titled Page", "Titled", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    let content = std::fs::read_to_string(tmp.path().join("titled.html")).unwrap();
    assert!(
        content.contains("Acme Corp Dashboard"),
        "HTML should contain the dashboard title"
    );
}

// ── native renderer — CDN path ───────────────────────────────────────────────

#[test]
fn native_cdn_creates_html_file() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("native-cdn", "Native CDN", "CDN", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Values",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(rust_to_bokeh::BokehResources::Cdn)
        .unwrap();

    assert!(tmp.path().join("native-cdn.html").exists());
}

#[test]
fn native_cdn_html_references_bokeh_cdn() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out).title("CDN Test");
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("cdn-content", "CDN Content", "CDN", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(rust_to_bokeh::BokehResources::Cdn)
        .unwrap();

    let content = std::fs::read_to_string(tmp.path().join("cdn-content.html")).unwrap();
    assert!(
        content.contains("cdn.bokeh.org"),
        "CDN path should reference cdn.bokeh.org"
    );
    assert!(
        content.contains("CDN Content"),
        "HTML should contain page title"
    );
    assert!(
        content.contains("CDN Test"),
        "HTML should contain dashboard title"
    );
}

#[test]
fn native_cdn_multi_page_creates_all_files() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();

    for slug in &["native-one", "native-two", "native-three"] {
        let cfg = HBarConfig::builder()
            .category("category")
            .value("value")
            .x_label("Val")
            .build()
            .unwrap();
        dash.add_page(
            PageBuilder::new(slug, slug, slug, 1)
                .chart(ChartSpecBuilder::hbar("C", &h("data"), cfg).at(0, 0, 1).build())
                .build()
                .unwrap(),
        );
    }

    dash.render_native(rust_to_bokeh::BokehResources::Cdn)
        .unwrap();

    for slug in &["native-one", "native-two", "native-three"] {
        assert!(
            tmp.path().join(format!("{slug}.html")).exists(),
            "{slug}.html should be created"
        );
    }
}

#[test]
fn native_cdn_nav_links_cross_reference_pages() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();

    for (slug, title) in &[("nav-alpha", "Alpha Page"), ("nav-beta", "Beta Page")] {
        let cfg = HBarConfig::builder()
            .category("category")
            .value("value")
            .x_label("Val")
            .build()
            .unwrap();
        dash.add_page(
            PageBuilder::new(slug, title, title, 1)
                .chart(ChartSpecBuilder::hbar("C", &h("data"), cfg).at(0, 0, 1).build())
                .build()
                .unwrap(),
        );
    }

    dash.render_native(rust_to_bokeh::BokehResources::Cdn)
        .unwrap();

    let alpha = std::fs::read_to_string(tmp.path().join("nav-alpha.html")).unwrap();
    let beta = std::fs::read_to_string(tmp.path().join("nav-beta.html")).unwrap();

    assert!(alpha.contains("nav-beta"), "alpha.html should link to nav-beta");
    assert!(beta.contains("nav-alpha"), "beta.html should link to nav-alpha");
}

#[test]
fn native_cdn_output_dir_created_automatically() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().join("nested").join("native-out");
    let out_str = out.to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out_str);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("n", "N", "N", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "C",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("V")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(rust_to_bokeh::BokehResources::Cdn)
        .unwrap();

    assert!(out.join("n.html").exists());
}

// ── native renderer — Inline path ────────────────────────────────────────────

/// Build a minimal dashboard with one hbar chart, return it ready to render.
fn make_inline_dash(out: &str) -> Dashboard {
    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(out).title("Inline Test");
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("inline-page", "Inline Page", "Inline", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart",
                    &h("data"),
                    HBarConfig::builder()
                        .category("category")
                        .value("value")
                        .x_label("Val")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash
}

/// Without the `bokeh-inline` Cargo feature, `BokehResources::Inline` must
/// return an error rather than silently falling back to CDN.
#[test]
#[cfg(not(feature = "bokeh-inline"))]
fn native_inline_without_feature_returns_error() {
    let tmp = TempDir::new().unwrap();
    let result = make_inline_dash(tmp.path().to_str().unwrap())
        .render_native(rust_to_bokeh::BokehResources::Inline);
    assert!(
        result.is_err(),
        "Inline without bokeh-inline feature should return an error"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("bokeh-inline"),
        "error message should mention the 'bokeh-inline' feature, got: {msg}"
    );
}

/// With `bokeh-inline` feature: output is created and contains no CDN links.
#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_creates_html_file() {
    let tmp = TempDir::new().unwrap();
    make_inline_dash(tmp.path().to_str().unwrap())
        .render_native(rust_to_bokeh::BokehResources::Inline)
        .unwrap();
    assert!(tmp.path().join("inline-page.html").exists());
}

/// Inline HTML must be self-contained — no references to cdn.bokeh.org.
#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_html_has_no_cdn_links() {
    let tmp = TempDir::new().unwrap();
    make_inline_dash(tmp.path().to_str().unwrap())
        .render_native(rust_to_bokeh::BokehResources::Inline)
        .unwrap();

    let content = std::fs::read_to_string(tmp.path().join("inline-page.html")).unwrap();
    assert!(
        !content.contains("cdn.bokeh.org"),
        "Inline HTML must not reference cdn.bokeh.org"
    );
    assert!(
        content.contains("Inline Page"),
        "HTML should contain page title"
    );
    assert!(
        content.contains("Inline Test"),
        "HTML should contain dashboard title"
    );
}

/// Inline HTML must embed the Bokeh JS directly (look for the Bokeh global).
#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_html_embeds_bokeh_js() {
    let tmp = TempDir::new().unwrap();
    make_inline_dash(tmp.path().to_str().unwrap())
        .render_native(rust_to_bokeh::BokehResources::Inline)
        .unwrap();

    let content = std::fs::read_to_string(tmp.path().join("inline-page.html")).unwrap();
    // Bokeh's minified JS includes its copyright header — confirm it's present
    assert!(
        content.contains("Bokeh Contributors"),
        "Inline HTML should embed Bokeh JS (expected 'Bokeh Contributors' copyright in output)"
    );
}

// ── navigation categories ─────────────────────────────────────────────────────

#[test]
#[cfg(feature = "python")]
fn categorised_pages_all_created() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();

    for (slug, cat) in &[
        ("fin-one", "Finance"),
        ("fin-two", "Finance"),
        ("ops-one", "Ops"),
    ] {
        let cfg = HBarConfig::builder()
            .category("category")
            .value("value")
            .x_label("V")
            .build()
            .unwrap();
        dash.add_page(
            PageBuilder::new(slug, slug, slug, 1)
                .category(cat)
                .chart(ChartSpecBuilder::hbar("C", &h("data"), cfg).at(0, 0, 1).build())
                .build()
                .unwrap(),
        );
    }

    dash.render().unwrap();

    for slug in &["fin-one", "fin-two", "ops-one"] {
        assert!(
            tmp.path().join(format!("{slug}.html")).exists(),
            "{slug}.html should be created"
        );
    }
}

// ── native renderer — chart types ────────────────────────────────────────────
//
// Every chart type is tested against native CDN rendering.  Each test verifies
// the output HTML is produced and contains key Bokeh model names that prove the
// chart was serialised correctly.

#[test]
fn native_cdn_line_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_timeseries_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("ts", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("line-test", "Line Test", "Lines", 1)
            .chart(
                ChartSpecBuilder::line(
                    "Series Over Time",
                    &h("ts"),
                    LineConfig::builder()
                        .x("month")
                        .y_cols(&["series_a", "series_b"])
                        .y_label("Value")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("line-test.html")).unwrap();
    assert!(content.contains("FactorRange"), "string x-data should use FactorRange");
    assert!(content.contains("Jan"), "should contain x-value 'Jan'");
    assert!(content.contains("series_a"), "should contain series name");
    assert!(content.contains("Line"), "should contain Line glyph");
    assert!(content.contains("Legend"), "should contain Legend");
}

#[test]
fn native_cdn_grouped_bar_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_grouped_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("grouped", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("bar-test", "Bar Test", "Bars", 1)
            .chart(
                ChartSpecBuilder::bar(
                    "Grouped Bars",
                    &h("grouped"),
                    GroupedBarConfig::builder()
                        .x("month")
                        .group("category")
                        .value("value")
                        .y_label("USD")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("bar-test.html")).unwrap();
    assert!(content.contains("VBar"), "should contain VBar glyph");
    assert!(content.contains("FactorRange"), "grouped bar needs FactorRange");
    assert!(content.contains("Jan"), "should contain x-value 'Jan'");
}

#[test]
fn native_cdn_scatter_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = df![
        "x_vals" => [1.0f64, 2.0, 3.0, 4.0],
        "y_vals" => [4.0f64, 3.0, 2.0, 1.0],
    ]
    .unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("scatter_data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("scatter-test", "Scatter Test", "Scatter", 1)
            .chart(
                ChartSpecBuilder::scatter(
                    "X vs Y",
                    &h("scatter_data"),
                    ScatterConfig::builder()
                        .x("x_vals")
                        .y("y_vals")
                        .x_label("X")
                        .y_label("Y")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("scatter-test.html")).unwrap();
    assert!(content.contains("Scatter"), "should contain Scatter glyph");
    assert!(content.contains("x_vals"), "should contain x column name");
}

#[test]
fn native_cdn_pie_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("pie_data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("pie-test", "Pie Test", "Pie", 1)
            .chart(
                ChartSpecBuilder::pie(
                    "Share",
                    &h("pie_data"),
                    PieConfig::builder()
                        .label("category")
                        .value("value")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("pie-test.html")).unwrap();
    assert!(content.contains("AnnularWedge"), "pie chart should use AnnularWedge");
    assert!(content.contains("Alpha"), "should contain label 'Alpha'");
}

#[test]
fn native_cdn_histogram_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let raw = df![
        "salary" => [50.0f64, 55.0, 60.0, 65.0, 70.0, 75.0, 80.0, 85.0, 90.0, 95.0],
    ]
    .unwrap();
    let mut hist = compute_histogram(&raw, "salary", 5).unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("hist_data", &mut hist).unwrap();
    dash.add_page(
        PageBuilder::new("hist-test", "Histogram Test", "Hist", 1)
            .chart(
                ChartSpecBuilder::histogram(
                    "Salary Distribution",
                    &h("hist_data"),
                    HistogramConfig::builder()
                        .x_label("Salary")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("hist-test.html")).unwrap();
    assert!(content.contains("Quad"), "histogram should use Quad glyph");
}

#[test]
fn native_cdn_box_plot_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let raw = df![
        "department" => ["A","A","A","A","A","B","B","B","B","B"],
        "salary_k"  => [50.0f64, 55.0, 60.0, 65.0, 70.0, 80.0, 85.0, 90.0, 95.0, 100.0],
    ]
    .unwrap();
    let mut box_stats = compute_box_stats(&raw, "department", "salary_k").unwrap();
    let mut outliers = compute_box_outliers(&raw, "department", "salary_k").unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("box_data", &mut box_stats).unwrap();
    dash.add_df("box_outliers", &mut outliers).unwrap();
    dash.add_page(
        PageBuilder::new("box-test", "Box Plot Test", "Box", 1)
            .chart(
                ChartSpecBuilder::box_plot(
                    "Salary by Dept",
                    &h("box_data"),
                    BoxPlotConfig::builder()
                        .category("department")
                        .q1("q1")
                        .q2("q2")
                        .q3("q3")
                        .lower("lower")
                        .upper("upper")
                        .y_label("Salary K")
                        .outlier_source("box_outliers")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("box-test.html")).unwrap();
    assert!(content.contains("VBar"), "box plot body uses VBar");
    assert!(content.contains("Segment"), "box plot whiskers use Segment");
}

#[test]
fn native_cdn_density_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = df![
        "group" => ["A","A","A","A","A","B","B","B","B","B"],
        "score" => [1.0f64, 2.0, 3.0, 4.0, 5.0, 2.0, 3.0, 4.0, 5.0, 6.0],
    ]
    .unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("density_data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("density-test", "Density Test", "Density", 1)
            .chart(
                ChartSpecBuilder::density(
                    "Score Density",
                    &h("density_data"),
                    DensityConfig::builder()
                        .category("group")
                        .value("score")
                        .y_label("Score")
                        .build()
                        .unwrap(),
                )
                .at(0, 0, 1)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let content = std::fs::read_to_string(tmp.path().join("density-test.html")).unwrap();
    assert!(content.contains("FactorRange"), "density uses categorical y-axis");
    assert!(content.contains("Scatter"), "density plots use Scatter glyphs");
}

// ── native renderer — filter types ──────────────────────────────────────────
//
// Each filter type is tested to verify the widget model, filter model, and
// CustomJS callback are correctly serialised in the output HTML.

#[test]
fn native_cdn_range_filter() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("range-f", "Range Filter", "RF", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).filtered().build(),
            )
            .filter(FilterSpec::range(&h("data"), "value", "Value Range", 0.0, 50.0, 1.0))
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("range-f.html")).unwrap();
    assert!(c.contains("RangeSlider"), "should contain RangeSlider widget");
    assert!(c.contains("BooleanFilter"), "should contain BooleanFilter model");
    assert!(c.contains("CustomJS"), "should contain CustomJS callback");
    assert!(c.contains("HBar"), "chart should still render");
}

#[test]
fn native_cdn_select_filter() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("select-f", "Select Filter", "SF", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).filtered().build(),
            )
            .filter(FilterSpec::select(&h("data"), "category", "Pick", vec!["Alpha", "Beta", "Gamma"]))
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("select-f.html")).unwrap();
    assert!(c.contains("Select"), "should contain Select widget");
    assert!(c.contains("BooleanFilter"), "should contain BooleanFilter");
    assert!(c.contains("(All)"), "select should have (All) option");
}

#[test]
fn native_cdn_group_filter() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("group-f", "Group Filter", "GF", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).filtered().build(),
            )
            .filter(FilterSpec::group(&h("data"), "category", "Group By", vec!["Alpha", "Beta", "Gamma"]))
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("group-f.html")).unwrap();
    assert!(c.contains("GroupFilter"), "should contain GroupFilter model");
    assert!(c.contains("Select"), "group filter uses Select widget");
}

#[test]
fn native_cdn_threshold_filter() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("threshold-f", "Threshold Filter", "TF", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).filtered().build(),
            )
            .filter(FilterSpec::threshold(&h("data"), "value", "High Values", 15.0, true))
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("threshold-f.html")).unwrap();
    assert!(c.contains("Switch"), "threshold filter uses Switch widget");
    assert!(c.contains("BooleanFilter"), "should contain BooleanFilter");
}

#[test]
fn native_cdn_top_n_filter() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("topn-f", "TopN Filter", "TN", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).filtered().build(),
            )
            .filter(FilterSpec::top_n(&h("data"), "value", "Top N", 3, true))
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("topn-f.html")).unwrap();
    assert!(c.contains("Slider"), "top-n filter uses Slider widget");
    assert!(c.contains("IndexFilter"), "should contain IndexFilter model");
}

#[test]
fn native_cdn_date_range_filter() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    // epoch-ms column
    let mut df = df![
        "category" => ["A", "B", "C"],
        "value"    => [10.0f64, 20.0, 30.0],
        "ts_ms"    => [1_700_000_000_000.0f64, 1_700_100_000_000.0, 1_700_200_000_000.0],
    ]
    .unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("date-f", "Date Filter", "DF", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).filtered().build(),
            )
            .filter(FilterSpec::date_range(
                &h("data"), "ts_ms", "Date Range",
                1_700_000_000_000.0, 1_700_200_000_000.0,
                DateStep::Day, TimeScale::Days,
            ))
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("date-f.html")).unwrap();
    assert!(c.contains("DatetimeRangeSlider"), "should contain DatetimeRangeSlider");
    assert!(c.contains("BooleanFilter"), "should contain BooleanFilter");
}

#[test]
fn native_cdn_multiple_filters_on_same_source() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("multi-f", "Multi Filter", "MF", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).filtered().build(),
            )
            .filter(FilterSpec::range(&h("data"), "value", "Range", 0.0, 50.0, 1.0))
            .filter(FilterSpec::select(&h("data"), "category", "Pick", vec!["Alpha", "Beta", "Gamma"]))
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("multi-f.html")).unwrap();
    assert!(c.contains("IntersectionFilter"), "multiple filters should combine via IntersectionFilter");
    assert!(c.contains("RangeSlider"), "should contain RangeSlider");
    assert!(c.contains("Select"), "should contain Select");
    assert!(c.contains("HBar"), "chart should still render");
}

// ── native renderer — structural features ───────────────────────────────────

#[test]
fn native_cdn_paragraph_module() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("para-test", "Para Test", "Para", 1)
            .paragraph(
                ParagraphSpec::new("Hello from a paragraph.")
                    .title("About Section")
                    .at(0, 0, 1)
                    .build(),
            )
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(1, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("para-test.html")).unwrap();
    assert!(c.contains("Hello from a paragraph."), "should contain paragraph text");
    assert!(c.contains("About Section"), "should contain paragraph title");
    assert!(c.contains("HBar"), "chart should also render");
}

#[test]
fn native_cdn_table_module() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("table-test", "Table Test", "Table", 1)
            .table(
                TableSpec::new("Data Table", &h("data"))
                    .column(TableColumn::text("category", "Category"))
                    .column(TableColumn::number("value", "Value", 1))
                    .at(0, 0, 1)
                    .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("table-test.html")).unwrap();
    assert!(c.contains("<table>"), "should contain HTML table");
    assert!(c.contains("Alpha"), "table should show data values");
    assert!(c.contains("Category"), "table should show column headers");
}

#[test]
fn native_cdn_vertical_nav() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new()
        .output_dir(&out)
        .nav_style(NavStyle::Vertical);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("vert-test", "Vertical Nav", "VNav", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("vert-test.html")).unwrap();
    assert!(c.contains("layout-vertical"), "should use vertical layout class");
    assert!(c.contains("nav-vertical"), "should contain vertical nav");
}

#[test]
fn native_cdn_dashboard_title() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new()
        .output_dir(&out)
        .title("Acme Corp Dashboard");
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("title-test", "Page Title", "PT", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("title-test.html")).unwrap();
    assert!(c.contains("Acme Corp Dashboard"), "should contain dashboard title");
    assert!(c.contains("Page Title"), "should contain page title");
}

#[test]
fn native_cdn_categorised_pages() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    for (slug, cat) in &[("fin-a", "Finance"), ("fin-b", "Finance"), ("ops-a", "Ops")] {
        let cfg = HBarConfig::builder()
            .category("category").value("value").x_label("V").build().unwrap();
        dash.add_page(
            PageBuilder::new(slug, slug, slug, 1)
                .category(cat)
                .chart(ChartSpecBuilder::hbar("C", &h("data"), cfg).at(0, 0, 1).build())
                .build()
                .unwrap(),
        );
    }
    dash.render_native(BokehResources::Cdn).unwrap();

    for slug in &["fin-a", "fin-b", "ops-a"] {
        assert!(tmp.path().join(format!("{slug}.html")).exists(), "{slug}.html should exist");
    }
    // Category labels should appear in the nav
    let c = std::fs::read_to_string(tmp.path().join("fin-a.html")).unwrap();
    assert!(c.contains("Finance"), "nav should contain category 'Finance'");
    assert!(c.contains("Ops"), "nav should contain category 'Ops'");
}

#[test]
fn native_cdn_custom_dimensions() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("dim-test", "Dimensions", "Dim", 2)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                )
                .at(0, 0, 1)
                .dimensions(500, 300)
                .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Cdn).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("dim-test.html")).unwrap();
    // Fixed dimensions use "fixed" sizing_mode
    assert!(c.contains("fixed"), "custom dimensions should use fixed sizing mode");
}

// ── native inline — chart types ─────────────────────────────────────────────
//
// These tests verify the Inline rendering path produces the same chart types
// correctly.  They require the `bokeh-inline` Cargo feature.

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_line_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_timeseries_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("ts", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("line-inline", "Line Inline", "Lines", 1)
            .chart(
                ChartSpecBuilder::line(
                    "Series", &h("ts"),
                    LineConfig::builder().x("month").y_cols(&["series_a", "series_b"]).y_label("V").build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("line-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("Line"), "should contain Line glyph");
    assert!(c.contains("series_a"), "should contain series data");
}

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_grouped_bar_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_grouped_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("grouped", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("bar-inline", "Bar Inline", "Bars", 1)
            .chart(
                ChartSpecBuilder::bar(
                    "Grouped Bars", &h("grouped"),
                    GroupedBarConfig::builder().x("month").group("category").value("value").y_label("V").build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("bar-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("VBar"), "should contain VBar glyph");
}

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_scatter_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = df!["x" => [1.0f64, 2.0], "y" => [3.0f64, 4.0]].unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("sc", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("scatter-inline", "Scatter Inline", "Sc", 1)
            .chart(
                ChartSpecBuilder::scatter(
                    "XY", &h("sc"),
                    ScatterConfig::builder().x("x").y("y").x_label("X").y_label("Y").build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("scatter-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("Scatter"), "should contain Scatter glyph");
}

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_pie_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("pie", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("pie-inline", "Pie Inline", "Pie", 1)
            .chart(
                ChartSpecBuilder::pie(
                    "Share", &h("pie"),
                    PieConfig::builder().label("category").value("value").build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("pie-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("AnnularWedge"), "should contain AnnularWedge");
}

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_histogram_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let raw = df!["val" => [1.0f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]].unwrap();
    let mut hist = compute_histogram(&raw, "val", 4).unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("h", &mut hist).unwrap();
    dash.add_page(
        PageBuilder::new("hist-inline", "Hist Inline", "Hist", 1)
            .chart(
                ChartSpecBuilder::histogram(
                    "Distribution", &h("h"),
                    HistogramConfig::builder().x_label("Val").build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("hist-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("Quad"), "should contain Quad glyph");
}

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_box_plot_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let raw = df![
        "dept" => ["X","X","X","X","X","Y","Y","Y","Y","Y"],
        "val"  => [10.0f64, 20.0, 30.0, 40.0, 50.0, 15.0, 25.0, 35.0, 45.0, 55.0],
    ].unwrap();
    let mut stats = compute_box_stats(&raw, "dept", "val").unwrap();
    let mut outs = compute_box_outliers(&raw, "dept", "val").unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("bs", &mut stats).unwrap();
    dash.add_df("bo", &mut outs).unwrap();
    dash.add_page(
        PageBuilder::new("box-inline", "Box Inline", "Box", 1)
            .chart(
                ChartSpecBuilder::box_plot(
                    "Stats", &h("bs"),
                    BoxPlotConfig::builder()
                        .category("dept").q1("q1").q2("q2").q3("q3")
                        .lower("lower").upper("upper").y_label("V")
                        .outlier_source("bo")
                        .build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("box-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("VBar"), "box plot should contain VBar");
}

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_density_chart() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = df![
        "grp" => ["A","A","A","B","B","B"],
        "v"   => [1.0f64, 2.0, 3.0, 2.0, 3.0, 4.0],
    ].unwrap();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("d", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("dens-inline", "Density Inline", "Dens", 1)
            .chart(
                ChartSpecBuilder::density(
                    "Density", &h("d"),
                    DensityConfig::builder().category("grp").value("v").y_label("V").build().unwrap(),
                ).at(0, 0, 1).build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("dens-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("Scatter"), "density should contain Scatter");
}

// ── native inline — filters and structure ───────────────────────────────────

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_filtered_page() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("filt-inline", "Filtered Inline", "FI", 1)
            .chart(
                ChartSpecBuilder::hbar(
                    "Chart", &h("data"),
                    HBarConfig::builder().category("category").value("value").x_label("V").build().unwrap(),
                ).at(0, 0, 1).filtered().build(),
            )
            .filter(FilterSpec::range(&h("data"), "value", "Range", 0.0, 50.0, 1.0))
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("filt-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("BooleanFilter"), "should contain BooleanFilter");
    assert!(c.contains("RangeSlider"), "should contain RangeSlider");
    assert!(c.contains("HBar"), "chart should render");
}

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_paragraph_and_table() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("mixed-inline", "Mixed Inline", "Mix", 1)
            .paragraph(
                ParagraphSpec::new("Inline paragraph.").title("Info").at(0, 0, 1).build(),
            )
            .table(
                TableSpec::new("Inline Table", &h("data"))
                    .column(TableColumn::text("category", "Cat"))
                    .column(TableColumn::number("value", "Val", 1))
                    .at(1, 0, 1)
                    .build(),
            )
            .build()
            .unwrap(),
    );
    dash.render_native(BokehResources::Inline).unwrap();

    let c = std::fs::read_to_string(tmp.path().join("mixed-inline.html")).unwrap();
    assert!(!c.contains("cdn.bokeh.org"), "inline should not reference CDN");
    assert!(c.contains("Inline paragraph."), "should contain paragraph text");
    assert!(c.contains("<table>"), "should contain HTML table");
    assert!(c.contains("Alpha"), "table should contain data value");
}

#[test]
#[cfg(feature = "bokeh-inline")]
fn native_inline_multi_page_with_vertical_nav() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new()
        .output_dir(&out)
        .title("Inline Multi")
        .nav_style(NavStyle::Vertical);
    dash.add_df("data", &mut df).unwrap();
    for slug in &["inline-p1", "inline-p2"] {
        let cfg = HBarConfig::builder()
            .category("category").value("value").x_label("V").build().unwrap();
        dash.add_page(
            PageBuilder::new(slug, slug, slug, 1)
                .chart(ChartSpecBuilder::hbar("C", &h("data"), cfg).at(0, 0, 1).build())
                .build()
                .unwrap(),
        );
    }
    dash.render_native(BokehResources::Inline).unwrap();

    for slug in &["inline-p1", "inline-p2"] {
        let c = std::fs::read_to_string(tmp.path().join(format!("{slug}.html"))).unwrap();
        assert!(!c.contains("cdn.bokeh.org"), "{slug} should not reference CDN");
        assert!(c.contains("nav-vertical"), "{slug} should have vertical nav");
        assert!(c.contains("Inline Multi"), "{slug} should contain dashboard title");
        // Cross-page links
        assert!(c.contains("inline-p1"), "{slug} should link to inline-p1");
        assert!(c.contains("inline-p2"), "{slug} should link to inline-p2");
    }
}

#[test]
fn print_native_json_for_inspection() {
    use polars::prelude::*;
    let tmp = tempfile::TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();
    let mut df = df![
        "category" => ["A", "B"],
        "value" => [10.0f64, 20.0],
    ].unwrap();
    let mut dash = rust_to_bokeh::Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        rust_to_bokeh::PageBuilder::new("test", "Test", "T", 1)
            .chart(
                rust_to_bokeh::ChartSpecBuilder::hbar(
                    "Values", &h("data"),
                    rust_to_bokeh::HBarConfig::builder()
                        .category("category").value("value").x_label("Val")
                        .build().unwrap()
                ).at(0,0,1).build()
            ).build().unwrap()
    );
    dash.render_native(rust_to_bokeh::BokehResources::Cdn).unwrap();
    let content = std::fs::read_to_string(tmp.path().join("test.html")).unwrap();
    // Extract docs_json
    if let Some(start) = content.find("const docs_json = '") {
        let s = &content[start + 19..];
        if let Some(end) = s.find("';\n") {
            println!("DOCS_JSON:\n{}", &s[..end.min(3000)]);
        }
    }
    if let Some(start) = content.find("const render_items = ") {
        let s = &content[start + 21..];
        if let Some(end) = s.find(";\n") {
            println!("RENDER_ITEMS:\n{}", &s[..end.min(500)]);
        }
    }
}
