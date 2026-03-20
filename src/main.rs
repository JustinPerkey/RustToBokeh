use polars::io::ipc::IpcWriter;
use polars::io::SerWriter;
use polars::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};
use std::ffi::CString;
use std::io::Cursor;

/// Configure the vendored Python so PyO3 can find the interpreter, standard
/// library, and installed packages. Must run before any PyO3 call.
fn configure_vendored_python() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()));

    let candidates = [
        exe_dir
            .as_ref()
            .map(|d| d.join("../../vendor/python")),
        exe_dir.as_ref().map(|d| d.join("vendor/python")),
        Some(std::path::PathBuf::from("vendor/python")),
    ];

    for candidate in candidates.iter().flatten() {
        if let Ok(mut canon) = candidate.canonicalize() {
            if cfg!(windows) {
                let s = canon.to_string_lossy().to_string();
                if let Some(stripped) = s.strip_prefix(r"\\?\") {
                    canon = std::path::PathBuf::from(stripped);
                }
            }
            if canon.join("python.exe").exists() || canon.join("bin/python3").exists() {
                std::env::set_var("PYTHONHOME", &canon);

                let site_packages = if cfg!(windows) {
                    canon.join("Lib").join("site-packages")
                } else {
                    let lib = canon.join("lib");
                    std::fs::read_dir(&lib)
                        .ok()
                        .and_then(|mut entries| {
                            entries.find_map(|e| {
                                let name = e.ok()?.file_name().to_string_lossy().to_string();
                                name.starts_with("python3").then(|| lib.join(name).join("site-packages"))
                            })
                        })
                        .unwrap_or_else(|| lib.join("python3").join("site-packages"))
                };
                std::env::set_var("PYTHONPATH", &site_packages);

                let path_var = std::env::var_os("PATH").unwrap_or_default();
                let mut paths = std::env::split_paths(&path_var).collect::<Vec<_>>();
                paths.insert(0, canon);
                if let Ok(new_path) = std::env::join_paths(&paths) {
                    std::env::set_var("PATH", &new_path);
                }
                return;
            }
        }
    }
}

fn serialize_df(df: &mut DataFrame) -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    IpcWriter::new(&mut buf)
        .finish(df)
        .expect("Failed to serialize DataFrame");
    buf.into_inner()
}

// ── Chart types ─────────────────────────────────────────────────────────────

enum ChartType {
    GroupedBar,
    LineMulti,
    HBar,
    ScatterPlot,
}

impl ChartType {
    fn as_str(&self) -> &'static str {
        match self {
            ChartType::GroupedBar => "grouped_bar",
            ChartType::LineMulti => "line_multi",
            ChartType::HBar => "hbar",
            ChartType::ScatterPlot => "scatter",
        }
    }
}

// ── Layout structs ──────────────────────────────────────────────────────────

struct GridCell {
    row: usize,
    col: usize,
    col_span: usize,
}

struct ChartSpec {
    title: String,
    chart_type: ChartType,
    source_key: String,
    config: Vec<(String, String)>,
    grid: GridCell,
    filtered: bool,
}

