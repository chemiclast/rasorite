use crate::benches::KpiType;
use crate::data::{DataParsingError, DataPoint};
use chrono::{DateTime, Utc};
use csv::StringRecordsIntoIter;
use once_cell::unsync::Lazy;
use regex::Regex;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;

static FILE_NAME_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new("([^ -]+?),").expect("Failed to compile Regex!"));

pub struct AnalyticsData {
    kpi_type: KpiType,
    universe_id: u64,
    data: Vec<(DateTime<Utc>, DataPoint)>,
}

#[derive(Debug, Error)]
enum AnalyticsParseError {
    #[error("The provided file was not able to be read as a CSV document!")]
    UnreadableFile,

    #[error("The KPI \"{0}\" is not supported!")]
    IncompatibleKpiType(String),

    #[error("The provided file is empty!")]
    EmptyFile,

    #[error("The provided file does not have the Experience ID as its first line!")]
    MissingHeader,

    #[error("The provided file does not have a valid Experience ID line!")]
    InvalidHeader,

    #[error("Unable to extract KPI type from file name! Did you rename the file?")]
    MissingKpiType,

    #[error(transparent)]
    DataParsingError { source: DataParsingError },
}

fn get_universe_id(records: &mut StringRecordsIntoIter<File>) -> Result<u64, AnalyticsParseError> {
    let Some(Ok(first_line)) = records.next() else {
        Err(AnalyticsParseError::EmptyFile)
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

pub fn parse_analytics_file(file: PathBuf) -> Result<AnalyticsData, AnalyticsParseError> {
    let Some(kpi_type_captures) = FILE_NAME_PATTERN.captures(file.file_name()?.into()) else {
        Err(AnalyticsParseError::MissingKpiType)
    };

    let Some(kpi_type_match) = kpi_type_captures.get(1).map(|value| value.as_str()) else {
        Err(AnalyticsParseError::MissingKpiType)
    };

    let Ok(kpi_type) = KpiType::from_str(kpi_type_match) else {
        Err(AnalyticsParseError::IncompatibleKpiType(
            kpi_type_match.into_string(),
        ))
    };

    let Ok(mut reader) = csv::ReaderBuilder::new().has_headers(false).from_path(file) else {
        Err(AnalyticsParseError::UnreadableFile)
    };

    let mut records = reader.into_records();

    let universe_id = get_universe_id(&mut records)?;

    // get_universe_id will read the Experience ID line and the next two lines after that line are a blank line and a header line
    records = records.skip(2).into();

    let mut data = Vec::new();

    for Ok(record) in records {
        data.push((record.get(1)?.parse()?, record.get(2)?.parse()?))
    }

    Ok(AnalyticsData {
        universe_id,
        kpi_type,
        data,
    })
}
