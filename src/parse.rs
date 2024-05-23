use crate::data::KpiType;
use crate::data::{DataParsingError, DataPoint};
use chrono::{DateTime, NaiveDateTime, Utc};
use csv::{StringRecord, StringRecordsIntoIter};
use regex::Regex;
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

fn parse_record<'a>(
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
    let Some(kpi_type_captures) = Regex::new("([^ -]+?),")
        .expect("Failed to compile Regex!")
        .captures(
            file.file_name()
                .ok_or(AnalyticsParseError::MissingKpiType)?
                .to_str()
                .ok_or(AnalyticsParseError::MissingKpiType)?,
        )
    else {
        return Err(AnalyticsParseError::MissingKpiType);
    };

    let Some(kpi_type_match) = kpi_type_captures.get(1).map(|value| value.as_str()) else {
        return Err(AnalyticsParseError::MissingKpiType);
    };

    let Ok(kpi_type) = KpiType::from_str(kpi_type_match) else {
        return Err(AnalyticsParseError::IncompatibleKpiType(
            kpi_type_match.to_string(),
        ));
    };

    let Ok(reader) = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_path(file)
    else {
        return Err(AnalyticsParseError::UnreadableFile);
    };

    let mut records = reader.into_records();

    let universe_id = get_universe_id(&mut records)?;

    let mut data: HashMap<String, Vec<(DateTime<Utc>, DataPoint)>> = HashMap::new();

    for record in records {
        let Ok(record) = record else { continue };
        let result = parse_record(record);
        if let Ok((name, result)) = result {
            if data.contains_key(&name) {
                data.get_mut(&name).unwrap().push(result);
            } else {
                data.insert(name, vec![result]);
            }
        }
    }

    if data.is_empty() {
        return Err(AnalyticsParseError::EmptyFile);
    }

    Ok(AnalyticsData {
        universe_id,
        kpi_type,
        data,
    })
}
