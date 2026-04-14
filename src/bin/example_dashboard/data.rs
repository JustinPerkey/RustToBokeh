use polars::prelude::*;

pub fn build_monthly_revenue() -> DataFrame {
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
    ]
    .expect("monthly_revenue")
}

pub fn build_quarterly_products() -> DataFrame {
    df![
        "quarter" => ["Q1","Q1","Q1","Q1","Q2","Q2","Q2","Q2",
                       "Q3","Q3","Q3","Q3","Q4","Q4","Q4","Q4"],
        "product" => ["Alpha","Beta","Gamma","Delta",
                       "Alpha","Beta","Gamma","Delta",
                       "Alpha","Beta","Gamma","Delta",
                       "Alpha","Beta","Gamma","Delta"],
        "value" => [320.5,210.0,140.3,95.0, 410.2,275.8,165.0,120.5,
                    390.7,305.3,195.5,145.2, 520.1,380.6,240.9,180.3f64]
    ]
    .expect("quarterly_products")
}

pub fn build_monthly_trends() -> DataFrame {
    df![
        "month" => ["Jan","Feb","Mar","Apr","May","Jun",
                     "Jul","Aug","Sep","Oct","Nov","Dec"],
        "revenue"  => [120.5,135.2,148.7,162.3,175.0,190.8,205.1,198.4,210.7,225.3,240.6,280.9f64],
        "expenses" => [95.0,102.5,110.3,118.7,125.2,132.8,140.1,136.5,145.2,152.7,160.3,175.5f64],
        "profit"   => [25.5,32.7,38.4,43.6,49.8,58.0,65.0,61.9,65.5,72.6,80.3,105.4f64],
        "margin"   => [21.2,24.2,25.8,26.9,28.5,30.4,31.7,31.2,31.1,32.2,33.4,37.5f64]
    ]
    .expect("monthly_trends")
}

pub fn build_regional_sales() -> DataFrame {
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
    ]
    .expect("regional_sales")
}

pub fn build_dept_headcount() -> DataFrame {
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
    ]
    .expect("dept_headcount")
}

pub fn build_satisfaction() -> DataFrame {
    df![
        "category" => ["Product Quality","Customer Service","Pricing",
                        "Delivery Speed","Documentation","Onboarding",
                        "Mobile App","API Reliability"],
        "score" => [4.5, 4.2, 3.8, 4.0, 3.5, 3.9, 4.3, 4.6f64]
    ]
    .expect("satisfaction")
}

pub fn build_website_traffic() -> DataFrame {
    df![
        "month" => ["Jan","Feb","Mar","Apr","May","Jun",
                     "Jul","Aug","Sep","Oct","Nov","Dec"],
        "visitors"    => [45000i64,48500,52000,58000,62000,67000,
                          71000,69000,73000,78000,85000,92000],
        "signups"     => [1200i64,1350,1500,1800,2100,2400,2600,2450,2700,3000,3400,3800],
        "conversions" => [320i64,380,420,510,590,680,720,690,750,830,950,1050]
    ]
    .expect("website_traffic")
}

pub fn build_market_share() -> DataFrame {
    df![
        "company" => ["Our Company","Competitor A","Competitor B",
                       "Competitor C","Competitor D","Others"],
        "share" => [28.5, 22.0, 18.3, 12.7, 8.5, 10.0f64]
    ]
    .expect("market_share")
}

pub fn build_budget_vs_actual() -> DataFrame {
    df![
        "department" => ["Engineering","Engineering","Marketing","Marketing",
                          "Sales","Sales","Support","Support",
                          "Finance","Finance","Operations","Operations"],
        "type" => ["Budget","Actual","Budget","Actual",
                    "Budget","Actual","Budget","Actual",
                    "Budget","Actual","Budget","Actual"],
        "amount" => [500.0,480.0, 200.0,220.0, 300.0,310.0,
                     150.0,140.0, 100.0,95.0, 250.0,235.0f64]
    ]
    .expect("budget_vs_actual")
}

pub fn build_scatter_performance() -> DataFrame {
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
    ]
    .expect("scatter_performance")
}

