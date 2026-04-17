use rust_to_bokeh::prelude::*;

use crate::handles::Handles;

type C = ChartSpecBuilder;
type Line = LineConfig;
type HB = HBarConfig;
type Scat = ScatterConfig;

pub fn page_project_portfolio(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("project-portfolio", "Project Portfolio", "Projects", 2)
        .category("Operations")
        .chart(
            C::hbar(
                "Project Completion Status",
                &h.project_status,
                HB::builder().category("project").value("completion").x_label("% Complete").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Employees",
                &h.scatter_performance,
                Scat::builder().x("revenue").y("employees").x_label("Revenue (k)").y_label("Team Size").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Profit vs Employees",
                &h.scatter_performance,
                Scat::builder().x("profit").y("employees").x_label("Profit (k)").y_label("Team Size").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::top_n(&h.scatter_performance, "revenue", "Top N by Revenue", 30, true))
        .build()
}

pub fn page_cost_optimization(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("cost-optimization", "Cost Optimization", "Costs", 2)
        .category("Operations")
        .chart(
            C::hbar(
                "Spending by Category",
                &h.cost_breakdown,
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Expense vs Margin Trend",
                &h.monthly_trends,
                Line::builder().x("month").y_cols(&["expenses", "margin"]).y_label("Value").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Profit Efficiency",
                &h.scatter_performance,
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::threshold(
            &h.scatter_performance,
            "profit",
            "Profitable Only (>30k)",
            30.0,
            true,
        ))
        .build()
}

pub fn page_forecast_targets(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("forecast-targets", "Forecast & Targets", "Forecast", 2)
        .category("Operations")
        .chart(
            C::line(
                "Monthly Forecast",
                &h.monthly_trends,
                Line::builder()
                    .x("month")
                    .y_cols(&["revenue", "expenses", "profit"])
                    .y_label("USD (k)")
                    .build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Quarterly Outlook",
                &h.quarterly_trends,
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Target Completion",
                &h.project_status,
                HB::builder().category("project").value("completion").x_label("% Complete").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

pub fn page_operations_dashboard(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("operations-dashboard", "Operations Dashboard", "Operations", 3)
        .category("Operations")
        .chart(
            C::hbar(
                "Project Status",
                &h.project_status,
                HB::builder().category("project").value("completion").x_label("% Complete").build()?,
            )
            .at(0, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Cost Breakdown",
                &h.cost_breakdown,
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(0, 1, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Satisfaction",
                &h.satisfaction,
                HB::builder().category("category").value("score").x_label("Score").build()?,
            )
            .at(0, 2, 1)
            .build(),
        )
        .chart(
            C::line(
                "Traffic & Signups",
                &h.website_traffic,
                Line::builder().x("month").y_cols(&["visitors", "signups"]).y_label("Count").build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Team Efficiency",
                &h.scatter_performance,
                Scat::builder().x("employees").y("profit").x_label("Team Size").y_label("Profit (k)").build()?,
            )
            .at(1, 2, 1)
            .build(),
        )
        .build()
}
