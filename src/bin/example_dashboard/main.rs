mod data;
mod pages;

use rust_to_bokeh::prelude::*;

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
    dash.add_df("sensor_events", &mut data::build_sensor_events())?;
    let salary_raw = data::build_salary_distribution();
    let mut salary_hist = compute_histogram(&salary_raw, "salary", 12)?;
    dash.add_df("salary_hist", &mut salary_hist)?;
    Ok(())
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dash = Dashboard::new()
        .title("Annual Business Dashboard")
        .nav_style(NavStyle::Vertical);

    register_dataframes(&mut dash)?;

    dash.add_page(pages::page_executive_summary()?);
    dash.add_page(pages::page_revenue_overview()?);
    dash.add_page(pages::page_expense_analysis()?);
    dash.add_page(pages::page_quarterly_performance()?);
    dash.add_page(pages::page_product_analysis()?);
    dash.add_page(pages::page_regional_breakdown()?);
    dash.add_page(pages::page_team_metrics()?);
    dash.add_page(pages::page_customer_insights()?);
    dash.add_page(pages::page_web_analytics()?);
    dash.add_page(pages::page_market_position()?);
    dash.add_page(pages::page_budget_management()?);
    dash.add_page(pages::page_project_portfolio()?);
    dash.add_page(pages::page_growth_indicators()?);
    dash.add_page(pages::page_cost_optimization()?);
    dash.add_page(pages::page_marketing_roi()?);
    dash.add_page(pages::page_operations_dashboard()?);
    dash.add_page(pages::page_financial_health()?);
    dash.add_page(pages::page_workforce_planning()?);
    dash.add_page(pages::page_forecast_targets()?);
    dash.add_page(pages::page_annual_review()?);
    dash.add_page(pages::page_module_showcase()?);
    dash.add_page(pages::page_chart_customization()?);
    dash.add_page(pages::page_time_series_events()?);
    dash.add_page(pages::page_range_tool_demo()?);
    dash.add_page(pages::page_pie_donut_charts()?);
    dash.add_page(pages::page_histogram_demo()?);

    dash.render()?;
    Ok(())
}
