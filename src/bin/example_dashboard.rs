use polars::prelude::*;
use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Bar = GroupedBarConfig;
type Line = LineConfig;
type HB = HBarConfig;
type Scat = ScatterConfig;

// ── DataFrame builders ──────────────────────────────────────────────────────

fn build_monthly_revenue() -> DataFrame {
    df![
        "month" => ["Jan","Jan","Feb","Feb","Mar","Mar","Apr","Apr",
                     "May","May","Jun","Jun","Jul","Jul","Aug","Aug",
                     "Sep","Sep","Oct","Oct","Nov","Nov","Dec","Dec"],
        "category" => ["Revenue","Expenses","Revenue","Expenses","Revenue","Expenses",
                        "Revenue","Expenses","Revenue","Expenses","Revenue","Expenses",
                        "Revenue","Expenses","Revenue","Expenses","Revenue","Expenses",
                        "Revenue","Expenses","Revenue","Expenses","Revenue","Expenses"],
        "value" => [120.5,95.0, 135.2,102.5, 148.7,110.3, 162.3,118.7,
                    175.0,125.2, 190.8,132.8, 205.1,140.1, 198.4,136.5,
                    210.7,145.2, 225.3,152.7, 240.6,160.3, 280.9,175.5f64]
    ].expect("monthly_revenue")
}

fn build_quarterly_products() -> DataFrame {
    df![
        "quarter" => ["Q1","Q1","Q1","Q1","Q2","Q2","Q2","Q2",
                       "Q3","Q3","Q3","Q3","Q4","Q4","Q4","Q4"],
        "product" => ["Alpha","Beta","Gamma","Delta",
                       "Alpha","Beta","Gamma","Delta",
                       "Alpha","Beta","Gamma","Delta",
                       "Alpha","Beta","Gamma","Delta"],
        "value" => [320.5,210.0,140.3,95.0, 410.2,275.8,165.0,120.5,
                    390.7,305.3,195.5,145.2, 520.1,380.6,240.9,180.3f64]
    ].expect("quarterly_products")
}

fn build_monthly_trends() -> DataFrame {
    df![
        "month" => ["Jan","Feb","Mar","Apr","May","Jun",
                     "Jul","Aug","Sep","Oct","Nov","Dec"],
        "revenue"  => [120.5,135.2,148.7,162.3,175.0,190.8,205.1,198.4,210.7,225.3,240.6,280.9f64],
        "expenses" => [95.0,102.5,110.3,118.7,125.2,132.8,140.1,136.5,145.2,152.7,160.3,175.5f64],
        "profit"   => [25.5,32.7,38.4,43.6,49.8,58.0,65.0,61.9,65.5,72.6,80.3,105.4f64],
        "margin"   => [21.2,24.2,25.8,26.9,28.5,30.4,31.7,31.2,31.1,32.2,33.4,37.5f64]
    ].expect("monthly_trends")
}

fn build_regional_sales() -> DataFrame {
    df![
        "region" => ["North","North","North","South","South","South",
                      "East","East","East","West","West","West",
                      "Central","Central","Central"],
        "channel" => ["Online","Retail","Wholesale","Online","Retail","Wholesale",
                       "Online","Retail","Wholesale","Online","Retail","Wholesale",
                       "Online","Retail","Wholesale"],
        "value" => [245.0,180.5,120.3, 198.7,210.0,95.5,
                    310.2,165.8,140.0, 175.5,195.3,110.8,
                    220.1,155.6,130.2f64]
    ].expect("regional_sales")
}

fn build_dept_headcount() -> DataFrame {
    df![
        "department" => ["Engineering","Engineering","Engineering",
                          "Marketing","Marketing","Marketing",
                          "Sales","Sales","Sales",
                          "Support","Support","Support",
                          "Finance","Finance","Finance",
                          "Operations","Operations","Operations"],
        "year" => ["2022","2023","2024","2022","2023","2024",
                    "2022","2023","2024","2022","2023","2024",
                    "2022","2023","2024","2022","2023","2024"],
        "count" => [45i64,62,78, 20,25,30, 35,40,48,
                    15,18,22, 10,12,14, 25,28,32]
    ].expect("dept_headcount")
}

fn build_satisfaction() -> DataFrame {
    df![
        "category" => ["Product Quality","Customer Service","Pricing",
                        "Delivery Speed","Documentation","Onboarding",
                        "Mobile App","API Reliability"],
        "score" => [4.5, 4.2, 3.8, 4.0, 3.5, 3.9, 4.3, 4.6f64]
    ].expect("satisfaction")
}

