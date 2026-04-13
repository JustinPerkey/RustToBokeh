//! Native Rust → Bokeh HTML rendering path.
//!
//! Generates Bokeh-compatible HTML dashboards directly from Rust, with no
//! Python / PyO3 dependency at runtime. The output is functionally equivalent
//! to the Python path (`Dashboard::render`).
//!
//! ## Usage
//!
//! ```ignore
//! use rust_to_bokeh::{Dashboard, BokehResources};
//!
//! dash.render_native(BokehResources::Cdn)?;      // CDN (requires internet)
//! dash.render_native(BokehResources::Inline)?;   // offline (same, CDN fallback for now)
//! ```

pub mod axis;
pub mod charts;
pub mod document;
pub mod figure;
pub mod filters;
pub mod html;
pub mod id_gen;
pub mod model;
pub mod nav;
pub mod palette;
pub mod source;

use std::collections::HashMap;
use std::io::Cursor;

use polars::io::ipc::IpcReader;
use polars::io::SerReader;
use polars::prelude::DataFrame;

use crate::error::ChartError;
use crate::modules::{ColumnFormat, PageModule, TableSpec};
use crate::pages::Page;
use crate::NavStyle;

use self::charts::build_chart_obj;
use self::document::{BokehDocument, build_embed_script};
use self::filters::{build_filter_widgets, combine_filters, FilterOutput};
use self::html::{FilterWidgetItem, GridItem, PageHtmlData, render_page_html};
use self::id_gen::IdGen;
use self::model::{BokehObject, BokehValue};
use self::nav::build_nav_html;

// ── BokehResources ────────────────────────────────────────────────────────────

