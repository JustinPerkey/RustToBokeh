//! Native Rust → Bokeh HTML rendering path.
//!
//! Generates Bokeh-compatible HTML dashboards directly from Rust, with no
//! Python / PyO3 dependency at runtime. The output is functionally equivalent
//! to the Python path (`Dashboard::render`).
//!
//! ## Module layout
//!
//! - [`model`] / [`id_gen`] — Bokeh object/value representation and UUID generation.
//! - [`document`] — `BokehDocument` root collection and embed-script emission.
//! - [`figure`] — `Figure` + axes + toolbar + glyph renderer builders.
//! - [`axis`] / [`palette`] / [`source`] — axis, palette, `ColumnDataSource` helpers.
//! - [`charts`] — per-chart-type renderers dispatched from `ChartConfig`.
//! - [`filters`] — filter widget builders (RangeSlider, Select, Switch, …).
//! - [`html`] / [`nav`] / [`modules_html`] — HTML templating helpers.
//! - [`page`] — per-page assembly: wires charts + filters + modules into HTML.
//! - [`placeholder`] — CDS-ID placeholder rewriting across filter widget
//!   CustomJS args after real `ColumnDataSource` IDs are known.
//!
//! ## Usage
//!
//! ```ignore
//! use rust_to_bokeh::{Dashboard, BokehResources};
//!
//! dash.render_native(BokehResources::Cdn)?;      // CDN (requires internet)
//! dash.render_native(BokehResources::Inline)?;   // offline (requires bokeh-inline feature)
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

mod modules_html;
mod html_css_scripts;
mod page;
mod placeholder;
mod verifier;

use std::collections::HashMap;
use std::io::Cursor;

use polars::io::ipc::IpcReader;
use polars::io::SerReader;
use polars::prelude::DataFrame;

use crate::error::ChartError;
use crate::pages::Page;
use crate::NavStyle;

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
    let mut frames: HashMap<String, DataFrame> = HashMap::new();
    for (key, bytes) in frame_data {
        let df = IpcReader::new(Cursor::new(bytes.as_slice()))
            .finish()
            .map_err(|e| ChartError::NativeRender(format!("IPC decode '{}': {}", key, e)))?;
        frames.insert(key.to_string(), df);
    }

    std::fs::create_dir_all(output_dir)
        .map_err(|e| ChartError::NativeRender(format!("create_dir_all '{}': {}", output_dir, e)))?;

    let bokeh_resources_html = match resources {
        BokehResources::Cdn => html::bokeh_cdn_resources(),
        BokehResources::Inline => {
            #[cfg(bokeh_vendor_present)]
            { html::bokeh_inline_resources() }
            #[cfg(not(bokeh_vendor_present))]
            {
                return Err(ChartError::NativeRender(
                    "BokehResources::Inline requires vendor Bokeh assets. \
                     Run `bash scripts/setup_vendor.sh` then recompile."
                        .into(),
                ));
            }
        }
    };

    let nav_style_str = match nav_style {
        NavStyle::Horizontal => "horizontal",
        NavStyle::Vertical => "vertical",
    };

    for page in pages {
        let page_html = page::render_page(
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