pub fn build_project_status() -> DataFrame {
    df![
        "project" => ["Auth Rewrite","API v3","Mobile App","Dashboard",
                       "Search Engine","Payment Gateway","CI/CD Pipeline",
                       "Data Lake","Notifications","Analytics"],
        "completion" => [95.0, 78.0, 62.0, 88.0, 45.0, 92.0, 100.0, 55.0, 70.0, 82.0f64]
    ]
    .expect("project_status")
}

pub fn build_cost_breakdown() -> DataFrame {
    df![
        "category" => ["Salaries","Cloud Infra","Marketing","Office",
                        "Software Licenses","Travel","Training","Legal"],
        "amount" => [850.0, 320.0, 200.0, 150.0, 95.0, 60.0, 45.0, 35.0f64]
    ]
    .expect("cost_breakdown")
}

pub fn build_quarterly_trends() -> DataFrame {
    df![
        "quarter"  => ["Q1-23","Q2-23","Q3-23","Q4-23","Q1-24","Q2-24","Q3-24","Q4-24"],
        "revenue"  => [680.0,750.0,720.0,810.0,890.0,960.0,940.0,1050.0f64],
        "costs"    => [520.0,560.0,540.0,590.0,630.0,670.0,660.0,710.0f64],
        "margin"   => [23.5,25.3,25.0,27.2,29.2,30.2,29.8,32.4f64]
    ]
    .expect("quarterly_trends")
}

pub fn build_marketing_channels() -> DataFrame {
    df![
        "quarter" => ["Q1","Q1","Q1","Q1","Q2","Q2","Q2","Q2",
                       "Q3","Q3","Q3","Q3","Q4","Q4","Q4","Q4"],
        "channel" => ["Social","Email","Search","Direct",
                       "Social","Email","Search","Direct",
                       "Social","Email","Search","Direct",
                       "Social","Email","Search","Direct"],
        "spend" => [45.0,30.0,65.0,20.0, 55.0,35.0,75.0,22.0,
                    60.0,38.0,80.0,25.0, 70.0,42.0,90.0,28.0f64]
    ]
    .expect("marketing_channels")
}

/// 30 days of sensor readings (Jan 1–30 2024), cycling through three sensors.
///
/// The `timestamp_ms` column holds milliseconds since the Unix epoch so that
/// `FilterConfig::DateRange` and datetime axes work correctly.
pub fn build_sensor_events() -> DataFrame {
    // Jan 1 2024 00:00:00 UTC
    const EPOCH_START: i64 = 1_704_067_200_000;
    const ONE_DAY_MS: i64 = 86_400_000;
    const SENSORS: [&str; 3] = ["Alpha", "Beta", "Gamma"];

    let n = 30usize;
    let mut timestamps: Vec<i64> = Vec::with_capacity(n);
    let mut sensors: Vec<&str> = Vec::with_capacity(n);
    let mut temperatures: Vec<f64> = Vec::with_capacity(n);
    let mut humidities: Vec<f64> = Vec::with_capacity(n);
    let mut pressures: Vec<f64> = Vec::with_capacity(n);

    // Synthetic but realistic-looking readings
    let temp_base = [22.1, 21.5, 23.0, 20.8, 24.2, 22.7, 21.9, 23.5, 25.1, 22.3,
                     20.5, 23.8, 24.6, 21.2, 22.9, 23.1, 20.9, 25.3, 22.6, 21.7,
                     24.0, 23.3, 21.0, 22.4, 25.5, 20.6, 23.7, 22.0, 24.8, 21.4f64];
    let hum_base =  [55.0, 58.2, 52.4, 61.0, 48.5, 57.3, 63.1, 50.8, 45.2, 59.7,
                     64.0, 51.5, 47.8, 60.3, 54.9, 56.1, 62.4, 44.7, 58.8, 61.5,
                     49.3, 53.6, 65.0, 57.9, 43.2, 63.8, 52.1, 59.4, 46.5, 60.7f64];
    let pres_base = [1013.0,1015.2,1012.5,1016.8,1011.0,1014.3,1017.5,1010.8,1013.9,1015.6,
                     1012.1,1016.2,1011.7,1014.8,1013.4,1015.0,1012.8,1017.1,1013.6,1014.5,
                     1011.3,1016.5,1012.2,1015.8,1010.5,1017.9,1013.1,1014.2,1012.6,1015.4f64];

    for i in 0..n {
        timestamps.push(EPOCH_START + i as i64 * ONE_DAY_MS);
        sensors.push(SENSORS[i % 3]);
        temperatures.push(temp_base[i]);
        humidities.push(hum_base[i]);
        pressures.push(pres_base[i]);
    }

    df![
        "timestamp_ms" => timestamps,
        "sensor"       => sensors,
        "temperature"  => temperatures,
        "humidity"     => humidities,
        "pressure"     => pressures
    ]
    .expect("sensor_events")
}

