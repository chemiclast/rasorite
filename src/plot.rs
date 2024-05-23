use crate::data::{get_data_range, DataPoint};
use crate::parse::AnalyticsData;
use crate::Cli;
use chrono::{DateTime, Utc};
use derive_more::Display;
use plotters::backend::{BitMapBackend, DrawingBackend};
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::drawing::IntoDrawingArea;
use plotters::series::LineSeries;
use plotters::style::full_palette::{GREY, LIGHTBLUE, ORANGE};
use plotters::style::FontFamily::SansSerif;
use plotters::style::{Color, FontStyle, IntoFont, BLACK, WHITE};
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingErrorKind,
};
use plotters_svg::SVGBackend;
use std::error::Error;
use std::ops::Mul;

enum DrawingBackendVariant<'a> {
    Vector(SVGBackend<'a>),
    Bitmap(BitMapBackend<'a>),
}

#[derive(Debug, Display)]
enum DrawingBackendError {
    Vector(std::io::Error),
    Bitmap(plotters_bitmap::BitMapBackendError),
}

fn map_vector_err(e: DrawingErrorKind<std::io::Error>) -> DrawingErrorKind<DrawingBackendError> {
    match e {
        DrawingErrorKind::DrawingError(inner) => DrawingErrorKind::DrawingError(inner.into()),
        DrawingErrorKind::FontError(inner) => DrawingErrorKind::FontError(inner),
    }
}

fn map_bitmap_err(
    e: DrawingErrorKind<plotters_bitmap::BitMapBackendError>,
) -> DrawingErrorKind<DrawingBackendError> {
    match e {
        DrawingErrorKind::DrawingError(inner) => DrawingErrorKind::DrawingError(inner.into()),
        DrawingErrorKind::FontError(inner) => DrawingErrorKind::FontError(inner),
    }
}

impl From<std::io::Error> for DrawingBackendError {
    fn from(value: std::io::Error) -> Self {
        DrawingBackendError::Vector(value)
    }
}

impl From<plotters_bitmap::BitMapBackendError> for DrawingBackendError {
    fn from(value: plotters_bitmap::BitMapBackendError) -> Self {
        DrawingBackendError::Bitmap(value)
    }
}

impl Error for DrawingBackendError {}

impl DrawingBackend for DrawingBackendVariant<'_> {
    type ErrorType = DrawingBackendError;

    fn get_size(&self) -> (u32, u32) {
        match self {
            DrawingBackendVariant::Vector(backend) => backend.get_size(),
            DrawingBackendVariant::Bitmap(backend) => backend.get_size(),
        }
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => {
                backend.ensure_prepared().map_err(map_vector_err)
            }
            DrawingBackendVariant::Bitmap(backend) => {
                backend.ensure_prepared().map_err(map_bitmap_err)
            }
        }
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => backend.present().map_err(map_vector_err),
            DrawingBackendVariant::Bitmap(backend) => backend.present().map_err(map_bitmap_err),
        }
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => {
                backend.draw_pixel(point, color).map_err(map_vector_err)
            }
            DrawingBackendVariant::Bitmap(backend) => {
                backend.draw_pixel(point, color).map_err(map_bitmap_err)
            }
        }
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => {
                backend.draw_line(from, to, style).map_err(map_vector_err)
            }
            DrawingBackendVariant::Bitmap(backend) => {
                backend.draw_line(from, to, style).map_err(map_bitmap_err)
            }
        }
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => backend
                .draw_rect(upper_left, bottom_right, style, fill)
                .map_err(map_vector_err),
            DrawingBackendVariant::Bitmap(backend) => backend
                .draw_rect(upper_left, bottom_right, style, fill)
                .map_err(map_bitmap_err),
        }
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => {
                backend.draw_path(path, style).map_err(map_vector_err)
            }
            DrawingBackendVariant::Bitmap(backend) => {
                backend.draw_path(path, style).map_err(map_bitmap_err)
            }
        }
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => backend
                .draw_circle(center, radius, style, fill)
                .map_err(map_vector_err),
            DrawingBackendVariant::Bitmap(backend) => backend
                .draw_circle(center, radius, style, fill)
                .map_err(map_bitmap_err),
        }
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => {
                backend.fill_polygon(vert, style).map_err(map_vector_err)
            }
            DrawingBackendVariant::Bitmap(backend) => {
                backend.fill_polygon(vert, style).map_err(map_bitmap_err)
            }
        }
    }

    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => {
                backend.draw_text(text, style, pos).map_err(map_vector_err)
            }
            DrawingBackendVariant::Bitmap(backend) => {
                backend.draw_text(text, style, pos).map_err(map_bitmap_err)
            }
        }
    }

    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => backend
                .estimate_text_size(text, style)
                .map_err(map_vector_err),
            DrawingBackendVariant::Bitmap(backend) => backend
                .estimate_text_size(text, style)
                .map_err(map_bitmap_err),
        }
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        match self {
            DrawingBackendVariant::Vector(backend) => backend
                .blit_bitmap(pos, (iw, ih), src)
                .map_err(map_vector_err),
            DrawingBackendVariant::Bitmap(backend) => backend
                .blit_bitmap(pos, (iw, ih), src)
                .map_err(map_bitmap_err),
        }
    }
}