/// Controls how Bokeh JS/CSS is delivered in the generated HTML.
///
/// - `Cdn` — small files, requires internet to view charts.
/// - `Inline` — larger HTML, works completely offline.
#[derive(Clone, Copy, Debug, Default)]
pub enum BokehResources {
    /// Load Bokeh from cdn.bokeh.org — small HTML, requires internet.
    #[default]
    Cdn,
    /// Embed Bokeh JS/CSS inline — larger HTML, works offline.
    Inline,
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Render a complete multi-page dashboard to HTML without Python.
///
/// `frame_data` is a list of `(key, Arrow-IPC-bytes)` pairs as produced by
/// [`crate::serialize_df`]. `pages` is the page list from the Dashboard.
pub fn render_native_dashboard(
    frame_data: &[(&str, Vec<u8>)],
    pages: &[Page],
    output_dir: &str,
    report_title: &str,
    nav_style: NavStyle,
    resources: BokehResources,
) -> Result<(), ChartError> {
    // Deserialize all Arrow IPC frames
    let mut frames: HashMap<String, DataFrame> = HashMap::new();
    for (key, bytes) in frame_data {
        let df = IpcReader::new(Cursor::new(bytes.as_slice()))
            .finish()
            .map_err(|e| ChartError::NativeRender(format!("IPC decode '{}': {}", key, e)))?;
        frames.insert(key.to_string(), df);
    }

    // Create output directory
    std::fs::create_dir_all(output_dir)
        .map_err(|e| ChartError::NativeRender(format!("create_dir_all '{}': {}", output_dir, e)))?;

    // Bokeh JS/CSS resource block
    let bokeh_resources_html = match resources {
        BokehResources::Cdn => html::bokeh_cdn_resources(),
        BokehResources::Inline => {
            #[cfg(feature = "bokeh-inline")]
            { html::bokeh_inline_resources() }
            #[cfg(not(feature = "bokeh-inline"))]
            {
                return Err(ChartError::NativeRender(
                    "BokehResources::Inline requires the 'bokeh-inline' Cargo feature. \
                     Run `bash scripts/setup_vendor.sh` then rebuild with \
                     `cargo build --features bokeh-inline`."
                        .into(),
                ));
            }
        }
    };

    let nav_style_str = match nav_style {
        NavStyle::Horizontal => "horizontal",
        NavStyle::Vertical => "vertical",
    };

    // Render each page
    for page in pages {
        let page_html = render_page(
            page,
            pages,
            &frames,
            report_title,
            nav_style_str,
            &bokeh_resources_html,
        )?;

        let path = format!("{}/{}.html", output_dir, page.slug);
        std::fs::write(&path, &page_html)
            .map_err(|e| ChartError::NativeRender(format!("write '{}': {}", path, e)))?;
    }

    Ok(())
}

// ── Page renderer ─────────────────────────────────────────────────────────────

fn render_page(
    page: &Page,
    all_pages: &[Page],
    frames: &HashMap<String, DataFrame>,
    report_title: &str,
    nav_style_str: &str,
    bokeh_resources_html: &str,
) -> Result<String, ChartError> {
    let mut id_gen = IdGen::new();
    let mut doc = BokehDocument::new();

    // ── 1. Build filter widgets ─────────────────────────────────────────────
    let (cds_filter_outputs, range_tool_outputs) =
        build_filter_widgets(&mut id_gen, &page.filters, frames)?;

    // Range tool x-range ID (for chart x-axis synchronisation)
    let range_tool_x_range_id: Option<String> = range_tool_outputs
        .first()
        .and_then(|o| o.range_tool_range_id.clone());

    // Collect filter objects per source_key for combine_filters
    let mut filter_objs_by_source: HashMap<String, Vec<BokehObject>> = HashMap::new();
    for fo in &cds_filter_outputs {
        filter_objs_by_source
            .entry(fo.source_key.clone())
            .or_default()
            .push(fo.filter_obj.clone());
    }
    // Range tool filters also get wired up if charts are `filtered`
    for fo in &range_tool_outputs {
        filter_objs_by_source
            .entry(fo.source_key.clone())
            .or_default()
            .push(fo.filter_obj.clone());
    }

    // ── 2. Build chart figures (don't add to doc yet) ───────────────────────
    struct ChartInfo {
        fig: BokehObject,
        grid: crate::charts::GridCell,
        title: String,
        source_key: String,
        cds_id: Option<String>,
    }

    let mut chart_infos: Vec<ChartInfo> = Vec::new();
    let mut source_key_to_cds_id: HashMap<String, String> = HashMap::new();

    for module in &page.modules {
        let PageModule::Chart(spec) = module else { continue };

        let filter_ref = if spec.filtered {
            let objs = filter_objs_by_source
                .get(&spec.source_key)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            Some(combine_filters(&mut id_gen, objs))
        } else {
            None
        };

        let fig = build_chart_obj(
            &mut id_gen,
            spec,
            frames,
            filter_ref,
            range_tool_x_range_id.as_deref(),
        )?;

        let cds_id = extract_first_cds_id(&fig);
        if let Some(ref id) = cds_id {
            source_key_to_cds_id
                .entry(spec.source_key.clone())
                .or_insert_with(|| id.clone());
        }

        chart_infos.push(ChartInfo {
            fig,
            grid: crate::charts::GridCell {
                row: spec.grid.row,
                col: spec.grid.col,
                col_span: spec.grid.col_span,
            },
            title: spec.title.clone(),
            source_key: spec.source_key.clone(),
            cds_id,
        });
    }

    // ── 3. Patch CDS placeholder IDs in filter widgets ──────────────────────
    let mut patched_cds_filters: Vec<FilterOutput> = cds_filter_outputs;
    let mut patched_range_tools: Vec<FilterOutput> = range_tool_outputs;

    for fo in &mut patched_cds_filters {
        if let Some(real_id) = source_key_to_cds_id.get(&fo.source_key) {
            let placeholder = format!("__cds_{}", fo.source_key);
            replace_placeholder_in_obj(&mut fo.widget, &placeholder, real_id);
        }
    }
    for fo in &mut patched_range_tools {
        if let Some(real_id) = source_key_to_cds_id.get(&fo.source_key) {
            let placeholder = format!("__cds_{}", fo.source_key);
            replace_placeholder_in_obj(&mut fo.widget, &placeholder, real_id);
        }
    }

    // ── 3b. Filter object embedding strategy ──────────────────────────────
    // Filter objects (BooleanFilter, IndexFilter, GroupFilter) are embedded
    // inline in BOTH the widget's CustomJS args AND the chart's CDSView
    // filter attribute (same model ID in both locations). BokehJS
    // recognises duplicate IDs as the same model instance, so the widget
    // callback mutates the same filter the chart observes.

    // ── 4. Add Range1d widgets (range tool) to doc ──────────────────────────
    for fo in &patched_range_tools {
        doc.add_root_no_div(fo.widget.clone());
    }

    // ── 5. Add filter widgets to doc and collect div UUIDs ──────────────────
    let mut filter_widget_items: Vec<FilterWidgetItem> = Vec::new();
    for fo in &patched_cds_filters {
        let div_uuid = doc.add_root(fo.widget.clone());
        filter_widget_items.push(FilterWidgetItem {
            div: format!(r#"<div id="{div_uuid}"></div>"#),
            label: fo.switch_label.clone(),
        });
    }

    // ── 6. Add range tool overview figures to doc ───────────────────────────
    let mut range_overview_divs: Vec<String> = Vec::new();
    for fo in &patched_range_tools {
        if let Some(overview) = &fo.range_tool_overview {
            let div_uuid = doc.add_root(overview.clone());
            range_overview_divs.push(format!(r#"<div id="{div_uuid}"></div>"#));
        }
    }

    // ── 7. Add chart figures to doc and collect grid items ──────────────────
    let mut grid_items: Vec<GridItem> = Vec::new();

    for info in chart_infos {
        let div_uuid = doc.add_root(info.fig);
        grid_items.push(GridItem {
            grid_row: info.grid.row + 1,  // CSS grid is 1-based
            grid_col: info.grid.col + 1,
            grid_col_span: info.grid.col_span,
            title: info.title,
            content: format!(r#"<div id="{div_uuid}"></div>"#),
            is_chart: true,
        });
    }

    // ── 8. Add paragraph and table modules ──────────────────────────────────
    for module in &page.modules {
        match module {
            PageModule::Chart(_) => {} // already handled
            PageModule::Paragraph(para) => {
                let content = render_paragraph_html(para);
                grid_items.push(GridItem {
                    grid_row: para.grid.row + 1,
                    grid_col: para.grid.col + 1,
                    grid_col_span: para.grid.col_span,
                    title: para.title.clone().unwrap_or_default(),
                    content,
                    is_chart: false,
                });
            }
            PageModule::Table(table) => {
                let df = frames.get(&table.source_key).ok_or_else(|| {
                    ChartError::NativeRender(format!(
                        "Table source_key '{}' not found",
                        table.source_key
                    ))
                })?;
                let content = render_table_html(table, df);
                grid_items.push(GridItem {
                    grid_row: table.grid.row + 1,
                    grid_col: table.grid.col + 1,
                    grid_col_span: table.grid.col_span,
                    title: table.title.clone(),
                    content,
                    is_chart: false,
                });
            }
        }
    }

    // Sort grid items by (row, col) so they appear in reading order in the HTML
    grid_items.sort_by_key(|i| (i.grid_row, i.grid_col));

    // ── 9. Generate embed script ─────────────────────────────────────────────
    let docs_json = doc.to_docs_json(&mut id_gen);
    let render_items = doc.to_render_items();
    let embed_script = build_embed_script(&docs_json, &render_items);

    // ── 10. Build nav HTML ───────────────────────────────────────────────────
    let nav_html = build_nav_html(all_pages, report_title, nav_style_str, &page.slug);

    // ── 11. Assemble page HTML ───────────────────────────────────────────────
    let page_data = PageHtmlData {
        title: &page.title,
        grid_cols: page.grid_cols,
        report_title,
        nav_html: &nav_html,
        nav_style: nav_style_str,
        bokeh_resources_html,
        embed_script: &embed_script,
        grid_items,
        filter_widgets: filter_widget_items,
        range_tool_overviews: range_overview_divs,
    };

    Ok(render_page_html(&page_data))
}

// ── Utility: extract first CDS ID from a figure ───────────────────────────────

/// Walk a figure's renderers and return the ID of the first `ColumnDataSource`.
fn extract_first_cds_id(fig: &BokehObject) -> Option<String> {
    for (key, val) in &fig.attributes {
        if key == "renderers" {
            if let BokehValue::Array(renderers) = val {
                for renderer in renderers {
                    if let Some(id) = find_cds_id_in_value(renderer) {
                        return Some(id);
                    }
                }
            }
        }
    }
    None
}

fn find_cds_id_in_value(val: &BokehValue) -> Option<String> {
    if let BokehValue::Object(obj) = val {
        if obj.name == "GlyphRenderer" {
            for (k, v) in &obj.attributes {
                if k == "data_source" {
                    return match v {
                        BokehValue::Object(cds) => Some(cds.id.clone()),
                        BokehValue::Ref(id) => Some(id.clone()),
                        _ => None,
                    };
                }
            }
        }
    }
    None
}

// ── Utility: replace CDS placeholder IDs ─────────────────────────────────────

/// Recursively replace `BokehValue::Ref(placeholder)` with `BokehValue::Ref(real_id)`.
fn replace_placeholder(val: &mut BokehValue, placeholder: &str, real_id: &str) {
    match val {
        BokehValue::Ref(id) if id == placeholder => *id = real_id.to_string(),
        BokehValue::Array(arr) => {
            for v in arr {
                replace_placeholder(v, placeholder, real_id);
            }
        }
        BokehValue::Map(entries) => {
            for (_, v) in entries {
                replace_placeholder(v, placeholder, real_id);
            }
        }
        BokehValue::Object(obj) => {
            replace_placeholder_in_obj(obj, placeholder, real_id);
        }
        BokehValue::Value(inner) => replace_placeholder(inner, placeholder, real_id),
        _ => {}
    }
}

fn replace_placeholder_in_obj(obj: &mut BokehObject, placeholder: &str, real_id: &str) {
    for (_, v) in &mut obj.attributes {
        replace_placeholder(v, placeholder, real_id);
    }
}

// ── Utility: render paragraph HTML ───────────────────────────────────────────

fn render_paragraph_html(para: &crate::modules::ParagraphSpec) -> String {
    let mut html = String::from(r#"<div class="paragraph-module">"#);
    for paragraph in para.text.split("\n\n") {
        let trimmed = paragraph.trim();
        if !trimmed.is_empty() {
            html.push_str(&format!(
                "<p>{}</p>",
                html::escape_html(trimmed)
            ));
        }
    }
    html.push_str("</div>");
    html
}

// ── Utility: render table HTML ────────────────────────────────────────────────

fn render_table_html(spec: &TableSpec, df: &DataFrame) -> String {
    let mut html = String::from(r#"<div class="table-module"><div class="table-wrapper"><table>"#);

    // Header
    html.push_str("<thead><tr>");
    for col in &spec.columns {
        html.push_str(&format!(
            "<th>{}</th>",
            html::escape_html(&col.label)
        ));
    }
    html.push_str("</tr></thead>");

    // Rows
    let n = df.height();
    html.push_str("<tbody>");
    for row in 0..n {
        html.push_str("<tr>");
        for col_def in &spec.columns {
            let cell = if let Ok(series) = df.column(&col_def.key) {
                format_cell(series, row, &col_def.format)
            } else {
                String::new()
            };
            html.push_str(&format!("<td>{cell}</td>"));
        }
        html.push_str("</tr>");
    }
    html.push_str("</tbody></table></div></div>");
    html
}

fn format_cell(series: &polars::prelude::Column, row: usize, fmt: &ColumnFormat) -> String {
    use polars::prelude::*;

    let raw_val: Option<f64> = match series.dtype() {
        DataType::Float32 => series.f32().ok().and_then(|s| s.get(row)).map(|v| v as f64),
        DataType::Float64 => series.f64().ok().and_then(|s| s.get(row)),
        DataType::Int32 => series.i32().ok().and_then(|s| s.get(row)).map(|v| v as f64),
        DataType::Int64 => series.i64().ok().and_then(|s| s.get(row)).map(|v| v as f64),
        DataType::UInt32 => series.u32().ok().and_then(|s| s.get(row)).map(|v| v as f64),
        DataType::UInt64 => series.u64().ok().and_then(|s| s.get(row)).map(|v| v as f64),
        _ => None,
    };

    // String columns
    if raw_val.is_none() {
        if let Ok(ca) = series.str() {
            return ca.get(row).unwrap_or("").to_string();
        }
        return series.get(row).map(|v| format!("{v}")).unwrap_or_default();
    }

    let v = raw_val.unwrap_or(0.0);
    match fmt {
        ColumnFormat::Text => format!("{v}"),
        ColumnFormat::Number { decimals } => {
            format!("{:.prec$}", v, prec = *decimals as usize)
        }
        ColumnFormat::Currency { symbol, decimals } => {
            let abs = v.abs();
            let sign = if v < 0.0 { "-" } else { "" };
            let formatted = format_thousands(abs, *decimals as usize);
            format!("{sign}{symbol}{formatted}")
        }
        ColumnFormat::Percent { decimals } => {
            format!("{:.prec$}%", v, prec = *decimals as usize)
        }
    }
}

fn format_thousands(v: f64, decimals: usize) -> String {
    let int_part = v as u64;
    let frac = v - int_part as f64;

    // Format integer part with commas
    let int_str = int_part.to_string();
    let mut with_commas = String::new();
    for (i, ch) in int_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            with_commas.insert(0, ',');
        }
        with_commas.insert(0, ch);
    }

    if decimals == 0 {
        with_commas
    } else {
        let frac_str = format!("{:.prec$}", frac, prec = decimals);
        // frac_str starts with "0.", take the decimal part
        let decimal_part = &frac_str[2..];
        format!("{with_commas}.{decimal_part}")
    }
}
