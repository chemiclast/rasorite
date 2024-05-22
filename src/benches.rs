use crate::data::DataPoint;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumString;
use thiserror::Error;

#[derive(EnumString)]
pub enum KpiType {
    DailyActiveUsers,
    MonthlyActiveUsers,
    Visits,
    TotalHours,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkApiResponse {
    benchmark_percentile: String,
    kpi_type: String,
    universe_kpi_percentile: Option<u64>,
    data: HashMap<String, DataPoint>,
}

pub struct Benchmark {
    benchmark_percentile: u64,
    universe_kpi_percentile: Option<u64>,
    data: Vec<(DateTime<Utc>, DataPoint)>,
}

#[derive(Debug, Error)]
enum AnalyticsFetchError {
    #[error(transparent)]
    Reqwest {
        #[from]
        source: reqwest::Error,
    },

    #[error("Roblox API returned an invalid response!")]
    InvalidResponse,

    #[error("Failed to fetch .ROBLOSECURITY cookie!")]
    Cookie,
}

pub async fn fetch_benches(
    universe_id: u64,
    kpi_type: KpiType,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<Benchmark, AnalyticsFetchError> {
    let url = format!("https://apis.roblox.com/developer-analytics-aggregations/v2/get-benchmarks?universeId={}&kpiType={}&startTime={}&endTime={}", universe_id, kpi_type.into(), start_date, end_date);
    let BenchmarkApiResponse {
        benchmark_percentile,
        kpi_type: _,
        universe_kpi_percentile,
        data: response_data,
    } = reqwest::Client::default()
        .get(url)
        .header(
            "Cookie",
            rbx_cookie::get().ok_or(AnalyticsFetchError::Cookie)?,
        )
        .send()
        .await
        .map_err(|err| AnalyticsFetchError::Reqwest { source: err })?
        .json::<BenchmarkApiResponse>()
        .await
        .map_err(|err| AnalyticsFetchError::Reqwest { source: err })?;

    let mut data = Vec::new();

    for (date, point) in response_data {
        data.push((
            date.parse()
                .map_err(|_| AnalyticsFetchError::InvalidResponse)?,
            point,
        ))
    }

    Ok(Benchmark {
        benchmark_percentile: benchmark_percentile.matches(char::is_numeric).into(),
        universe_kpi_percentile,
        data,
    })
}
