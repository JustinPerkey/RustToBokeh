use rust_to_bokeh::prelude::*;

use crate::handles::Handles;

type C = ChartSpecBuilder;
type Bar = GroupedBarConfig;
type Line = LineConfig;
type HB = HBarConfig;
type Scat = ScatterConfig;

pub fn page_executive_summary(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("executive-summary", "Executive Summary", "Executive", 2)
        .chart(
            C::line(
                "Revenue & Profit Trends",
                &h.monthly_trends,
                Line::builder().x("month").y_cols(&["revenue", "profit"]).y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::hbar(
                "Market Position",
                &h.market_share,
                HB::builder().category("company").value("share").x_label("Market Share %").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::bar(
                "Quarterly Products",
                &h.quarterly_products,
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Profit",
                &h.scatter_performance,
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?,
            )
            .at(2, 0, 2)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::range(&h.scatter_performance, "revenue", "Revenue Range", 40.0, 320.0, 10.0))
        .build()
}
