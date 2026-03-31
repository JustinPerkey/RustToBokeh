use rust_to_bokeh::prelude::*;

type C = ChartSpecBuilder;
type Hist = HistogramConfig;
type BP = BoxPlotConfig;
type Para = ParagraphSpec;
type Pie = PieConfig;

pub fn page_pie_donut_charts() -> Result<Page, ChartError> {
    PageBuilder::new("pie-donut-charts", "Pie & Donut Charts", "Pie & Donut", 2)
        .category("Reference")
        .chart(
            C::pie(
                "Market Share",
                "market_share",
                Pie::builder().label("company").value("share").build()?,
            )
            .at(0, 0, 1)
            .dimensions(380, 380)
            .build(),
        )
        .chart(
            C::pie(
                "Cost Breakdown",
                "cost_breakdown",
                Pie::builder()
                    .label("category")
                    .value("amount")
                    .inner_radius(0.45)
                    .build()?,
            )
            .at(0, 1, 1)
            .dimensions(380, 380)
            .build(),
        )
        .build()
}

pub fn page_histogram_demo() -> Result<Page, ChartError> {
    PageBuilder::new("histogram-demo", "Histogram Demo", "Histogram", 2)
        .category("Reference")
        .chart(
            C::histogram(
                "Salary Distribution — Count",
                "salary_hist",
                Hist::builder()
                    .x_label("Salary (k)")
                    .build()?,
            )
            .at(0, 0, 1)
            .build(),
        )
        .chart(
            C::histogram(
                "Salary Distribution — Density (PDF)",
                "salary_hist",
                Hist::builder()
                    .x_label("Salary (k)")
                    .display(HistogramDisplay::Pdf)
                    .color("#2ecc71")
                    .build()?,
            )
            .at(0, 1, 1)
            .build(),
        )
        .chart(
            C::histogram(
                "Salary Distribution — Cumulative (CDF)",
                "salary_hist",
                Hist::builder()
                    .x_label("Salary (k)")
                    .display(HistogramDisplay::Cdf)
                    .color("#e74c3c")
                    .build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        .build()
}

pub fn page_box_plot_demo() -> Result<Page, ChartError> {
    PageBuilder::new("box-plot-demo", "Box Plot Demo", "Box Plot", 2)
        .category("Reference")
        .chart(
            C::box_plot(
                "Salary Distribution by Department",
                "salary_box",
                BP::builder()
                    .category("category")
                    .q1("q1")
                    .q2("q2")
                    .q3("q3")
                    .lower("lower")
                    .upper("upper")
                    .y_label("Salary (k USD)")
                    .palette(PaletteSpec::Named("Set2".into()))
                    .outlier_source("salary_outliers")
                    .outlier_value_col("salary_k")
                    .build()?,
            )
            .at(0, 0, 2)
            .build(),
        )
        .build()
}

pub fn page_density_demo() -> Result<Page, ChartError> {
    PageBuilder::new("density-demo", "Density Plots", "Density", 2)
        .category("Reference")
        .paragraph(
            Para::new(
                "Density plots reveal the full shape of a distribution across categories. \
                 This page demonstrates the automatic sina/violin selection:\n\n\
                 The top chart uses the raw salary dataset (≈10 data points per department). \
                 Because each category is sparsely populated the renderer chooses a \
                 sina plot — each observation is drawn as a scatter marker jittered \
                 uniformly within the local KDE density envelope, so points fill the \
                 interior of the distribution and every individual observation is visible.\n\n\
                 The bottom chart uses a denser performance-score dataset (51 observations \
                 per department). The higher point count triggers the violin variant — a \
                 mirrored KDE polygon is drawn for each category with a median line \
                 overlaid, giving a smooth picture of the overall distribution shape. \
                 The threshold between modes defaults to 50 points per category and is \
                 configurable via DensityConfig::point_threshold().",
            )
            .title("Sina vs Violin — Automatic Mode Selection")
            .at(0, 0, 2)
            .build(),
        )
        // Sina mode: salary_raw has ~10 pts per department → below default threshold of 50
        .chart(
            C::density(
                "Salary by Department",
                "salary_raw",
                DensityConfig::builder()
                    .category("department")
                    .value("salary_k")
                    .y_label("Salary (k USD)")
                    .palette(PaletteSpec::Named("Set2".into()))
                    .build()?,
            )
            .at(1, 0, 2)
            .build(),
        )
        // Violin mode: density_scores has 51 pts per department → above default threshold of 50
        .chart(
            C::density(
                "Performance Score by Department",
                "density_scores",
                DensityConfig::builder()
                    .category("dept")
                    .value("score")
                    .y_label("Performance Score")
                    .palette(PaletteSpec::Named("Category10".into()))
                    .alpha(0.7)
                    .build()?,
            )
            .at(2, 0, 2)
            .build(),
        )
        .build()
}
