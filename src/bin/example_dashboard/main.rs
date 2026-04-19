mod data;
mod handles;
mod pages;

use rust_to_bokeh::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut dash = Dashboard::new()
        .title("Annual Business Dashboard")
        .nav_style(NavStyle::Vertical);

    let h = handles::register(&mut dash)?;

    dash.add_page(pages::page_executive_summary(&h)?);
    dash.add_page(pages::page_revenue_overview(&h)?);
    dash.add_page(pages::page_expense_analysis(&h)?);
    dash.add_page(pages::page_quarterly_performance(&h)?);
    dash.add_page(pages::page_product_analysis(&h)?);
    dash.add_page(pages::page_regional_breakdown(&h)?);
    dash.add_page(pages::page_team_metrics(&h)?);
    dash.add_page(pages::page_customer_insights(&h)?);
    dash.add_page(pages::page_web_analytics(&h)?);
    dash.add_page(pages::page_market_position(&h)?);
    dash.add_page(pages::page_budget_management(&h)?);
    dash.add_page(pages::page_project_portfolio(&h)?);
    dash.add_page(pages::page_growth_indicators(&h)?);
    dash.add_page(pages::page_cost_optimization(&h)?);
    dash.add_page(pages::page_marketing_roi(&h)?);
    dash.add_page(pages::page_operations_dashboard(&h)?);
    dash.add_page(pages::page_financial_health(&h)?);
    dash.add_page(pages::page_workforce_planning(&h)?);
    dash.add_page(pages::page_forecast_targets(&h)?);
    dash.add_page(pages::page_annual_review(&h)?);
    dash.add_page(pages::page_module_showcase(&h)?);
    dash.add_page(pages::page_chart_customization(&h)?);
    dash.add_page(pages::page_time_series_events(&h)?);
    dash.add_page(pages::page_range_tool_demo(&h)?);
    dash.add_page(pages::page_pie_donut_charts(&h)?);
    dash.add_page(pages::page_histogram_demo(&h)?);
    dash.add_page(pages::page_box_plot_demo(&h)?);
    dash.add_page(pages::page_density_demo(&h)?);

    #[cfg(feature = "python")]
    dash.render()?;
    #[cfg(not(feature = "python"))]
    dash.render_native(BokehResources::Inline)?;

    Ok(())
}
