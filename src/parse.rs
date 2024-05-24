use crate::data::DataPoint;
use crate::data::KpiType;
use chrono::{DateTime, NaiveDateTime, Utc};
use csv::{StringRecord, StringRecordsIntoIter};
use log::info;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug)]
pub struct AnalyticsData {
    pub kpi_type: KpiType,
    pub universe_id: u64,
    pub data: HashMap<String, Vec<(DateTime<Utc>, DataPoint)>>,
}

#[derive(Debug, Error)]
pub enum AnalyticsParseError {
    #[error("The provided file was not able to be read as a CSV document!")]
    UnreadableFile,

    #[error("The KPI \"{0}\" does not support benchmarks!")]
    IncompatibleKpiType(String),

    #[error("The provided file is empty!")]
    EmptyFile,

    #[error("The provided file does not have the Experience ID as its first line!")]
    MissingHeader,

    #[error("The provided file does not have a valid Experience ID line!")]
    InvalidHeader,

    #[error("Unable to determine KPI type! Make sure the header line for the data is present and correct!")]
    MissingKpiType,
}

fn get_universe_id(records: &mut StringRecordsIntoIter<File>) -> Result<u64, AnalyticsParseError> {
    let Some(Ok(first_line)) = records.next() else {
        return Err(AnalyticsParseError::EmptyFile);
    };

    if first_line.get(0).ne(&Some("Experience ID")) {
        return Err(AnalyticsParseError::MissingHeader);
    };

    first_line
        .get(1)
        .ok_or(AnalyticsParseError::InvalidHeader)
        .and_then(|value| {
            value
                .parse()
                .map_err(|_| AnalyticsParseError::InvalidHeader)
        })
}

/// Must be called after the first line (Experience ID) has been consumed
fn get_kpi_type(records: &mut StringRecordsIntoIter<File>) -> Result<KpiType, AnalyticsParseError> {
    let Some(Ok(first_line)) = records.next() else {
        return Err(AnalyticsParseError::MissingKpiType);
    };

    if first_line.get(0).ne(&Some("Breakdown")) {
        return Err(AnalyticsParseError::MissingKpiType);
    };

    first_line
        .get(2)
        .ok_or(AnalyticsParseError::MissingKpiType)
        .and_then(|value| {
            KpiType::from_str(value)
                .map_err(|_| AnalyticsParseError::IncompatibleKpiType(value.to_string()))
        })
}

fn parse_record(
    record: StringRecord,
) -> Result<(String, (DateTime<Utc>, DataPoint)), AnalyticsParseError> {
    Ok((
        record
            .get(0)
            .ok_or(AnalyticsParseError::UnreadableFile)?
            .to_string(),
        (
            NaiveDateTime::parse_from_str(
                record.get(1).ok_or(AnalyticsParseError::UnreadableFile)?,
                "%FT%T%.fZ",
            )
            .map_err(|_| AnalyticsParseError::UnreadableFile)?
            .and_utc(),
            record
                .get(2)
                .ok_or(AnalyticsParseError::UnreadableFile)?
                .parse()
                .map_err(|_| AnalyticsParseError::UnreadableFile)?,
        ),
    ))
}

pub fn parse_analytics_file(file: &PathBuf) -> Result<AnalyticsData, AnalyticsParseError> {
    let Ok(reader) = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(file)
    else {
        return Err(AnalyticsParseError::UnreadableFile);
    };

    let mut records = reader.into_records();

    info!("Finding Experience ID...");

    let universe_id = get_universe_id(&mut records)?;

    info!("Found Experience ID {}", universe_id);

    info!("Finding KPI type...");

    let kpi_type = get_kpi_type(&mut records)?;

    info!("Found KPI type {}", kpi_type);

    let mut data: HashMap<String, Vec<(DateTime<Utc>, DataPoint)>> = HashMap::new();

    info!("Collecting data records...");

    for record in records {
        let Ok(record) = record else { continue };
        let result = parse_record(record);
        if let Ok((name, result)) = result {
            if let std::collections::hash_map::Entry::Vacant(e) = data.entry(name.clone()) {
                e.insert(vec![result]);
            } else {
                data.get_mut(&name).unwrap().push(result);
            }
        }
    }

    if data.is_empty() {
        return Err(AnalyticsParseError::EmptyFile);
    }

    info!(
        "Found {} series totalling {} records",
        data.len(),
        data.values().map(|value| value.len()).sum::<usize>()
    );

    Ok(AnalyticsData {
        universe_id,
        kpi_type,
        data,
    })
}