impl<'a> From<SVGBackend<'a>> for DrawingBackendVariant<'a> {
    fn from(value: SVGBackend<'a>) -> Self {
        DrawingBackendVariant::Vector(value)
    }
}

impl<'a> From<BitMapBackend<'a>> for DrawingBackendVariant<'a> {
    fn from(value: BitMapBackend<'a>) -> Self {
        DrawingBackendVariant::Bitmap(value)
    }
}

pub fn plot_data(data: AnalyticsData, opts: &Cli) {
    let Cli {
        normalize,
        out_file,
        ..
    } = opts;
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

    let backend = match &out_file.extension().and_then(|value| value.to_str()) {
        Some("svg") => DrawingBackendVariant::Vector(SVGBackend::new(&out_file, (1200, 800))),
        Some(_) => DrawingBackendVariant::Bitmap(BitMapBackend::new(&out_file, (1200, 800))),
        _ => panic!("The provided output file is of an invalid file type!"),
    };
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

    drawing_area = if *normalize {
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
        .margin_right(80)
        .set_label_area_size(LabelAreaPosition::Left, 80)
        .set_label_area_size(LabelAreaPosition::Bottom, 80);

    let normalized_data = if *normalize {
        Some(normalize_data(
            data_series.clone().1,
            bench_series.clone().1,
        ))
    } else {
        None
    };

    let (date_range, data_range) = if let Some(data) = &normalized_data {
        get_data_range(data)
    } else {
        get_data_range(
            &data
                .data
                .into_values()
                .collect::<Vec<Vec<(DateTime<Utc>, DataPoint)>>>()
                .into_iter()
                .flatten()
                .collect(),
        )
    };

    let mut chart_context = chart
        .build_cartesian_2d(date_range, data_range)
        .expect("Failed to construct chart!");
    chart_context
        .configure_mesh()
        .label_style((SansSerif, 18))
        .x_label_formatter(&|x| x.format("%F").to_string())
        .y_label_formatter(&|y| <DataPoint as Into<u64>>::into(*y).to_string())
        .draw()
        .expect("Failed to draw chart!");

    if let Some(data) = normalized_data {
        chart_context
            .draw_series(LineSeries::new(data, Color::stroke_width(&ORANGE, 2)).point_size(0))
            .expect("Failed to draw data series!");
    } else {
        chart_context
            .draw_series(
                LineSeries::new(data_series.1, Color::stroke_width(&LIGHTBLUE, 2)).point_size(0),
            )
            .expect("Failed to draw analytics data series!");
        chart_context
            .draw_series(
                LineSeries::new(bench_series.1, Color::stroke_width(&GREY, 1)).point_size(0),
            )
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