/// Raw salary data (department, salary_k) for box plot statistics.
///
/// 10 salary data points per department across 6 departments — enough to
/// produce meaningful quartiles and visible IQR differences between groups.
pub fn build_salary_raw() -> DataFrame {
    df![
        "department" => [
            "Engineering","Engineering","Engineering","Engineering","Engineering",
            "Engineering","Engineering","Engineering","Engineering","Engineering",
            "Sales","Sales","Sales","Sales","Sales",
            "Sales","Sales","Sales","Sales","Sales",
            "Marketing","Marketing","Marketing","Marketing","Marketing",
            "Marketing","Marketing","Marketing","Marketing","Marketing",
            "Finance","Finance","Finance","Finance","Finance",
            "Finance","Finance","Finance","Finance","Finance",
            "HR","HR","HR","HR","HR",
            "HR","HR","HR","HR","HR",
            "Operations","Operations","Operations","Operations","Operations",
            "Operations","Operations","Operations","Operations","Operations",
            // Outliers: one per department to demonstrate scatter rendering
            "Engineering", "Sales", "Operations",
        ],
        "salary_k" => [
            // Engineering: 72–128k, median ~98k
            72.0_f64, 80.0, 88.0, 92.0, 95.0,
            100.0, 105.0, 112.0, 120.0, 128.0,
            // Sales: 55–105k, median ~76k
            55.0, 62.0, 68.0, 72.0, 75.0,
            78.0, 83.0, 90.0, 98.0, 105.0,
            // Marketing: 60–100k, median ~80k
            60.0, 65.0, 70.0, 75.0, 78.0,
            82.0, 86.0, 90.0, 95.0, 100.0,
            // Finance: 70–118k, median ~92k
            70.0, 75.0, 82.0, 87.0, 90.0,
            95.0, 100.0, 106.0, 112.0, 118.0,
            // HR: 50–82k, median ~64k
            50.0, 54.0, 58.0, 61.0, 63.0,
            65.0, 68.0, 72.0, 77.0, 82.0,
            // Operations: 48–78k, median ~60k
            48.0, 52.0, 56.0, 58.0, 60.0,
            62.0, 65.0, 68.0, 72.0, 78.0,
            // Outlier values: above/below the 1.5×IQR fences
            165.0, 28.0, 98.0,
        ]
    ]
    .expect("salary_raw")
}

pub fn build_salary_distribution() -> DataFrame {
    df![
        "salary" => [
            42.0_f64, 45.0, 48.0, 51.0, 53.0, 55.0, 57.0, 58.0, 60.0, 61.0,
            62.0, 63.0, 64.0, 65.0, 65.0, 66.0, 67.0, 68.0, 68.0, 69.0,
            70.0, 70.0, 71.0, 72.0, 73.0, 74.0, 75.0, 76.0, 77.0, 78.0,
            80.0, 82.0, 84.0, 87.0, 90.0, 93.0, 97.0, 102.0, 110.0, 125.0,
        ]
    ]
    .expect("salary_distribution")
}

/// Dense performance-score dataset for the violin density demo.
///
/// 50 observations per department (5 departments = 250 rows total), with
/// deliberately varied distribution shapes to produce interesting violins:
/// - Engineering: near-normal, wide spread (50–100)
/// - Sales:       right-skewed, many lower scores (40–95)
/// - Marketing:   bimodal (two clusters at ~55 and ~80)
/// - Finance:     tight, high-performing (70–95)
/// - HR:          near-uniform spread (45–90)
pub fn build_density_scores() -> DataFrame {
    let departments: &[(&str, &[f64])] = &[
        ("Engineering", &engineering_scores()),
        ("Sales", &sales_scores()),
        ("Marketing", &marketing_scores()),
        ("Finance", &finance_scores()),
        ("HR", &hr_scores()),
    ];

    let mut dept_col: Vec<&str> = Vec::new();
    let mut score_col: Vec<f64> = Vec::new();
    for (name, scores) in departments {
        dept_col.extend(std::iter::repeat(*name).take(scores.len()));
        score_col.extend_from_slice(scores);
    }

    df!["dept" => dept_col, "score" => score_col].expect("density_scores")
}

