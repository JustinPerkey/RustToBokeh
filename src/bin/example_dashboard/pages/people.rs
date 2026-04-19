use rust_to_bokeh::prelude::*;

use crate::handles::Handles;

type C = ChartSpecBuilder;
type Bar = GroupedBarConfig;
type HB = HBarConfig;
type Scat = ScatterConfig;

pub fn page_team_metrics(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("team-metrics", "Team & Workforce Metrics", "Team", 2)
        .category("People")
        .chart(
            C::bar(
                "Department Headcount by Year",
                &h.dept_headcount,
                Bar::builder().x("department").group("year").value("count").y_label("Employees").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Employees vs Profit",
                &h.scatter_performance,
                Scat::builder().x("employees").y("profit").x_label("Team Size").y_label("Profit (k)").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Employees vs Satisfaction",
                &h.scatter_performance,
                Scat::builder().x("employees").y("satisfaction").x_label("Team Size").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::threshold(
            &h.scatter_performance,
            "satisfaction",
            "High Satisfaction Only (>4.2)",
            4.2,
            true,
        ))
        .build()
}

pub fn page_customer_insights(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("customer-insights", "Customer Insights", "Customers", 2)
        .category("People")
        .chart(
            C::hbar(
                "Satisfaction Scores",
                &h.satisfaction,
                HB::builder().category("category").value("score").x_label("Score (1-5)").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Revenue vs Customer Satisfaction",
                &h.scatter_performance,
                Scat::builder().x("revenue").y("satisfaction").x_label("Revenue (k)").y_label("Rating").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Profit vs Satisfaction",
                &h.scatter_performance,
                Scat::builder().x("profit").y("satisfaction").x_label("Profit (k)").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .filter(FilterSpec::group(
            &h.scatter_performance,
            "tier",
            "Company Tier",
            vec!["Small", "Medium", "Large"],
        ))
        .build()
}

pub fn page_workforce_planning(h: &Handles) -> Result<Page, ChartError> {
    PageBuilder::new("workforce-planning", "Workforce Planning", "Workforce", 2)
        .category("People")
        .chart(
            C::bar(
                "Headcount Growth",
                &h.dept_headcount,
                Bar::builder().x("department").group("year").value("count").y_label("Employees").build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .chart(
            C::scatter(
                "Team Size vs Revenue",
                &h.scatter_performance,
                Scat::builder().x("employees").y("revenue").x_label("Employees").y_label("Revenue (k)").build()?,
            )
            .at(1, 0, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::scatter(
                "Team Size vs Satisfaction",
                &h.scatter_performance,
                Scat::builder().x("employees").y("satisfaction").x_label("Employees").y_label("Rating").build()?,
            )
            .at(1, 1, 1)
            .filtered()
            .build(),
        )
        .chart(
            C::hbar(
                "Budget by Department",
                &h.cost_breakdown,
                HB::builder().category("category").value("amount").x_label("USD (k)").build()?,
            )
            .at(2, 0, 2)
            .build(),
        )
        .filter(FilterSpec::top_n(&h.scatter_performance, "revenue", "Top N by Revenue", 30, true))
        .filter(FilterSpec::threshold(
            &h.scatter_performance,
            "satisfaction",
            "High Satisfaction Only (>4.0)",
            4.0,
            true,
        ))
        .build()
}
