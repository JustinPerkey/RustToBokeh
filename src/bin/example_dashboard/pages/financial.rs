use rust_to_bokeh::prelude::*;

use crate::handles::Handles;

type C = ChartSpecBuilder;
type Bar = GroupedBarConfig;
type Line = LineConfig;
type HB = HBarConfig;
type Scat = ScatterConfig;

pub fn page_revenue_overview(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("revenue-overview", "Revenue Overview", "Revenue", 2)
        .category("Financial")
        .chart(
            C::bar(
                "Monthly Revenue vs Expenses",
                &h.monthly_revenue,
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Revenue Trend",
                &h.monthly_trends,
                Line::builder().x("month").y_cols(&["revenue", "expenses"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Profit Margin",
                &h.monthly_trends,
                Line::builder().x("month").y_cols(&["margin"]).y_label("%").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .chart(
            C::bar(
                "Regional Sales",
                &h.regional_sales,
                Bar::builder().x("region").group("channel").value("value").y_label("USD (k)").build()?,
            )
            .at(2, 0, 2)
            .build(),
        )
        .build()
}

pub fn page_expense_analysis(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("expense-analysis", "Expense Analysis", "Expenses", 2)
        .category("Financial")
        .chart(
            C::hbar(
                "Cost Breakdown",
                &h.cost_breakdown,
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(0, 0, 1)
            .build(),
        )
        .chart(
            C::bar(
                "Budget vs Actual",
                &h.budget_vs_actual,
                Bar::builder().x("department").group("type").value("amount").y_label("USD (k)").build()?,
            )
            .at(0, 1, 1)
            .build(),
        )
        .chart(
            C::line(
                "Expense Trends",
                &h.monthly_trends,
                Line::builder().x("month").y_cols(&["expenses"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Margin Trend",
                &h.monthly_trends,
                Line::builder().x("month").y_cols(&["margin"]).y_label("%").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

pub fn page_quarterly_performance(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("quarterly-performance", "Quarterly Performance", "Quarterly", 2)
        .category("Financial")
        .chart(
            C::bar(
                "Product Revenue by Quarter",
                &h.quarterly_products,
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Quarterly Revenue & Costs",
                &h.quarterly_trends,
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Quarterly Margin",
                &h.quarterly_trends,
                Line::builder().x("quarter").y_cols(&["margin"]).y_label("%").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

pub fn page_budget_management(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("budget-management", "Budget Management", "Budget", 2)
        .category("Financial")
        .chart(
            C::bar(
                "Budget vs Actual Spending",
                &h.budget_vs_actual,
                Bar::builder().x("department").group("type").value("amount").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::hbar(
                "Cost Categories",
                &h.cost_breakdown,
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Revenue Trend",
                &h.monthly_trends,
                Line::builder().x("month").y_cols(&["revenue", "expenses"]).y_label("USD (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

pub fn page_financial_health(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("financial-health", "Financial Health", "Finance", 2)
        .category("Financial")
        .chart(
            C::line(
                "Quarterly Revenue, Costs & Margin",
                &h.quarterly_trends,
                Line::builder().x("quarter").y_cols(&["revenue", "costs", "margin"]).y_label("Value").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::bar(
                "Monthly Revenue vs Expenses",
                &h.monthly_revenue,
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Cost Structure",
                &h.cost_breakdown,
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .chart(
            C::scatter(
                "Profitability Map",
                &h.scatter_performance,
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?,
            )
            .at(2, 0, 2)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::select(
            &h.scatter_performance,
            "tier",
            "Company Tier",
            vec!["Small", "Medium", "Large"],
        ))
        .filter(FilterSpec::range(
            &h.scatter_performance,
            "employees",
            "Team Size Range",
            4.0,
            40.0,
            1.0,
        ))
        .build()
}

pub fn page_annual_review(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("annual-review", "Annual Review", "Annual", 2)
        .category("Financial")
        .chart(
            C::bar(
                "Monthly Revenue vs Expenses",
                &h.monthly_revenue,
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::bar(
                "Quarterly Product Performance",
                &h.quarterly_products,
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .chart(
            C::hbar(
                "Market Share",
                &h.market_share,
                HB::builder().category("company").value("share").x_label("%").build()?,
            )
            .at(2, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Satisfaction Scores",
                &h.satisfaction,
                HB::builder().category("category").value("score").x_label("Score").build()?,
            )
            .at(2, 1, 1)
            .build(),
        )
        .chart(
            C::line(
                "Full Year Trends",
                &h.monthly_trends,
                Line::builder()
                    .x("month")
                    .y_cols(&["revenue", "expenses", "profit", "margin"])
                    .y_label("Value")
                    .build()?,
            )
            .at(3, 0, 2)
            .build(),
        )
        .build()
}