/// Engineering: near-normal around 75, std ~12 (51 pts).
fn engineering_scores() -> [f64; 51] {
    [
        52.0, 55.0, 57.0, 59.0, 61.0, 63.0, 64.0, 65.0, 66.0, 67.0,
        68.0, 69.0, 70.0, 71.0, 72.0, 72.0, 73.0, 74.0, 74.0, 75.0,
        75.0, 76.0, 76.0, 77.0, 77.0, 78.0, 78.0, 79.0, 80.0, 81.0,
        82.0, 83.0, 84.0, 85.0, 85.0, 86.0, 87.0, 88.0, 89.0, 90.0,
        91.0, 91.0, 92.0, 93.0, 94.0, 95.0, 96.0, 97.0, 98.0, 100.0,
        76.0,
    ]
}

/// Sales: right-skewed (mode ~50, long tail to 95) — 51 pts.
fn sales_scores() -> [f64; 51] {
    [
        40.0, 42.0, 44.0, 45.0, 46.0, 47.0, 48.0, 48.0, 49.0, 50.0,
        50.0, 51.0, 51.0, 52.0, 52.0, 53.0, 53.0, 54.0, 55.0, 55.0,
        56.0, 57.0, 58.0, 59.0, 60.0, 61.0, 62.0, 63.0, 64.0, 65.0,
        66.0, 67.0, 68.0, 70.0, 71.0, 73.0, 74.0, 76.0, 78.0, 80.0,
        82.0, 83.0, 85.0, 86.0, 87.0, 89.0, 90.0, 92.0, 93.0, 95.0,
        54.0,
    ]
}

/// Marketing: bimodal — clusters at ~55 and ~80, valley 68–72 (51 pts).
fn marketing_scores() -> [f64; 51] {
    [
        48.0, 50.0, 51.0, 52.0, 53.0, 54.0, 54.0, 55.0, 55.0, 56.0,
        56.0, 57.0, 57.0, 58.0, 58.0, 59.0, 60.0, 61.0, 62.0, 63.0,
        64.0, 65.0, 66.0, 67.0, 68.0,
        72.0, 73.0, 74.0, 75.0, 76.0, 77.0, 77.0, 78.0, 78.0, 79.0,
        79.0, 80.0, 80.0, 81.0, 81.0, 82.0, 82.0, 83.0, 84.0, 85.0,
        86.0, 87.0, 88.0, 89.0, 90.0,
        79.0,
    ]
}

/// Finance: tight high cluster 70–95 (51 pts).
fn finance_scores() -> [f64; 51] {
    [
        70.0, 71.0, 72.0, 73.0, 74.0, 74.0, 75.0, 75.0, 76.0, 76.0,
        77.0, 77.0, 78.0, 78.0, 79.0, 79.0, 80.0, 80.0, 81.0, 81.0,
        82.0, 82.0, 82.0, 83.0, 83.0, 83.0, 84.0, 84.0, 85.0, 85.0,
        85.0, 86.0, 86.0, 87.0, 87.0, 88.0, 88.0, 89.0, 89.0, 90.0,
        90.0, 90.0, 91.0, 91.0, 92.0, 92.0, 93.0, 93.0, 94.0, 95.0,
        83.0,
    ]
}

/// HR: near-uniform spread 45–90 (51 pts).
fn hr_scores() -> [f64; 51] {
    [
        45.0, 46.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0,
        56.0, 57.0, 58.0, 59.0, 60.0, 61.0, 62.0, 63.0, 64.0, 65.0,
        66.0, 67.0, 68.0, 69.0, 70.0, 71.0, 72.0, 73.0, 74.0, 75.0,
        76.0, 77.0, 78.0, 79.0, 80.0, 81.0, 82.0, 83.0, 84.0, 85.0,
        86.0, 87.0, 87.0, 88.0, 88.0, 89.0, 89.0, 90.0, 90.0, 90.0,
        67.0,
    ]
}
