mod data;

use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Bar = GroupedBarConfig;
type Line = LineConfig;
type HB = HBarConfig;
type Scat = ScatterConfig;
type Para = ParagraphSpec;
type Tbl = TableSpec;
type TC = TableColumn;

// ── Page builders ─────────────────────────────────────────────────────────────

fn page_executive_summary() -> Result<Page, ChartError> {
    PageBuilder::new("executive-summary", "Executive Summary", "Executive", 2)
        .chart(
            C::line(
                "Revenue & Profit Trends",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "profit"]).y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::hbar(
                "Market Position",
                "market_share",
                HB::builder().category("company").value("share").x_label("Market Share %").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::bar(
                "Quarterly Products",
                "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Profit",
                "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?,
            )
            .at(2, 0, 2)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::range("scatter_performance", "revenue", "Revenue Range", 40.0, 320.0, 10.0))
        .build()
}

fn page_revenue_overview() -> Result<Page, ChartError> {
    PageBuilder::new("revenue-overview", "Revenue Overview", "Revenue", 2)
        .category("Financial")
        .chart(
            C::bar(
                "Monthly Revenue vs Expenses",
                "monthly_revenue",
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Revenue Trend",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "expenses"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Profit Margin",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["margin"]).y_label("%").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .chart(
            C::bar(
                "Regional Sales",
                "regional_sales",
                Bar::builder().x("region").group("channel").value("value").y_label("USD (k)").build()?,
            )
            .at(2, 0, 2)
            .build(),
        )
        .build()
}

fn page_expense_analysis() -> Result<Page, ChartError> {
    PageBuilder::new("expense-analysis", "Expense Analysis", "Expenses", 2)
        .category("Financial")
        .chart(
            C::hbar(
                "Cost Breakdown",
                "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(0, 0, 1)
            .build(),
        )
        .chart(
            C::bar(
                "Budget vs Actual",
                "budget_vs_actual",
                Bar::builder().x("department").group("type").value("amount").y_label("USD (k)").build()?,
            )
            .at(0, 1, 1)
            .build(),
        )
        .chart(
            C::line(
                "Expense Trends",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["expenses"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Margin Trend",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["margin"]).y_label("%").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

fn page_quarterly_performance() -> Result<Page, ChartError> {
    PageBuilder::new("quarterly-performance", "Quarterly Performance", "Quarterly", 2)
        .category("Financial")
        .chart(
            C::bar(
                "Product Revenue by Quarter",
                "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Quarterly Revenue & Costs",
                "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Quarterly Margin",
                "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["margin"]).y_label("%").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

fn page_product_analysis() -> Result<Page, ChartError> {
    PageBuilder::new("product-analysis", "Product Analysis", "Products", 2)
        .category("Commercial")
        .chart(
            C::bar(
                "Quarterly Product Revenue",
                "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Profit by Team",
                "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Satisfaction",
                "scatter_performance",
                Scat::builder().x("revenue").y("satisfaction").x_label("Revenue (k)").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::select(
            "scatter_performance",
            "tier",
            "Company Tier",
            vec!["Small", "Medium", "Large"],
        ))
        .filter(FilterSpec::range("scatter_performance", "revenue", "Revenue Range", 40.0, 320.0, 10.0))
        .build()
}

fn page_regional_breakdown() -> Result<Page, ChartError> {
    PageBuilder::new("regional-breakdown", "Regional Sales Breakdown", "Regions", 2)
        .category("Commercial")
        .chart(
            C::bar(
                "Sales by Region & Channel",
                "regional_sales",
                Bar::builder().x("region").group("channel").value("value").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::hbar(
                "Market Share",
                "market_share",
                HB::builder().category("company").value("share").x_label("%").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::scatter(
                "Employees vs Revenue",
                "scatter_performance",
                Scat::builder().x("employees").y("revenue").x_label("Team Size").y_label("Revenue (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

fn page_team_metrics() -> Result<Page, ChartError> {
    PageBuilder::new("team-metrics", "Team & Workforce Metrics", "Team", 2)
        .category("People")
        .chart(
            C::bar(
                "Department Headcount by Year",
                "dept_headcount",
                Bar::builder().x("department").group("year").value("count").y_label("Employees").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Employees vs Profit",
                "scatter_performance",
                Scat::builder().x("employees").y("profit").x_label("Team Size").y_label("Profit (k)").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Employees vs Satisfaction",
                "scatter_performance",
                Scat::builder().x("employees").y("satisfaction").x_label("Team Size").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::threshold(
            "scatter_performance",
            "satisfaction",
            "High Satisfaction Only (>4.2)",
            4.2,
            true,
        ))
        .build()
}

fn page_customer_insights() -> Result<Page, ChartError> {
    PageBuilder::new("customer-insights", "Customer Insights", "Customers", 2)
        .category("People")
        .chart(
            C::hbar(
                "Satisfaction Scores",
                "satisfaction",
                HB::builder().category("category").value("score").x_label("Score (1-5)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Customer Satisfaction",
                "scatter_performance",
                Scat::builder().x("revenue").y("satisfaction").x_label("Revenue (k)").y_label("Rating").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Profit vs Satisfaction",
                "scatter_performance",
                Scat::builder().x("profit").y("satisfaction").x_label("Profit (k)").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::group(
            "scatter_performance",
            "tier",
            "Company Tier",
            vec!["Small", "Medium", "Large"],
        ))
        .build()
}

fn page_web_analytics() -> Result<Page, ChartError> {
    PageBuilder::new("web-analytics", "Website Analytics", "Web", 2)
        .category("Digital")
        .chart(
            C::line(
                "Visitor Traffic",
                "website_traffic",
                Line::builder().x("month").y_cols(&["visitors"]).y_label("Visitors").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Signups Over Time",
                "website_traffic",
                Line::builder().x("month").y_cols(&["signups"]).y_label("Signups").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Conversions Over Time",
                "website_traffic",
                Line::builder().x("month").y_cols(&["conversions"]).y_label("Conversions").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

fn page_market_position() -> Result<Page, ChartError> {
    PageBuilder::new("market-position", "Market Position", "Market", 2)
        .category("Commercial")
        .chart(
            C::hbar(
                "Market Share",
                "market_share",
                HB::builder().category("company").value("share").x_label("Share %").build()?,
            )
            .at(0, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Project Completion",
                "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?,
            )
            .at(0, 1, 1)
            .build(),
        )
        .chart(
            C::line(
                "Revenue vs Costs (Quarterly)",
                "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .build()
}

fn page_budget_management() -> Result<Page, ChartError> {
    PageBuilder::new("budget-management", "Budget Management", "Budget", 2)
        .category("Financial")
        .chart(
            C::bar(
                "Budget vs Actual Spending",
                "budget_vs_actual",
                Bar::builder().x("department").group("type").value("amount").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::hbar(
                "Cost Categories",
                "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::line(
                "Revenue Trend",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "expenses"]).y_label("USD (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

fn page_project_portfolio() -> Result<Page, ChartError> {
    PageBuilder::new("project-portfolio", "Project Portfolio", "Projects", 2)
        .category("Operations")
        .chart(
            C::hbar(
                "Project Completion Status",
                "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Employees",
                "scatter_performance",
                Scat::builder().x("revenue").y("employees").x_label("Revenue (k)").y_label("Team Size").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Profit vs Employees",
                "scatter_performance",
                Scat::builder().x("profit").y("employees").x_label("Profit (k)").y_label("Team Size").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::top_n("scatter_performance", "revenue", "Top N by Revenue", 30, true))
        .build()
}

fn page_growth_indicators() -> Result<Page, ChartError> {
    PageBuilder::new("growth-indicators", "Growth Indicators", "Growth", 2)
        .category("Digital")
        .chart(
            C::line(
                "Revenue & Profit Growth",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "profit"]).y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Visitor Growth",
                "website_traffic",
                Line::builder().x("month").y_cols(&["visitors", "signups"]).y_label("Count").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::bar(
                "Quarterly Products",
                "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

fn page_cost_optimization() -> Result<Page, ChartError> {
    PageBuilder::new("cost-optimization", "Cost Optimization", "Costs", 2)
        .category("Operations")
        .chart(
            C::hbar(
                "Spending by Category",
                "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Expense vs Margin Trend",
                "monthly_trends",
                Line::builder().x("month").y_cols(&["expenses", "margin"]).y_label("Value").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Profit Efficiency",
                "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::threshold(
            "scatter_performance",
            "profit",
            "Profitable Only (>30k)",
            30.0,
            true,
        ))
        .build()
}

fn page_marketing_roi() -> Result<Page, ChartError> {
    PageBuilder::new("marketing-roi", "Marketing ROI", "Marketing", 2)
        .category("Digital")
        .chart(
            C::bar(
                "Channel Spend by Quarter",
                "marketing_channels",
                Bar::builder().x("quarter").group("channel").value("spend").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::line(
                "Website Conversions",
                "website_traffic",
                Line::builder().x("month").y_cols(&["signups", "conversions"]).y_label("Count").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Market Share",
                "market_share",
                HB::builder().category("company").value("share").x_label("%").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

fn page_operations_dashboard() -> Result<Page, ChartError> {
    PageBuilder::new("operations-dashboard", "Operations Dashboard", "Operations", 3)
        .category("Operations")
        .chart(
            C::hbar(
                "Project Status",
                "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?,
            )
            .at(0, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Cost Breakdown",
                "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(0, 1, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Satisfaction",
                "satisfaction",
                HB::builder().category("category").value("score").x_label("Score").build()?,
            )
            .at(0, 2, 1)
            .build(),
        )
        .chart(
            C::line(
                "Traffic & Signups",
                "website_traffic",
                Line::builder().x("month").y_cols(&["visitors", "signups"]).y_label("Count").build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Team Efficiency",
                "scatter_performance",
                Scat::builder().x("employees").y("profit").x_label("Team Size").y_label("Profit (k)").build()?,
            )
            .at(1, 2, 1)
            .build(),
        )
        .build()
}

fn page_financial_health() -> Result<Page, ChartError> {
    PageBuilder::new("financial-health", "Financial Health", "Finance", 2)
        .category("Financial")
        .chart(
            C::line(
                "Quarterly Revenue, Costs & Margin",
                "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs", "margin"]).y_label("Value").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::bar(
                "Monthly Revenue vs Expenses",
                "monthly_revenue",
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Cost Structure",
                "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .chart(
            C::scatter(
                "Profitability Map",
                "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?,
            )
            .at(2, 0, 2)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::select(
            "scatter_performance",
            "tier",
            "Company Tier",
            vec!["Small", "Medium", "Large"],
        ))
        .filter(FilterSpec::range(
            "scatter_performance",
            "employees",
            "Team Size Range",
            4.0,
            40.0,
            1.0,
        ))
        .build()
}

fn page_workforce_planning() -> Result<Page, ChartError> {
    PageBuilder::new("workforce-planning", "Workforce Planning", "Workforce", 2)
        .category("People")
        .chart(
            C::bar(
                "Headcount Growth",
                "dept_headcount",
                Bar::builder().x("department").group("year").value("count").y_label("Employees").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Team Size vs Revenue",
                "scatter_performance",
                Scat::builder().x("employees").y("revenue").x_label("Employees").y_label("Revenue (k)").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Team Size vs Satisfaction",
                "scatter_performance",
                Scat::builder().x("employees").y("satisfaction").x_label("Employees").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::hbar(
                "Budget by Department",
                "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(2, 0, 2)
            .build(),
        )
        .filter(FilterSpec::top_n("scatter_performance", "revenue", "Top N by Revenue", 30, true))
        .filter(FilterSpec::threshold(
            "scatter_performance",
            "satisfaction",
            "High Satisfaction Only (>4.0)",
            4.0,
            true,
        ))
        .build()
}

fn page_forecast_targets() -> Result<Page, ChartError> {
    PageBuilder::new("forecast-targets", "Forecast & Targets", "Forecast", 2)
        .category("Operations")
        .chart(
            C::line(
                "Monthly Forecast",
                "monthly_trends",
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
                "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?,
            )
            .at(1, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Target Completion",
                "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .build()
}

fn page_annual_review() -> Result<Page, ChartError> {
    PageBuilder::new("annual-review", "Annual Review", "Annual", 2)
        .category("Financial")
        .chart(
            C::bar(
                "Monthly Revenue vs Expenses",
                "monthly_revenue",
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::bar(
                "Quarterly Product Performance",
                "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .chart(
            C::hbar(
                "Market Share",
                "market_share",
                HB::builder().category("company").value("share").x_label("%").build()?,
            )
            .at(2, 0, 1)
            .build(),
        )
        .chart(
            C::hbar(
                "Satisfaction Scores",
                "satisfaction",
                HB::builder().category("category").value("score").x_label("Score").build()?,
            )
            .at(2, 1, 1)
            .build(),
        )
        .chart(
            C::line(
                "Full Year Trends",
                "monthly_trends",
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

fn page_module_showcase() -> Result<Page, ChartError> {
    PageBuilder::new("module-showcase", "Module Showcase", "Showcase", 2)
        .category("Reference")
        .paragraph(
            Para::new(
                "This page demonstrates the three content module types available \
                 in RustToBokeh: charts, paragraphs, and data tables.\n\n\
                 Paragraph modules render styled text blocks and support multiple \
                 paragraphs separated by blank lines. They are useful for adding \
                 context, annotations, or commentary alongside data visualisations.\n\n\
                 Table modules pull directly from any registered DataFrame and \
                 support per-column formatting: plain text, fixed-point numbers, \
                 currency with thousands separators, and percentage values.",
            )
            .title("About This Page")
            .at(0, 0, 2)
            .build(),
        )
        .table(
            Tbl::new("Monthly Revenue & Expenses", "monthly_revenue")
                .column(TC::text("month", "Month"))
                .column(TC::text("category", "Category"))
                .column(TC::currency("value", "Amount (k)", "$", 1))
                .at(1, 0, 1)
                .build(),
        )
        .chart(
            C::line(
                "Revenue Trend",
                "monthly_trends",
                Line::builder()
                    .x("month")
                    .y_cols(&["revenue", "expenses", "profit"])
                    .y_label("USD (k)")
                    .build()?,
            )
            .at(1, 1, 1)
            .build(),
        )
        .table(
            Tbl::new("Project Status", "project_status")
                .column(TC::text("project", "Project"))
                .column(TC::percent("completion", "Completion %", 0))
                .at(2, 0, 1)
                .build(),
        )
        .table(
            Tbl::new("Performance Snapshot", "scatter_performance")
                .column(TC::text("tier", "Tier"))
                .column(TC::currency("revenue", "Revenue (k)", "$", 0))
                .column(TC::number("profit", "Profit (k)", 1))
                .column(TC::number("employees", "Headcount", 0))
                .column(TC::number("satisfaction", "Satisfaction", 2))
                .at(2, 1, 1)
                .build(),
        )
        .build()
}

fn page_chart_customisation() -> Result<Page, ChartError> {
    PageBuilder::new(
        "chart-customisation",
        "Chart Customisation",
        "Customisation",
        2,
    )
    .category("Reference")
    // Scatter — custom marker, color, alpha, tooltip, axis ranges + bounds
    .chart(
        C::scatter(
            "Revenue vs Profit (styled)",
            "scatter_performance",
            Scat::builder()
                .x("revenue")
                .y("profit")
                .x_label("Revenue (k)")
                .y_label("Profit (k)")
                .color("#e74c3c")
                .marker("diamond")
                .marker_size(12.0)
                .alpha(0.85)
                .tooltips(
                    TooltipSpec::builder()
                        .field("tier", "Tier", TooltipFormat::Text)
                        .field("revenue", "Revenue", TooltipFormat::Currency)
                        .field("profit", "Profit", TooltipFormat::Number(Some(1)))
                        .field("satisfaction", "Satisfaction", TooltipFormat::Number(Some(2)))
                        .build(),
                )
                .x_axis(
                    AxisConfig::builder()
                        .range(0.0, 350.0)
                        .bounds(0.0, 400.0)
                        .tick_format("$0,0")
                        .build(),
                )
                .y_axis(
                    AxisConfig::builder()
                        .range(0.0, 100.0)
                        .bounds(0.0, 120.0)
                        .show_grid(false)
                        .build(),
                )
                .build()?,
        )
        .at(0, 0, 1)
        .build(),
    )
    // HBar — custom color, tick format, axis range + bounds on value axis
    .chart(
        C::hbar(
            "Market Share % (styled)",
            "market_share",
            HB::builder()
                .category("company")
                .value("share")
                .x_label("Share (%)")
                .color("#9b59b6")
                .tooltips(
                    TooltipSpec::builder()
                        .field("company", "Company", TooltipFormat::Text)
                        .field("share", "Share", TooltipFormat::Number(Some(1)))
                        .build(),
                )
                .x_axis(
                    AxisConfig::builder()
                        .range(0.0, 35.0)
                        .bounds(0.0, 40.0)
                        .tick_format("0.0")
                        .build(),
                )
                .build()?,
        )
        .at(0, 1, 1)
        .build(),
    )
    // Line — custom palette, line_width, point_size, y-axis tick format
    .chart(
        C::line(
            "Revenue & Profit Trends (styled)",
            "monthly_trends",
            Line::builder()
                .x("month")
                .y_cols(&["revenue", "profit", "expenses"])
                .y_label("USD (k)")
                .palette(PaletteSpec::Custom(vec![
                    "#2ecc71".into(),
                    "#e74c3c".into(),
                    "#3498db".into(),
                ]))
                .line_width(3.5)
                .point_size(9.0)
                .y_axis(
                    AxisConfig::builder()
                        .tick_format("$0,0")
                        .build(),
                )
                .build()?,
        )
        .at(1, 0, 1)
        .build(),
    )
    // Grouped bar — named Bokeh palette, narrower bars, label rotation on x
    .chart(
        C::bar(
            "Quarterly Products (styled)",
            "quarterly_products",
            Bar::builder()
                .x("quarter")
                .group("product")
                .value("value")
                .y_label("Revenue (k)")
                .palette(PaletteSpec::Named("Category20".into()))
                .bar_width(0.65)
                .y_axis(AxisConfig::builder().tick_format("$0,0").build())
                .build()?,
        )
        .at(1, 1, 1)
        .build(),
    )
    .build()
}

// ── DataFrame registration ────────────────────────────────────────────────────

fn register_dataframes(dash: &mut Dashboard) -> Result<(), ChartError> {
    dash.add_df("monthly_revenue", &mut data::build_monthly_revenue())?;
    dash.add_df("quarterly_products", &mut data::build_quarterly_products())?;
    dash.add_df("monthly_trends", &mut data::build_monthly_trends())?;
    dash.add_df("regional_sales", &mut data::build_regional_sales())?;
    dash.add_df("dept_headcount", &mut data::build_dept_headcount())?;
    dash.add_df("satisfaction", &mut data::build_satisfaction())?;
    dash.add_df("website_traffic", &mut data::build_website_traffic())?;
    dash.add_df("market_share", &mut data::build_market_share())?;
    dash.add_df("budget_vs_actual", &mut data::build_budget_vs_actual())?;
    dash.add_df("scatter_performance", &mut data::build_scatter_performance())?;
    dash.add_df("project_status", &mut data::build_project_status())?;
    dash.add_df("cost_breakdown", &mut data::build_cost_breakdown())?;
    dash.add_df("quarterly_trends", &mut data::build_quarterly_trends())?;
    dash.add_df("marketing_channels", &mut data::build_marketing_channels())?;
    Ok(())
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dash = Dashboard::new()
        .title("Annual Business Dashboard")
        .nav_style(NavStyle::Vertical);

    register_dataframes(&mut dash)?;

    dash.add_page(page_executive_summary()?);
    dash.add_page(page_revenue_overview()?);
    dash.add_page(page_expense_analysis()?);
    dash.add_page(page_quarterly_performance()?);
    dash.add_page(page_product_analysis()?);
    dash.add_page(page_regional_breakdown()?);
    dash.add_page(page_team_metrics()?);
    dash.add_page(page_customer_insights()?);
    dash.add_page(page_web_analytics()?);
    dash.add_page(page_market_position()?);
    dash.add_page(page_budget_management()?);
    dash.add_page(page_project_portfolio()?);
    dash.add_page(page_growth_indicators()?);
    dash.add_page(page_cost_optimization()?);
    dash.add_page(page_marketing_roi()?);
    dash.add_page(page_operations_dashboard()?);
    dash.add_page(page_financial_health()?);
    dash.add_page(page_workforce_planning()?);
    dash.add_page(page_forecast_targets()?);
    dash.add_page(page_annual_review()?);
    dash.add_page(page_module_showcase()?);
    dash.add_page(page_chart_customisation()?);

    dash.render()?;
    Ok(())
}