fn build_website_traffic() -> DataFrame {
    df![
        "month" => ["Jan","Feb","Mar","Apr","May","Jun",
                     "Jul","Aug","Sep","Oct","Nov","Dec"],
        "visitors"    => [45000i64,48500,52000,58000,62000,67000,
                          71000,69000,73000,78000,85000,92000],
        "signups"     => [1200i64,1350,1500,1800,2100,2400,2600,2450,2700,3000,3400,3800],
        "conversions" => [320i64,380,420,510,590,680,720,690,750,830,950,1050]
    ].expect("website_traffic")
}

fn build_market_share() -> DataFrame {
    df![
        "company" => ["Our Company","Competitor A","Competitor B",
                       "Competitor C","Competitor D","Others"],
        "share" => [28.5, 22.0, 18.3, 12.7, 8.5, 10.0f64]
    ].expect("market_share")
}

fn build_budget_vs_actual() -> DataFrame {
    df![
        "department" => ["Engineering","Engineering","Marketing","Marketing",
                          "Sales","Sales","Support","Support",
                          "Finance","Finance","Operations","Operations"],
        "type" => ["Budget","Actual","Budget","Actual",
                    "Budget","Actual","Budget","Actual",
                    "Budget","Actual","Budget","Actual"],
        "amount" => [500.0,480.0, 200.0,220.0, 300.0,310.0,
                     150.0,140.0, 100.0,95.0, 250.0,235.0f64]
    ].expect("budget_vs_actual")
}

fn build_scatter_performance() -> DataFrame {
    df![
        "revenue"      => [50.0,75.0,120.0,95.0,200.0,180.0,60.0,140.0,310.0,88.0,
                           155.0,220.0,45.0,170.0,280.0,110.0,190.0,65.0,240.0,135.0,
                           300.0,85.0,160.0,210.0,70.0,250.0,100.0,180.0,130.0,270.0f64],
        "profit"       => [8.0,15.0,30.0,18.0,52.0,42.0,10.0,35.0,85.0,16.0,
                           38.0,55.0,5.0,40.0,72.0,22.0,48.0,12.0,60.0,32.0,
                           78.0,14.0,39.0,53.0,11.0,65.0,20.0,44.0,28.0,70.0f64],
        "employees"    => [5i64,8,15,10,25,22,6,18,40,9,
                           19,28,4,21,35,12,24,7,30,16,
                           38,8,20,26,7,32,11,23,14,34],
        "satisfaction" => [3.8,4.0,4.3,4.1,4.6,4.4,3.9,4.2,4.8,4.0,
                           4.3,4.5,3.7,4.3,4.7,4.1,4.4,3.8,4.5,4.2,
                           4.7,3.9,4.3,4.5,3.8,4.6,4.0,4.4,4.1,4.6f64],
        "tier"         => ["Small","Small","Medium","Small","Large","Medium",
                           "Small","Medium","Large","Small","Medium","Large",
                           "Small","Medium","Large","Medium","Medium","Small",
                           "Large","Medium","Large","Small","Medium","Large",
                           "Small","Large","Small","Medium","Medium","Large"]
    ].expect("scatter_performance")
}

fn build_project_status() -> DataFrame {
    df![
        "project" => ["Auth Rewrite","API v3","Mobile App","Dashboard",
                       "Search Engine","Payment Gateway","CI/CD Pipeline",
                       "Data Lake","Notifications","Analytics"],
        "completion" => [95.0, 78.0, 62.0, 88.0, 45.0, 92.0, 100.0, 55.0, 70.0, 82.0f64]
    ].expect("project_status")
}

fn build_cost_breakdown() -> DataFrame {
    df![
        "category" => ["Salaries","Cloud Infra","Marketing","Office",
                        "Software Licenses","Travel","Training","Legal"],
        "amount" => [850.0, 320.0, 200.0, 150.0, 95.0, 60.0, 45.0, 35.0f64]
    ].expect("cost_breakdown")
}

fn build_quarterly_trends() -> DataFrame {
    df![
        "quarter"  => ["Q1-23","Q2-23","Q3-23","Q4-23","Q1-24","Q2-24","Q3-24","Q4-24"],
        "revenue"  => [680.0,750.0,720.0,810.0,890.0,960.0,940.0,1050.0f64],
        "costs"    => [520.0,560.0,540.0,590.0,630.0,670.0,660.0,710.0f64],
        "margin"   => [23.5,25.3,25.0,27.2,29.2,30.2,29.8,32.4f64]
    ].expect("quarterly_trends")
}

