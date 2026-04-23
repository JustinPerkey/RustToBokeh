use rust_to_bokeh::prelude::*;

use crate::data;

pub struct Handles {
    pub monthly_revenue: DfHandle,
    pub quarterly_products: DfHandle,
    pub monthly_trends: DfHandle,
    pub regional_sales: DfHandle,
    pub dept_headcount: DfHandle,
    pub satisfaction: DfHandle,
    pub website_traffic: DfHandle,
    pub market_share: DfHandle,
    pub scatter_performance: DfHandle,
    pub project_status: DfHandle,
    pub cost_breakdown: DfHandle,
    pub quarterly_trends: DfHandle,
    pub sensor_events: DfHandle,
    pub salary_hist: DfHandle,
    pub salary_box: DfHandle,
    pub salary_raw: DfHandle,
    pub density_scores: DfHandle,
}

pub fn register(dash: &mut Dashboard) -> Result<Handles, ChartError> {
    let monthly_revenue = dash.add_df("monthly_revenue", &mut data::build_monthly_revenue())?;
    let quarterly_products = dash.add_df("quarterly_products", &mut data::build_quarterly_products())?;
    let monthly_trends = dash.add_df("monthly_trends", &mut data::build_monthly_trends())?;
    let regional_sales = dash.add_df("regional_sales", &mut data::build_regional_sales())?;
    let dept_headcount = dash.add_df("dept_headcount", &mut data::build_dept_headcount())?;
    let satisfaction = dash.add_df("satisfaction", &mut data::build_satisfaction())?;
    let website_traffic = dash.add_df("website_traffic", &mut data::build_website_traffic())?;
    let market_share = dash.add_df("market_share", &mut data::build_market_share())?;
    let scatter_performance = dash.add_df("scatter_performance", &mut data::build_scatter_performance())?;
    let project_status = dash.add_df("project_status", &mut data::build_project_status())?;
    let cost_breakdown = dash.add_df("cost_breakdown", &mut data::build_cost_breakdown())?;
    let quarterly_trends = dash.add_df("quarterly_trends", &mut data::build_quarterly_trends())?;
    let sensor_events = dash.add_df("sensor_events", &mut data::build_sensor_events())?;

    let salary_raw_df = data::build_salary_distribution();
    let mut salary_hist_df = compute_histogram(&salary_raw_df, "salary", 12)?;
    let salary_hist = dash.add_df("salary_hist", &mut salary_hist_df)?;

    let salary_raw2 = data::build_salary_raw();
    let mut salary_box_df = compute_box_stats(&salary_raw2, "department", "salary_k")?;
    let salary_box = dash.add_df("salary_box", &mut salary_box_df)?;

    let mut salary_outliers_df = compute_box_outliers(&salary_raw2, "department", "salary_k")?;
    dash.add_df("salary_outliers", &mut salary_outliers_df)?;

    let salary_raw = dash.add_df("salary_raw", &mut data::build_salary_raw())?;
    let density_scores = dash.add_df("density_scores", &mut data::build_density_scores())?;

    Ok(Handles {
        monthly_revenue,
        quarterly_products,
        monthly_trends,
        regional_sales,
        dept_headcount,
        satisfaction,
        website_traffic,
        market_share,
        scatter_performance,
        project_status,
        cost_breakdown,
        quarterly_trends,
        sensor_events,
        salary_hist,
        salary_box,
        salary_raw,
        density_scores,
    })
}
