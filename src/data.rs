use clap::builder::TypedValueParser;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Serialize, Deserialize)]
pub enum DataPoint {
    Float(f64),
    Integer(u64),
}

#[derive(Debug, Error)]
pub enum DataParsingError {
    #[error("The provided string failed to parse as a data point!")]
    CannotParse,
}

impl FromStr for DataPoint {
    type Err = DataParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.matches(char::is_numeric).eq(s) {
            Ok(DataPoint::Integer(
                s.parse().map_err(|_| DataParsingError::CannotParse)?,
            ))
        } else {
            Ok(DataPoint::Float(
                s.parse().map_err(|_| DataParsingError::CannotParse)?,
            ))
        }
    }
}