fn build_marketing_channels() -> DataFrame {
    df![
        "quarter" => ["Q1","Q1","Q1","Q1","Q2","Q2","Q2","Q2",
                       "Q3","Q3","Q3","Q3","Q4","Q4","Q4","Q4"],
        "channel" => ["Social","Email","Search","Direct",
                       "Social","Email","Search","Direct",
                       "Social","Email","Search","Direct",
                       "Social","Email","Search","Direct"],
        "spend" => [45.0,30.0,65.0,20.0, 55.0,35.0,75.0,22.0,
                    60.0,38.0,80.0,25.0, 70.0,42.0,90.0,28.0f64]
    ].expect("marketing_channels")
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dash = Dashboard::new();

    // Register all DataFrames
    dash.add_df("monthly_revenue", &mut build_monthly_revenue())?;
    dash.add_df("quarterly_products", &mut build_quarterly_products())?;
    dash.add_df("monthly_trends", &mut build_monthly_trends())?;
    dash.add_df("regional_sales", &mut build_regional_sales())?;
    dash.add_df("dept_headcount", &mut build_dept_headcount())?;
    dash.add_df("satisfaction", &mut build_satisfaction())?;
    dash.add_df("website_traffic", &mut build_website_traffic())?;
    dash.add_df("market_share", &mut build_market_share())?;
    dash.add_df("budget_vs_actual", &mut build_budget_vs_actual())?;
    dash.add_df("scatter_performance", &mut build_scatter_performance())?;
    dash.add_df("project_status", &mut build_project_status())?;
    dash.add_df("cost_breakdown", &mut build_cost_breakdown())?;
    dash.add_df("quarterly_trends", &mut build_quarterly_trends())?;
    dash.add_df("marketing_channels", &mut build_marketing_channels())?;

    // ── Define all pages ────────────────────────────────────────────────────

    // 1. Executive Summary
    dash.add_page(
        PageBuilder::new("executive-summary", "Executive Summary", "Executive", 2)
            .chart(C::line("Revenue & Profit Trends", "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "profit"]).y_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::hbar("Market Position", "market_share",
                HB::builder().category("company").value("share").x_label("Market Share %").build()?
            ).at(1, 0, 1).build())
            .chart(C::bar("Quarterly Products", "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?
            ).at(1, 1, 1).build())
            .chart(C::scatter("Revenue vs Profit", "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?
            ).at(2, 0, 2).filtered().build())
            .filter(FilterSpec::range("scatter_performance", "revenue", "Revenue Range", 40.0, 320.0, 10.0))
            .build(),
    );

    // 2. Revenue Overview
    dash.add_page(
        PageBuilder::new("revenue-overview", "Revenue Overview", "Revenue", 2)
            .chart(C::bar("Monthly Revenue vs Expenses", "monthly_revenue",
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::line("Revenue Trend", "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "expenses"]).y_label("USD (k)").build()?
            ).at(1, 0, 1).build())
            .chart(C::line("Profit Margin", "monthly_trends",
                Line::builder().x("month").y_cols(&["margin"]).y_label("%").build()?
            ).at(1, 1, 1).build())
            .chart(C::bar("Regional Sales", "regional_sales",
                Bar::builder().x("region").group("channel").value("value").y_label("USD (k)").build()?
            ).at(2, 0, 2).build())
            .build(),
    );

    // 3. Expense Analysis
    dash.add_page(
        PageBuilder::new("expense-analysis", "Expense Analysis", "Expenses", 2)
            .chart(C::hbar("Cost Breakdown", "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?
            ).at(0, 0, 1).build())
            .chart(C::bar("Budget vs Actual", "budget_vs_actual",
                Bar::builder().x("department").group("type").value("amount").y_label("USD (k)").build()?
            ).at(0, 1, 1).build())
            .chart(C::line("Expense Trends", "monthly_trends",
                Line::builder().x("month").y_cols(&["expenses"]).y_label("USD (k)").build()?
            ).at(1, 0, 1).build())
            .chart(C::line("Margin Trend", "monthly_trends",
                Line::builder().x("month").y_cols(&["margin"]).y_label("%").build()?
            ).at(1, 1, 1).build())
            .build(),
    );

    // 4. Quarterly Performance
    dash.add_page(
        PageBuilder::new("quarterly-performance", "Quarterly Performance", "Quarterly", 2)
            .chart(C::bar("Product Revenue by Quarter", "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::line("Quarterly Revenue & Costs", "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?
            ).at(1, 0, 1).build())
            .chart(C::line("Quarterly Margin", "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["margin"]).y_label("%").build()?
            ).at(1, 1, 1).build())
            .build(),
    );

    // 5. Product Analysis
    dash.add_page(
        PageBuilder::new("product-analysis", "Product Analysis", "Products", 2)
            .chart(C::bar("Quarterly Product Revenue", "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::scatter("Revenue vs Profit by Team", "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?
            ).at(1, 0, 1).filtered().build())
            .chart(C::scatter("Revenue vs Satisfaction", "scatter_performance",
                Scat::builder().x("revenue").y("satisfaction").x_label("Revenue (k)").y_label("Rating").build()?
            ).at(1, 1, 1).filtered().build())
            .filter(FilterSpec::select("scatter_performance", "tier", "Company Tier", vec!["Small", "Medium", "Large"]))
            .filter(FilterSpec::range("scatter_performance", "revenue", "Revenue Range", 40.0, 320.0, 10.0))
            .build(),
    );

    // 6. Regional Breakdown
    dash.add_page(
        PageBuilder::new("regional-breakdown", "Regional Sales Breakdown", "Regions", 2)
            .chart(C::bar("Sales by Region & Channel", "regional_sales",
                Bar::builder().x("region").group("channel").value("value").y_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::hbar("Market Share", "market_share",
                HB::builder().category("company").value("share").x_label("%").build()?
            ).at(1, 0, 1).build())
            .chart(C::scatter("Employees vs Revenue", "scatter_performance",
                Scat::builder().x("employees").y("revenue").x_label("Team Size").y_label("Revenue (k)").build()?
            ).at(1, 1, 1).build())
            .build(),
    );

    // 7. Team Metrics
    dash.add_page(
        PageBuilder::new("team-metrics", "Team & Workforce Metrics", "Team", 2)
            .chart(C::bar("Department Headcount by Year", "dept_headcount",
                Bar::builder().x("department").group("year").value("count").y_label("Employees").build()?
            ).at(0, 0, 2).build())
            .chart(C::scatter("Employees vs Profit", "scatter_performance",
                Scat::builder().x("employees").y("profit").x_label("Team Size").y_label("Profit (k)").build()?
            ).at(1, 0, 1).filtered().build())
            .chart(C::scatter("Employees vs Satisfaction", "scatter_performance",
                Scat::builder().x("employees").y("satisfaction").x_label("Team Size").y_label("Rating").build()?
            ).at(1, 1, 1).filtered().build())
            .filter(FilterSpec::threshold("scatter_performance", "satisfaction", "High Satisfaction Only (>4.2)", 4.2, true))
            .build(),
    );

    // 8. Customer Insights
    dash.add_page(
        PageBuilder::new("customer-insights", "Customer Insights", "Customers", 2)
            .chart(C::hbar("Satisfaction Scores", "satisfaction",
                HB::builder().category("category").value("score").x_label("Score (1-5)").build()?
            ).at(0, 0, 2).build())
            .chart(C::scatter("Revenue vs Customer Satisfaction", "scatter_performance",
                Scat::builder().x("revenue").y("satisfaction").x_label("Revenue (k)").y_label("Rating").build()?
            ).at(1, 0, 1).filtered().build())
            .chart(C::scatter("Profit vs Satisfaction", "scatter_performance",
                Scat::builder().x("profit").y("satisfaction").x_label("Profit (k)").y_label("Rating").build()?
            ).at(1, 1, 1).filtered().build())
            .filter(FilterSpec::group("scatter_performance", "tier", "Company Tier", vec!["Small", "Medium", "Large"]))
            .build(),
    );

    // 9. Web Analytics
    dash.add_page(
        PageBuilder::new("web-analytics", "Website Analytics", "Web", 2)
            .chart(C::line("Visitor Traffic", "website_traffic",
                Line::builder().x("month").y_cols(&["visitors"]).y_label("Visitors").build()?
            ).at(0, 0, 2).build())
            .chart(C::line("Signups Over Time", "website_traffic",
                Line::builder().x("month").y_cols(&["signups"]).y_label("Signups").build()?
            ).at(1, 0, 1).build())
            .chart(C::line("Conversions Over Time", "website_traffic",
                Line::builder().x("month").y_cols(&["conversions"]).y_label("Conversions").build()?
            ).at(1, 1, 1).build())
            .build(),
    );

    // 10. Market Position
    dash.add_page(
        PageBuilder::new("market-position", "Market Position", "Market", 2)
            .chart(C::hbar("Market Share", "market_share",
                HB::builder().category("company").value("share").x_label("Share %").build()?
            ).at(0, 0, 1).build())
            .chart(C::hbar("Project Completion", "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?
            ).at(0, 1, 1).build())
            .chart(C::line("Revenue vs Costs (Quarterly)", "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?
            ).at(1, 0, 2).build())
            .build(),
    );

    // 11. Budget Management
    dash.add_page(
        PageBuilder::new("budget-management", "Budget Management", "Budget", 2)
            .chart(C::bar("Budget vs Actual Spending", "budget_vs_actual",
                Bar::builder().x("department").group("type").value("amount").y_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::hbar("Cost Categories", "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?
            ).at(1, 0, 1).build())
            .chart(C::line("Revenue Trend", "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "expenses"]).y_label("USD (k)").build()?
            ).at(1, 1, 1).build())
            .build(),
    );

    // 12. Project Portfolio
    dash.add_page(
        PageBuilder::new("project-portfolio", "Project Portfolio", "Projects", 2)
            .chart(C::hbar("Project Completion Status", "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?
            ).at(0, 0, 2).build())
            .chart(C::scatter("Revenue vs Employees", "scatter_performance",
                Scat::builder().x("revenue").y("employees").x_label("Revenue (k)").y_label("Team Size").build()?
            ).at(1, 0, 1).filtered().build())
            .chart(C::scatter("Profit vs Employees", "scatter_performance",
                Scat::builder().x("profit").y("employees").x_label("Profit (k)").y_label("Team Size").build()?
            ).at(1, 1, 1).filtered().build())
            .filter(FilterSpec::top_n("scatter_performance", "revenue", "Top N by Revenue", 30, true))
            .build(),
    );

    // 13. Growth Indicators
    dash.add_page(
        PageBuilder::new("growth-indicators", "Growth Indicators", "Growth", 2)
            .chart(C::line("Revenue & Profit Growth", "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "profit"]).y_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::line("Visitor Growth", "website_traffic",
                Line::builder().x("month").y_cols(&["visitors", "signups"]).y_label("Count").build()?
            ).at(1, 0, 1).build())
            .chart(C::bar("Quarterly Products", "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?
            ).at(1, 1, 1).build())
            .build(),
    );

    // 14. Cost Optimization
    dash.add_page(
        PageBuilder::new("cost-optimization", "Cost Optimization", "Costs", 2)
            .chart(C::hbar("Spending by Category", "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::line("Expense vs Margin Trend", "monthly_trends",
                Line::builder().x("month").y_cols(&["expenses", "margin"]).y_label("Value").build()?
            ).at(1, 0, 1).build())
            .chart(C::scatter("Revenue vs Profit Efficiency", "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?
            ).at(1, 1, 1).filtered().build())
            .filter(FilterSpec::threshold("scatter_performance", "profit", "Profitable Only (>30k)", 30.0, true))
            .build(),
    );

    // 15. Marketing ROI
    dash.add_page(
        PageBuilder::new("marketing-roi", "Marketing ROI", "Marketing", 2)
            .chart(C::bar("Channel Spend by Quarter", "marketing_channels",
                Bar::builder().x("quarter").group("channel").value("spend").y_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::line("Website Conversions", "website_traffic",
                Line::builder().x("month").y_cols(&["signups", "conversions"]).y_label("Count").build()?
            ).at(1, 0, 1).build())
            .chart(C::hbar("Market Share", "market_share",
                HB::builder().category("company").value("share").x_label("%").build()?
            ).at(1, 1, 1).build())
            .build(),
    );

    // 16. Operations Dashboard
    dash.add_page(
        PageBuilder::new("operations-dashboard", "Operations Dashboard", "Operations", 3)
            .chart(C::hbar("Project Status", "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?
            ).at(0, 0, 1).build())
            .chart(C::hbar("Cost Breakdown", "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?
            ).at(0, 1, 1).build())
            .chart(C::hbar("Satisfaction", "satisfaction",
                HB::builder().category("category").value("score").x_label("Score").build()?
            ).at(0, 2, 1).build())
            .chart(C::line("Traffic & Signups", "website_traffic",
                Line::builder().x("month").y_cols(&["visitors", "signups"]).y_label("Count").build()?
            ).at(1, 0, 2).build())
            .chart(C::scatter("Team Efficiency", "scatter_performance",
                Scat::builder().x("employees").y("profit").x_label("Team Size").y_label("Profit (k)").build()?
            ).at(1, 2, 1).build())
            .build(),
    );

    // 17. Financial Health
    dash.add_page(
        PageBuilder::new("financial-health", "Financial Health", "Finance", 2)
            .chart(C::line("Quarterly Revenue, Costs & Margin", "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs", "margin"]).y_label("Value").build()?
            ).at(0, 0, 2).build())
            .chart(C::bar("Monthly Revenue vs Expenses", "monthly_revenue",
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?
            ).at(1, 0, 1).build())
            .chart(C::hbar("Cost Structure", "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?
            ).at(1, 1, 1).build())
            .chart(C::scatter("Profitability Map", "scatter_performance",
                Scat::builder().x("revenue").y("profit").x_label("Revenue (k)").y_label("Profit (k)").build()?
            ).at(2, 0, 2).filtered().build())
            .filter(FilterSpec::select("scatter_performance", "tier", "Company Tier", vec!["Small", "Medium", "Large"]))
            .filter(FilterSpec::range("scatter_performance", "employees", "Team Size Range", 4.0, 40.0, 1.0))
            .build(),
    );

    // 18. Workforce Planning
    dash.add_page(
        PageBuilder::new("workforce-planning", "Workforce Planning", "Workforce", 2)
            .chart(C::bar("Headcount Growth", "dept_headcount",
                Bar::builder().x("department").group("year").value("count").y_label("Employees").build()?
            ).at(0, 0, 2).build())
            .chart(C::scatter("Team Size vs Revenue", "scatter_performance",
                Scat::builder().x("employees").y("revenue").x_label("Employees").y_label("Revenue (k)").build()?
            ).at(1, 0, 1).filtered().build())
            .chart(C::scatter("Team Size vs Satisfaction", "scatter_performance",
                Scat::builder().x("employees").y("satisfaction").x_label("Employees").y_label("Rating").build()?
            ).at(1, 1, 1).filtered().build())
            .chart(C::hbar("Budget by Department", "cost_breakdown",
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?
            ).at(2, 0, 2).build())
            .filter(FilterSpec::top_n("scatter_performance", "revenue", "Top N by Revenue", 30, true))
            .filter(FilterSpec::threshold("scatter_performance", "satisfaction", "High Satisfaction Only (>4.0)", 4.0, true))
            .build(),
    );

    // 19. Forecast & Targets
    dash.add_page(
        PageBuilder::new("forecast-targets", "Forecast & Targets", "Forecast", 2)
            .chart(C::line("Monthly Forecast", "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "expenses", "profit"]).y_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::line("Quarterly Outlook", "quarterly_trends",
                Line::builder().x("quarter").y_cols(&["revenue", "costs"]).y_label("USD (k)").build()?
            ).at(1, 0, 1).build())
            .chart(C::hbar("Target Completion", "project_status",
                HB::builder().category("project").value("completion").x_label("% Complete").build()?
            ).at(1, 1, 1).build())
            .build(),
    );

    // 20. Annual Review
    dash.add_page(
        PageBuilder::new("annual-review", "Annual Review", "Annual", 2)
            .chart(C::bar("Monthly Revenue vs Expenses", "monthly_revenue",
                Bar::builder().x("month").group("category").value("value").y_label("USD (k)").build()?
            ).at(0, 0, 2).build())
            .chart(C::bar("Quarterly Product Performance", "quarterly_products",
                Bar::builder().x("quarter").group("product").value("value").y_label("Revenue (k)").build()?
            ).at(1, 0, 2).build())
            .chart(C::hbar("Market Share", "market_share",
                HB::builder().category("company").value("share").x_label("%").build()?
            ).at(2, 0, 1).build())
            .chart(C::hbar("Satisfaction Scores", "satisfaction",
                HB::builder().category("category").value("score").x_label("Score").build()?
            ).at(2, 1, 1).build())
            .chart(C::line("Full Year Trends", "monthly_trends",
                Line::builder().x("month").y_cols(&["revenue", "expenses", "profit", "margin"]).y_label("Value").build()?
            ).at(3, 0, 2).build())
            .build(),
    );

    dash.render()?;
    Ok(())
}
