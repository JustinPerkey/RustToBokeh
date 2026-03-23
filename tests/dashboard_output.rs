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
                    "simple",
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
                    "simple",
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
                    "ts",
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
                    "grouped",
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
                    "scatter_data",
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
                .chart(ChartSpecBuilder::hbar("Chart", "data", cfg).at(0, 0, 1).build())
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
                .chart(ChartSpecBuilder::hbar("C", "data", cfg).at(0, 0, 1).build())
                .build()
                .unwrap(),
        );
    }

    dash.render().unwrap();

    // Each page should contain a link to the other page.
    let alpha_html = std::fs::read_to_string(tmp.path().join("alpha.html")).unwrap();
    let beta_html = std::fs::read_to_string(tmp.path().join("beta.html")).unwrap();

    assert!(alpha_html.contains("beta"), "alpha.html should link to beta page");
    assert!(beta_html.contains("alpha"), "beta.html should link to alpha page");
}

// ── output directory configuration ───────────────────────────────────────────

#[test]
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
                    "data",
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

    assert!(out.join("p.html").exists(), "HTML should be created in the nested output dir");
}

#[test]
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
                    "data",
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
                    "data",
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
            .filter(FilterSpec::range("data", "value", "Value Range", 0.0, 50.0, 1.0))
            .build()
            .unwrap(),
    );
    dash.render().unwrap();

    assert!(tmp.path().join("filtered.html").exists());
}

#[test]
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
                    "data",
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
                "data",
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
                    "data",
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
                    "data",
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
fn page_with_table_module_creates_html() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();
    dash.add_page(
        PageBuilder::new("table-page", "Table Page", "Table", 1)
            .table(
                TableSpec::new("Simple Table", "data")
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
                    "data",
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

// ── navigation categories ─────────────────────────────────────────────────────

#[test]
fn categorised_pages_all_created() {
    let tmp = TempDir::new().unwrap();
    let out = tmp.path().to_str().unwrap().to_owned();

    let mut df = make_simple_df();
    let mut dash = Dashboard::new().output_dir(&out);
    dash.add_df("data", &mut df).unwrap();

    for (slug, cat) in &[("fin-one", "Finance"), ("fin-two", "Finance"), ("ops-one", "Ops")] {
        let cfg = HBarConfig::builder()
            .category("category")
            .value("value")
            .x_label("V")
            .build()
            .unwrap();
        dash.add_page(
            PageBuilder::new(slug, slug, slug, 1)
                .category(cat)
                .chart(ChartSpecBuilder::hbar("C", "data", cfg).at(0, 0, 1).build())
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
