use crate::data::{get_data_range, DataPoint};
use crate::parse::AnalyticsData;
use chrono::{DateTime, Utc};
use plotters::backend::DrawingBackend;
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::drawing::IntoDrawingArea;
use plotters::series::LineSeries;
use plotters::style::full_palette::{GREY, LIGHTBLUE, ORANGE};
use plotters::style::FontFamily::SansSerif;
use plotters::style::{FontStyle, IntoFont, BLACK, WHITE};
use plotters_svg::SVGBackend;
use std::ops::Mul;

pub fn plot_data(data: AnalyticsData, normalize: bool) {
    let data_series = data
        .data
        .clone()
        .into_iter()
        .find(|(key, _)| !key.starts_with("Benchmark"))
        .expect("Failed to find analytics data series!");
    let bench_series = data
        .data
        .clone()
        .into_iter()
        .find(|(key, _)| key.starts_with("Benchmark"))
        .expect("Failed to find benchmark series!");

    let backend = SVGBackend::new("plot.svg", (1200, 800));
    let mut drawing_area = backend.into_drawing_area();

    drawing_area
        .fill(&WHITE)
        .expect("Failed to fill drawing area!");
    drawing_area = drawing_area
        .titled(
            &*format!("{} for Experience ID {}", data.kpi_type, data.universe_id),
            (SansSerif, 50, FontStyle::Bold).into_font().color(&BLACK),
        )
        .expect("Failed to draw title!");

    drawing_area = if normalize {
        drawing_area.titled(
            &*format!("Normalized over series \"{}\"", bench_series.0),
            (SansSerif, 25f64, FontStyle::Italic)
                .into_font()
                .color(&GREY),
        )
    } else {
        drawing_area.titled(
            &*format!("Plotted with series \"{}\"", bench_series.0),
            (SansSerif, 25f64, FontStyle::Italic)
                .into_font()
                .color(&GREY),
        )
    }
    .expect("Failed to draw subtitle!");

    let mut chart = ChartBuilder::on(&drawing_area);
    chart
        .margin(5)
        .top_x_label_area_size(0)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40);

    let (date_range, data_range) = get_data_range(
        &data
            .data
            .into_values()
            .collect::<Vec<Vec<(DateTime<Utc>, DataPoint)>>>()
            .into_iter()
            .flatten()
            .collect(),
    );

    let mut chart_context = chart
        .build_cartesian_2d(date_range, data_range)
        .expect("Failed to construct chart!");
    chart_context
        .configure_mesh()
        .draw()
        .expect("Failed to draw chart!");

    if normalize {
        chart_context
            .draw_series(
                LineSeries::new(normalize_data(data_series.1, bench_series.1), &ORANGE)
                    .point_size(0),
            )
            .expect("Failed to draw data series!");
    } else {
        chart_context
            .draw_series(LineSeries::new(data_series.1, &LIGHTBLUE).point_size(0))
            .expect("Failed to draw analytics data series!");
        chart_context
            .draw_series(LineSeries::new(bench_series.1, &GREY).point_size(0))
            .expect("Failed to draw benchmark data series!");
    }

    chart.caption(bench_series.0, (SansSerif, 25, FontStyle::Italic, &GREY));

    drawing_area.present().expect("Failed to present plot!");
}

impl Mul<f64> for &DataPoint {
    type Output = f64;

    fn mul(self, rhs: f64) -> Self::Output {
        match self {
            DataPoint::Float(value) => value.to_num::<f64>() * rhs,
            DataPoint::Integer(value) => *value as f64 * rhs,
        }
    }
}

pub fn normalize_data(
    data: Vec<(DateTime<Utc>, DataPoint)>,
    bench: Vec<(DateTime<Utc>, DataPoint)>,
) -> Vec<(DateTime<Utc>, DataPoint)> {
    let mut result = Vec::new();
    let avg = bench
        .iter()
        .map(|(_, point)| <DataPoint as Into<f64>>::into(*point))
        .sum::<f64>()
        / bench.len() as f64;

    for (date, bench_point) in bench {
        let scalar: f64 = avg / <DataPoint as Into<f64>>::into(bench_point);
        let Some((_, data_point)) = data.iter().find(|(date_point, _)| date_point == &date) else {
            continue;
        };
        result.push((date, DataPoint::from(data_point * scalar)));
    }

    result
}
