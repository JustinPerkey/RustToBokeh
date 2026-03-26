mod commercial;
mod digital;
mod executive;
mod financial;
mod operations;
mod people;
mod reference;

pub use commercial::{page_market_position, page_product_analysis, page_regional_breakdown};
pub use digital::{page_growth_indicators, page_marketing_roi, page_web_analytics};
pub use executive::page_executive_summary;
pub use financial::{
    page_annual_review, page_budget_management, page_expense_analysis, page_financial_health,
    page_quarterly_performance, page_revenue_overview,
};
pub use operations::{
    page_cost_optimization, page_forecast_targets, page_operations_dashboard,
    page_project_portfolio,
};
pub use people::{page_customer_insights, page_team_metrics, page_workforce_planning};
pub use reference::{
    page_chart_customisation, page_module_showcase, page_range_tool_demo, page_time_series_events,
};
