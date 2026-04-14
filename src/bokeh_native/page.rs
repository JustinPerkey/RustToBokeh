//! Single-page assembly: builds filter widgets, charts, overview figures,
//! non-chart modules, wires CDS IDs, and produces the final HTML string.

use std::collections::HashMap;

use polars::prelude::DataFrame;

use crate::error::ChartError;
use crate::modules::PageModule;
use crate::pages::Page;

use super::charts::build_chart_obj;
use super::document::{build_embed_script, BokehDocument};
use super::filters::{build_filter_widgets, combine_filters, FilterOutput};
use super::html::{render_page_html, FilterWidgetItem, GridItem, PageHtmlData};
use super::id_gen::IdGen;
use super::model::BokehObject;
use super::modules_html::{render_paragraph_html, render_table_html};
use super::nav::build_nav_html;
use super::placeholder::{extract_first_cds_id, replace_placeholder_in_obj};

pub(super) fn render_page(
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

    let range_tool_x_range_id: Option<String> = range_tool_outputs
        .first()
        .and_then(|o| o.range_tool_range_id.clone());

    let mut filter_objs_by_source: HashMap<String, Vec<BokehObject>> = HashMap::new();
    for fo in &cds_filter_outputs {
        filter_objs_by_source
            .entry(fo.source_key.clone())
            .or_default()
            .push(fo.filter_obj.clone());
    }
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

        if let Some(cds_id) = extract_first_cds_id(&fig) {
            source_key_to_cds_id
                .entry(spec.source_key.clone())
                .or_insert(cds_id);
        }

        chart_infos.push(ChartInfo {
            fig,
            grid: crate::charts::GridCell {
                row: spec.grid.row,
                col: spec.grid.col,
                col_span: spec.grid.col_span,
            },
            title: spec.title.clone(),
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
    // filter attribute (same model ID in both locations). BokehJS recognises
    // duplicate IDs as the same model instance, so the widget callback
    // mutates the same filter the chart observes.

    // ── 4. Add Range1d widgets (range tool) to doc ──────────────────────────
    for fo in &patched_range_tools {
        doc.add_root_no_div(fo.widget.clone());
    }

    // ── 5. Add chart figures FIRST so nested CDS models are registered
    //      before any filter widget Ref to them is decoded. (BokehJS decodes
    //      roots in order; a Ref to an unknown ID aborts.)
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

    // ── 6. Add range tool overview figures to doc ───────────────────────────
    let mut range_overview_divs: Vec<String> = Vec::new();
    for fo in &patched_range_tools {
        if let Some(overview) = &fo.range_tool_overview {
            let div_uuid = doc.add_root(overview.clone());
            range_overview_divs.push(format!(r#"<div id="{div_uuid}"></div>"#));
        }
    }

    // ── 7. Add filter widgets to doc and collect div UUIDs ──────────────────
    let mut filter_widget_items: Vec<FilterWidgetItem> = Vec::new();
    for fo in &patched_cds_filters {
        let div_uuid = doc.add_root(fo.widget.clone());
        filter_widget_items.push(FilterWidgetItem {
            div: format!(r#"<div id="{div_uuid}"></div>"#),
            label: fo.switch_label.clone(),
        });
    }

    // ── 8. Add paragraph and table modules ──────────────────────────────────
    for module in &page.modules {
        match module {
            PageModule::Chart(_) => {}
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