enum FilterConfig {
    Range { min: f64, max: f64, step: f64 },
    Select { options: Vec<&'static str> },
    Group { options: Vec<&'static str> },
    Threshold { value: f64, above: bool },
    TopN { max_n: usize, descending: bool },
}

struct FilterSpec {
    source_key: String,
    column: String,
    label: String,
    config: FilterConfig,
}

struct Page {
    slug: String,
    title: String,
    nav_label: String,
    grid_cols: usize,
    specs: Vec<ChartSpec>,
    filters: Vec<FilterSpec>,
}

// ── Spec builder helpers (keep page definitions concise) ────────────────────

fn bar(
    title: &str, key: &str, x: &str, group: &str, val: &str, ylabel: &str,
    row: usize, col: usize, span: usize,
) -> ChartSpec {
    ChartSpec {
        title: title.into(),
        chart_type: ChartType::GroupedBar,
        source_key: key.into(),
        config: vec![
            ("x_col".into(), x.into()),
            ("group_col".into(), group.into()),
            ("value_col".into(), val.into()),
            ("y_label".into(), ylabel.into()),
        ],
        grid: GridCell { row, col, col_span: span },
        filtered: false,
    }
}

fn line(
    title: &str, key: &str, x: &str, ycols: &str, ylabel: &str,
    row: usize, col: usize, span: usize,
) -> ChartSpec {
    ChartSpec {
        title: title.into(),
        chart_type: ChartType::LineMulti,
        source_key: key.into(),
        config: vec![
            ("x_col".into(), x.into()),
            ("y_cols".into(), ycols.into()),
            ("y_label".into(), ylabel.into()),
        ],
        grid: GridCell { row, col, col_span: span },
        filtered: false,
    }
}

fn hbar_spec(
    title: &str, key: &str, cat: &str, val: &str, xlabel: &str,
    row: usize, col: usize, span: usize,
) -> ChartSpec {
    ChartSpec {
        title: title.into(),
        chart_type: ChartType::HBar,
        source_key: key.into(),
        config: vec![
            ("category_col".into(), cat.into()),
            ("value_col".into(), val.into()),
            ("x_label".into(), xlabel.into()),
        ],
        grid: GridCell { row, col, col_span: span },
        filtered: false,
    }
}

fn scatter_spec(
    title: &str, key: &str, x: &str, y: &str, xlabel: &str, ylabel: &str,
    row: usize, col: usize, span: usize,
) -> ChartSpec {
    ChartSpec {
        title: title.into(),
        chart_type: ChartType::ScatterPlot,
        source_key: key.into(),
        config: vec![
            ("x_col".into(), x.into()),
            ("y_col".into(), y.into()),
            ("x_label".into(), xlabel.into()),
            ("y_label".into(), ylabel.into()),
        ],
        grid: GridCell { row, col, col_span: span },
        filtered: false,
    }
}

fn scatter_filtered(
    title: &str, key: &str, x: &str, y: &str, xlabel: &str, ylabel: &str,
    row: usize, col: usize, span: usize,
) -> ChartSpec {
    let mut spec = scatter_spec(title, key, x, y, xlabel, ylabel, row, col, span);
    spec.filtered = true;
    spec
}

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
        // Tier based on employee count: Small (<12), Medium (12-24), Large (>=25)
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

fn main() -> PyResult<()> {
    configure_vendored_python();

    // Build and serialize all DataFrames
    let frame_data: Vec<(&str, Vec<u8>)> = vec![
        ("monthly_revenue",      serialize_df(&mut build_monthly_revenue())),
        ("quarterly_products",   serialize_df(&mut build_quarterly_products())),
        ("monthly_trends",       serialize_df(&mut build_monthly_trends())),
        ("regional_sales",       serialize_df(&mut build_regional_sales())),
        ("dept_headcount",       serialize_df(&mut build_dept_headcount())),
        ("satisfaction",         serialize_df(&mut build_satisfaction())),
        ("website_traffic",      serialize_df(&mut build_website_traffic())),
        ("market_share",         serialize_df(&mut build_market_share())),
        ("budget_vs_actual",     serialize_df(&mut build_budget_vs_actual())),
        ("scatter_performance",  serialize_df(&mut build_scatter_performance())),
        ("project_status",       serialize_df(&mut build_project_status())),
        ("cost_breakdown",       serialize_df(&mut build_cost_breakdown())),
        ("quarterly_trends",     serialize_df(&mut build_quarterly_trends())),
        ("marketing_channels",   serialize_df(&mut build_marketing_channels())),
    ];

    // ── Define all pages ────────────────────────────────────────────────────

    let pages: Vec<Page> = vec![
        // 1. Executive Summary — Range filter on revenue
        Page {
            slug: "executive-summary".into(), title: "Executive Summary".into(),
            nav_label: "Executive".into(), grid_cols: 2,
            specs: vec![
                line("Revenue & Profit Trends", "monthly_trends", "month", "revenue,profit", "USD (k)", 0, 0, 2),
                hbar_spec("Market Position", "market_share", "company", "share", "Market Share %", 1, 0, 1),
                bar("Quarterly Products", "quarterly_products", "quarter", "product", "value", "Revenue (k)", 1, 1, 1),
                scatter_filtered("Revenue vs Profit", "scatter_performance", "revenue", "profit", "Revenue (k)", "Profit (k)", 2, 0, 2),
            ],
            filters: vec![
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "revenue".into(),
                    label: "Revenue Range".into(),
                    config: FilterConfig::Range { min: 40.0, max: 320.0, step: 10.0 },
                },
            ],
        },
        // 2. Revenue Overview
        Page {
            slug: "revenue-overview".into(), title: "Revenue Overview".into(),
            nav_label: "Revenue".into(), grid_cols: 2,
            specs: vec![
                bar("Monthly Revenue vs Expenses", "monthly_revenue", "month", "category", "value", "USD (k)", 0, 0, 2),
                line("Revenue Trend", "monthly_trends", "month", "revenue,expenses", "USD (k)", 1, 0, 1),
                line("Profit Margin", "monthly_trends", "month", "margin", "%", 1, 1, 1),
                bar("Regional Sales", "regional_sales", "region", "channel", "value", "USD (k)", 2, 0, 2),
            ],
            filters: vec![],
        },
        // 3. Expense Analysis
        Page {
            slug: "expense-analysis".into(), title: "Expense Analysis".into(),
            nav_label: "Expenses".into(), grid_cols: 2,
            specs: vec![
                hbar_spec("Cost Breakdown", "cost_breakdown", "category", "amount", "USD (k)", 0, 0, 1),
                bar("Budget vs Actual", "budget_vs_actual", "department", "type", "amount", "USD (k)", 0, 1, 1),
                line("Expense Trends", "monthly_trends", "month", "expenses", "USD (k)", 1, 0, 1),
                line("Margin Trend", "monthly_trends", "month", "margin", "%", 1, 1, 1),
            ],
            filters: vec![],
        },
        // 4. Quarterly Performance
        Page {
            slug: "quarterly-performance".into(), title: "Quarterly Performance".into(),
            nav_label: "Quarterly".into(), grid_cols: 2,
            specs: vec![
                bar("Product Revenue by Quarter", "quarterly_products", "quarter", "product", "value", "Revenue (k)", 0, 0, 2),
                line("Quarterly Revenue & Costs", "quarterly_trends", "quarter", "revenue,costs", "USD (k)", 1, 0, 1),
                line("Quarterly Margin", "quarterly_trends", "quarter", "margin", "%", 1, 1, 1),
            ],
            filters: vec![],
        },
        // 5. Product Analysis — GroupFilter (Select) by tier + Range on revenue
        Page {
            slug: "product-analysis".into(), title: "Product Analysis".into(),
            nav_label: "Products".into(), grid_cols: 2,
            specs: vec![
                bar("Quarterly Product Revenue", "quarterly_products", "quarter", "product", "value", "Revenue (k)", 0, 0, 2),
                scatter_filtered("Revenue vs Profit by Team", "scatter_performance", "revenue", "profit", "Revenue (k)", "Profit (k)", 1, 0, 1),
                scatter_filtered("Revenue vs Satisfaction", "scatter_performance", "revenue", "satisfaction", "Revenue (k)", "Rating", 1, 1, 1),
            ],
            filters: vec![
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "tier".into(),
                    label: "Company Tier".into(),
                    config: FilterConfig::Select { options: vec!["Small", "Medium", "Large"] },
                },
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "revenue".into(),
                    label: "Revenue Range".into(),
                    config: FilterConfig::Range { min: 40.0, max: 320.0, step: 10.0 },
                },
            ],
        },
        // 6. Regional Breakdown
        Page {
            slug: "regional-breakdown".into(), title: "Regional Sales Breakdown".into(),
            nav_label: "Regions".into(), grid_cols: 2,
            specs: vec![
                bar("Sales by Region & Channel", "regional_sales", "region", "channel", "value", "USD (k)", 0, 0, 2),
                hbar_spec("Market Share", "market_share", "company", "share", "%", 1, 0, 1),
                scatter_spec("Employees vs Revenue", "scatter_performance", "employees", "revenue", "Team Size", "Revenue (k)", 1, 1, 1),
            ],
            filters: vec![],
        },
        // 7. Team Metrics — BooleanFilter (Threshold) on satisfaction
        Page {
            slug: "team-metrics".into(), title: "Team & Workforce Metrics".into(),
            nav_label: "Team".into(), grid_cols: 2,
            specs: vec![
                bar("Department Headcount by Year", "dept_headcount", "department", "year", "count", "Employees", 0, 0, 2),
                scatter_filtered("Employees vs Profit", "scatter_performance", "employees", "profit", "Team Size", "Profit (k)", 1, 0, 1),
                scatter_filtered("Employees vs Satisfaction", "scatter_performance", "employees", "satisfaction", "Team Size", "Rating", 1, 1, 1),
            ],
            filters: vec![
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "satisfaction".into(),
                    label: "High Satisfaction Only (>4.2)".into(),
                    config: FilterConfig::Threshold { value: 4.2, above: true },
                },
            ],
        },
        // 8. Customer Insights — GroupFilter by tier (always one group selected)
        Page {
            slug: "customer-insights".into(), title: "Customer Insights".into(),
            nav_label: "Customers".into(), grid_cols: 2,
            specs: vec![
                hbar_spec("Satisfaction Scores", "satisfaction", "category", "score", "Score (1-5)", 0, 0, 2),
                scatter_filtered("Revenue vs Customer Satisfaction", "scatter_performance", "revenue", "satisfaction", "Revenue (k)", "Rating", 1, 0, 1),
                scatter_filtered("Profit vs Satisfaction", "scatter_performance", "profit", "satisfaction", "Profit (k)", "Rating", 1, 1, 1),
            ],
            filters: vec![
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "tier".into(),
                    label: "Company Tier".into(),
                    config: FilterConfig::Group { options: vec!["Small", "Medium", "Large"] },
                },
            ],
        },
        // 9. Web Analytics
        Page {
            slug: "web-analytics".into(), title: "Website Analytics".into(),
            nav_label: "Web".into(), grid_cols: 2,
            specs: vec![
                line("Visitor Traffic", "website_traffic", "month", "visitors", "Visitors", 0, 0, 2),
                line("Signups Over Time", "website_traffic", "month", "signups", "Signups", 1, 0, 1),
                line("Conversions Over Time", "website_traffic", "month", "conversions", "Conversions", 1, 1, 1),
            ],
            filters: vec![],
        },
        // 10. Market Position
        Page {
            slug: "market-position".into(), title: "Market Position".into(),
            nav_label: "Market".into(), grid_cols: 2,
            specs: vec![
                hbar_spec("Market Share", "market_share", "company", "share", "Share %", 0, 0, 1),
                hbar_spec("Project Completion", "project_status", "project", "completion", "% Complete", 0, 1, 1),
                line("Revenue vs Costs (Quarterly)", "quarterly_trends", "quarter", "revenue,costs", "USD (k)", 1, 0, 2),
            ],
            filters: vec![],
        },
        // 11. Budget Management
        Page {
            slug: "budget-management".into(), title: "Budget Management".into(),
            nav_label: "Budget".into(), grid_cols: 2,
            specs: vec![
                bar("Budget vs Actual Spending", "budget_vs_actual", "department", "type", "amount", "USD (k)", 0, 0, 2),
                hbar_spec("Cost Categories", "cost_breakdown", "category", "amount", "USD (k)", 1, 0, 1),
                line("Revenue Trend", "monthly_trends", "month", "revenue,expenses", "USD (k)", 1, 1, 1),
            ],
            filters: vec![],
        },
        // 12. Project Portfolio — IndexFilter (TopN) by revenue
        Page {
            slug: "project-portfolio".into(), title: "Project Portfolio".into(),
            nav_label: "Projects".into(), grid_cols: 2,
            specs: vec![
                hbar_spec("Project Completion Status", "project_status", "project", "completion", "% Complete", 0, 0, 2),
                scatter_filtered("Revenue vs Employees", "scatter_performance", "revenue", "employees", "Revenue (k)", "Team Size", 1, 0, 1),
                scatter_filtered("Profit vs Employees", "scatter_performance", "profit", "employees", "Profit (k)", "Team Size", 1, 1, 1),
            ],
            filters: vec![
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "revenue".into(),
                    label: "Top N by Revenue".into(),
                    config: FilterConfig::TopN { max_n: 30, descending: true },
                },
            ],
        },
        // 13. Growth Indicators
        Page {
            slug: "growth-indicators".into(), title: "Growth Indicators".into(),
            nav_label: "Growth".into(), grid_cols: 2,
            specs: vec![
                line("Revenue & Profit Growth", "monthly_trends", "month", "revenue,profit", "USD (k)", 0, 0, 2),
                line("Visitor Growth", "website_traffic", "month", "visitors,signups", "Count", 1, 0, 1),
                bar("Quarterly Products", "quarterly_products", "quarter", "product", "value", "Revenue (k)", 1, 1, 1),
            ],
            filters: vec![],
        },
        // 14. Cost Optimization — Threshold on profit margin
        Page {
            slug: "cost-optimization".into(), title: "Cost Optimization".into(),
            nav_label: "Costs".into(), grid_cols: 2,
            specs: vec![
                hbar_spec("Spending by Category", "cost_breakdown", "category", "amount", "USD (k)", 0, 0, 2),
                line("Expense vs Margin Trend", "monthly_trends", "month", "expenses,margin", "Value", 1, 0, 1),
                scatter_filtered("Revenue vs Profit Efficiency", "scatter_performance", "revenue", "profit", "Revenue (k)", "Profit (k)", 1, 1, 1),
            ],
            filters: vec![
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "profit".into(),
                    label: "Profitable Only (>30k)".into(),
                    config: FilterConfig::Threshold { value: 30.0, above: true },
                },
            ],
        },
        // 15. Marketing ROI
        Page {
            slug: "marketing-roi".into(), title: "Marketing ROI".into(),
            nav_label: "Marketing".into(), grid_cols: 2,
            specs: vec![
                bar("Channel Spend by Quarter", "marketing_channels", "quarter", "channel", "spend", "USD (k)", 0, 0, 2),
                line("Website Conversions", "website_traffic", "month", "signups,conversions", "Count", 1, 0, 1),
                hbar_spec("Market Share", "market_share", "company", "share", "%", 1, 1, 1),
            ],
            filters: vec![],
        },
        // 16. Operations Dashboard
        Page {
            slug: "operations-dashboard".into(), title: "Operations Dashboard".into(),
            nav_label: "Operations".into(), grid_cols: 3,
            specs: vec![
                hbar_spec("Project Status", "project_status", "project", "completion", "% Complete", 0, 0, 1),
                hbar_spec("Cost Breakdown", "cost_breakdown", "category", "amount", "USD (k)", 0, 1, 1),
                hbar_spec("Satisfaction", "satisfaction", "category", "score", "Score", 0, 2, 1),
                line("Traffic & Signups", "website_traffic", "month", "visitors,signups", "Count", 1, 0, 2),
                scatter_spec("Team Efficiency", "scatter_performance", "employees", "profit", "Team Size", "Profit (k)", 1, 2, 1),
            ],
            filters: vec![],
        },
        // 17. Financial Health — combined: GroupFilter + Range
        Page {
            slug: "financial-health".into(), title: "Financial Health".into(),
            nav_label: "Finance".into(), grid_cols: 2,
            specs: vec![
                line("Quarterly Revenue, Costs & Margin", "quarterly_trends", "quarter", "revenue,costs,margin", "Value", 0, 0, 2),
                bar("Monthly Revenue vs Expenses", "monthly_revenue", "month", "category", "value", "USD (k)", 1, 0, 1),
                hbar_spec("Cost Structure", "cost_breakdown", "category", "amount", "USD (k)", 1, 1, 1),
                scatter_filtered("Profitability Map", "scatter_performance", "revenue", "profit", "Revenue (k)", "Profit (k)", 2, 0, 2),
            ],
            filters: vec![
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "tier".into(),
                    label: "Company Tier".into(),
                    config: FilterConfig::Select { options: vec!["Small", "Medium", "Large"] },
                },
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "employees".into(),
                    label: "Team Size Range".into(),
                    config: FilterConfig::Range { min: 4.0, max: 40.0, step: 1.0 },
                },
            ],
        },
        // 18. Workforce Planning — TopN + Threshold combined
        Page {
            slug: "workforce-planning".into(), title: "Workforce Planning".into(),
            nav_label: "Workforce".into(), grid_cols: 2,
            specs: vec![
                bar("Headcount Growth", "dept_headcount", "department", "year", "count", "Employees", 0, 0, 2),
                scatter_filtered("Team Size vs Revenue", "scatter_performance", "employees", "revenue", "Employees", "Revenue (k)", 1, 0, 1),
                scatter_filtered("Team Size vs Satisfaction", "scatter_performance", "employees", "satisfaction", "Employees", "Rating", 1, 1, 1),
                hbar_spec("Budget by Department", "cost_breakdown", "category", "amount", "USD (k)", 2, 0, 2),
            ],
            filters: vec![
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "revenue".into(),
                    label: "Top N by Revenue".into(),
                    config: FilterConfig::TopN { max_n: 30, descending: true },
                },
                FilterSpec {
                    source_key: "scatter_performance".into(), column: "satisfaction".into(),
                    label: "High Satisfaction Only (>4.0)".into(),
                    config: FilterConfig::Threshold { value: 4.0, above: true },
                },
            ],
        },
        // 19. Forecast & Targets
        Page {
            slug: "forecast-targets".into(), title: "Forecast & Targets".into(),
            nav_label: "Forecast".into(), grid_cols: 2,
            specs: vec![
                line("Monthly Forecast", "monthly_trends", "month", "revenue,expenses,profit", "USD (k)", 0, 0, 2),
                line("Quarterly Outlook", "quarterly_trends", "quarter", "revenue,costs", "USD (k)", 1, 0, 1),
                hbar_spec("Target Completion", "project_status", "project", "completion", "% Complete", 1, 1, 1),
            ],
            filters: vec![],
        },
        // 20. Annual Review
        Page {
            slug: "annual-review".into(), title: "Annual Review".into(),
            nav_label: "Annual".into(), grid_cols: 2,
            specs: vec![
                bar("Monthly Revenue vs Expenses", "monthly_revenue", "month", "category", "value", "USD (k)", 0, 0, 2),
                bar("Quarterly Product Performance", "quarterly_products", "quarter", "product", "value", "Revenue (k)", 1, 0, 2),
                hbar_spec("Market Share", "market_share", "company", "share", "%", 2, 0, 1),
                hbar_spec("Satisfaction Scores", "satisfaction", "category", "score", "Score", 2, 1, 1),
                line("Full Year Trends", "monthly_trends", "month", "revenue,expenses,profit,margin", "Value", 3, 0, 2),
            ],
            filters: vec![],
        },
    ];

    // ── PyO3 bridge ─────────────────────────────────────────────────────────

    let python_script = include_str!("../python/render.py");
    let html_template = include_str!("../templates/chart.html");

    Python::with_gil(|py| {
        // Frames dict: source_key -> Arrow IPC bytes
        let py_frames = PyDict::new(py);
        for (key, bytes) in &frame_data {
            py_frames.set_item(*key, PyBytes::new(py, bytes))?;
        }

        // Nav links for all pages
        let py_nav = PyList::empty(py);
        for page in &pages {
            let d = PyDict::new(py);
            d.set_item("slug", &page.slug)?;
            d.set_item("label", &page.nav_label)?;
            py_nav.append(d)?;
        }

        // Pages with nested specs
        let py_pages = PyList::empty(py);
        for page in &pages {
            let p = PyDict::new(py);
            p.set_item("slug", &page.slug)?;
            p.set_item("title", &page.title)?;
            p.set_item("grid_cols", page.grid_cols)?;

            let py_specs = PyList::empty(py);
            for spec in &page.specs {
                let s = PyDict::new(py);
                s.set_item("title", &spec.title)?;
                s.set_item("chart_type", spec.chart_type.as_str())?;
                s.set_item("source_key", &spec.source_key)?;
                s.set_item("grid_row", spec.grid.row)?;
                s.set_item("grid_col", spec.grid.col)?;
                s.set_item("grid_col_span", spec.grid.col_span)?;
                s.set_item("filtered", spec.filtered)?;
                for (k, v) in &spec.config {
                    s.set_item(k.as_str(), v.as_str())?;
                }
                py_specs.append(s)?;
            }
            p.set_item("specs", py_specs)?;

            let py_filters = PyList::empty(py);
            for filter in &page.filters {
                let f = PyDict::new(py);
                f.set_item("source_key", &filter.source_key)?;
                f.set_item("column", &filter.column)?;
                f.set_item("label", &filter.label)?;
                match &filter.config {
                    FilterConfig::Range { min, max, step } => {
                        f.set_item("kind", "range")?;
                        f.set_item("min", *min)?;
                        f.set_item("max", *max)?;
                        f.set_item("step", *step)?;
                    }
                    FilterConfig::Select { options } => {
                        f.set_item("kind", "select")?;
                        let py_opts = PyList::new(py, options)?;
                        f.set_item("options", py_opts)?;
                    }
                    FilterConfig::Group { options } => {
                        f.set_item("kind", "group")?;
                        let py_opts = PyList::new(py, options)?;
                        f.set_item("options", py_opts)?;
                    }
                    FilterConfig::Threshold { value, above } => {
                        f.set_item("kind", "threshold")?;
                        f.set_item("value", *value)?;
                        f.set_item("above", *above)?;
                    }
                    FilterConfig::TopN { max_n, descending } => {
                        f.set_item("kind", "top_n")?;
                        f.set_item("max_n", *max_n)?;
                        f.set_item("descending", *descending)?;
                    }
                }
                py_filters.append(f)?;
            }
            p.set_item("filters", py_filters)?;
            py_pages.append(p)?;
        }

        let locals = PyDict::new(py);
        locals.set_item("frames", py_frames)?;
        locals.set_item("pages", py_pages)?;
        locals.set_item("nav_links", py_nav)?;
        locals.set_item("html_template", html_template)?;
        locals.set_item("output_dir", "output")?;

        let code = CString::new(python_script).expect("Python script contains null byte");
        py.run(code.as_c_str(), Some(&locals), Some(&locals))?;

        println!("Dashboard generated: {} pages in output/", pages.len());
        Ok(())
    })
}
