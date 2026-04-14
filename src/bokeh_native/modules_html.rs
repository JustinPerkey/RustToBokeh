//! HTML rendering for non-chart page modules: paragraphs and data tables.

use polars::prelude::DataFrame;

use crate::modules::{ColumnFormat, ParagraphSpec, TableSpec};

use super::html;

pub(super) fn render_paragraph_html(para: &ParagraphSpec) -> String {
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

pub(super) fn render_table_html(spec: &TableSpec, df: &DataFrame) -> String {
    let mut html = String::from(r#"<div class="table-module"><div class="table-wrapper"><table>"#);

    html.push_str("<thead><tr>");
    for col in &spec.columns {
        html.push_str(&format!(
            "<th>{}</th>",
            html::escape_html(&col.label)
        ));
    }
    html.push_str("</tr></thead>");

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
        let decimal_part = &frac_str[2..];
        format!("{with_commas}.{decimal_part}")
    }
}
