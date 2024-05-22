use chrono::{DateTime, Utc};
use clap::builder::TypedValueParser;
use derive_more::{Add, Sub};
use fixed::types::I32F32;
use plotters::coord::ranged1d::{KeyPointHint, NoDefaultFormatting, ValueFormatter};
use plotters::data::float::FloatPrettyPrinter;
use plotters::prelude::Ranged;
use serde::{Deserialize, Serialize};
use std::ops::{AddAssign, Div, Mul, Range, Sub, SubAssign};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Copy, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Sub, Add, Debug)]
pub enum DataPoint {
    Float(I32F32),
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
        if s.matches(char::is_numeric).collect::<String>() == s {
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

impl From<I32F32> for DataPoint {
    fn from(value: I32F32) -> Self {
        DataPoint::Float(value)
    }
}

impl From<u64> for DataPoint {
    fn from(value: u64) -> Self {
        DataPoint::Integer(value)
    }
}

impl Into<f64> for DataPoint {
    fn into(self) -> f64 {
        match self {
            DataPoint::Float(value) => value.to_num(),
            DataPoint::Integer(value) => value as f64,
        }
    }
}

impl From<f64> for DataPoint {
    fn from(value: f64) -> Self {
        DataPoint::Float(I32F32::from_num(value))
    }
}

impl Mul for DataPoint {
    type Output = DataPoint;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            DataPoint::Float(value_lhs) => {
                let DataPoint::Float(value_rhs) = rhs else {
                    panic!(
                        "Attempted to perform data point arithmetic on different data point types!"
                    )
                };

                DataPoint::Float(value_lhs * value_rhs)
            }
            DataPoint::Integer(value_lhs) => {
                let DataPoint::Integer(value_rhs) = rhs else {
                    panic!(
                        "Attempted to perform data point arithmetic on different data point types!"
                    )
                };

                DataPoint::Integer(value_lhs * value_rhs)
            }
        }
    }
}

impl Div<u32> for DataPoint {
    type Output = DataPoint;

    fn div(self, rhs: u32) -> Self::Output {
        match self {
            DataPoint::Float(value) => DataPoint::from(value.to_num::<f64>() / rhs as f64),
            DataPoint::Integer(value) => DataPoint::Integer(value / rhs as u64),
        }
    }
}

impl SubAssign for DataPoint {
    fn sub_assign(&mut self, rhs: Self) {
        *self = (self.to_owned() - rhs).unwrap()
    }
}

impl AddAssign for DataPoint {
    fn add_assign(&mut self, rhs: Self) {
        *self = (self.to_owned() + rhs).unwrap()
    }
}

pub struct RangedDataPoint(DataPoint, DataPoint);

impl Ranged for RangedDataPoint {
    type FormatOption = NoDefaultFormatting;
    type ValueType = DataPoint;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        // Corner case: If we have a range that have only one value,
        // then we just assign everything to the only point
        if self.1 == self.0 {
            return (limit.1 - limit.0) / 2;
        }

        let logic_length: f64 = (<DataPoint as Into<f64>>::into(value.to_owned())
            - <DataPoint as Into<f64>>::into(self.0.clone()))
            / (<DataPoint as Into<f64>>::into(self.1.clone())
                - <DataPoint as Into<f64>>::into(self.0.clone()));

        let actual_length = limit.1 - limit.0;

        if actual_length == 0 {
            return limit.1;
        }

        if logic_length.is_infinite() {
            return if logic_length.is_sign_positive() {
                limit.1
            } else {
                limit.0
            };
        }

        return if actual_length > 0 {
            limit.0 + (actual_length as f64 * logic_length + 1e-3).floor() as i32
        } else {
            limit.0 + (actual_length as f64 * logic_length - 1e-3).ceil() as i32
        };
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        let max_points = hint.max_num_points();

        if max_points == 0 {
            return vec![];
        }

        let range: (f64, f64) = (
            self.0.clone().min(self.1.clone()).into(),
            self.1.clone().max(self.0.clone()).into(),
        );

        assert!(!(range.0.is_nan() || range.1.is_nan()));

        if (range.0 - range.1).abs() < f64::EPSILON {
            return vec![DataPoint::from(range.0)];
        }

        let mut scale = 10f64.powf((range.1 - range.0).log(10.0).floor());
        // The value granularity controls how we round the values.
        // To avoid generating key points like 1.00000000001, we round to the nearest multiple of the
        // value granularity.
        // By default, we make the granularity as the 1/10 of the scale.
        let mut value_granularity = scale / 10.0;
        fn rem_euclid(a: f64, b: f64) -> f64 {
            let ret = if b > 0.0 {
                a - (a / b).floor() * b
            } else {
                a - (a / b).ceil() * b
            };
            if (ret - b).abs() < f64::EPSILON {
                0.0
            } else {
                ret
            }
        }

        // At this point we need to make sure that the loop invariant:
        // The scale must yield number of points than requested
        if 1 + ((range.1 - range.0) / scale).floor() as usize > max_points {
            scale *= 10.0;
            value_granularity *= 10.0;
        }

        'outer: loop {
            let old_scale = scale;
            for nxt in [2.0, 5.0, 10.0].iter() {
                let mut new_left = range.0 - rem_euclid(range.0, old_scale / nxt);
                if new_left < range.0 {
                    new_left += old_scale / nxt;
                }
                let new_right = range.1 - rem_euclid(range.1, old_scale / nxt);

                let npoints = 1.0 + ((new_right - new_left) / old_scale * nxt);

                if npoints.round() as usize > max_points {
                    break 'outer;
                }

                scale = old_scale / nxt;
            }
            scale = old_scale / 10.0;
            value_granularity /= 10.0;
        }

        let mut ret = vec![];
        // In some extreme cases, left might be too big, so that (left + scale) - left == 0 due to
        // floating point error.
        // In this case, we may loop forever. To avoid this, we need to use two variables to store
        // the current left value. So we need keep a left_base and a left_relative.
        let left = {
            let mut value = range.0 - rem_euclid(range.0, scale);
            if value < range.0 {
                value += scale;
            }
            value
        };
        let left_base = (left / value_granularity).floor() * value_granularity;
        let mut left_relative = left - left_base;
        let right = range.1 - rem_euclid(range.1, scale);
        while (right - left_relative - left_base) >= -f64::EPSILON {
            let new_left_relative = (left_relative / value_granularity).round() * value_granularity;
            if new_left_relative < 0.0 {
                left_relative += value_granularity;
            }
            ret.push((left_relative + left_base).into());
            left_relative += scale;
        }
        return ret;
    }

    fn range(&self) -> Range<Self::ValueType> {
        self.0..self.1
    }
}

impl ValueFormatter<DataPoint> for RangedDataPoint {
    fn format(_value: &DataPoint) -> String {
        match _value {
            DataPoint::Integer(value) => value.to_string(),
            DataPoint::Float(value) => FloatPrettyPrinter {
                allow_scientific: false,
                min_decimal: 1,
                max_decimal: 5,
            }
            .print(value.to_num::<f64>()),
        }
    }
}

pub fn get_data_range(
    data: &Vec<(DateTime<Utc>, DataPoint)>,
) -> (Range<DateTime<Utc>>, RangedDataPoint) {
    let mut value_range = data
        .iter()
        .min_by(|(_, point1), (_, point2)| point1.cmp(point2))
        .expect("Failed to obtain least data point!")
        .1
        ..data
            .iter()
            .max_by(|(_, point1), (_, point2)| point1.cmp(point2))
            .expect("Failed to obtain greatest data point!")
            .1;

    // add 10% boundary to make sure data points have margin
    let value_range_len = (value_range.end.clone() - value_range.start.clone()).unwrap();
    value_range.start -= (value_range_len.clone() / 10).min(value_range.start);
    value_range.end += value_range_len / 10;

    return (
        data.iter()
            .min_by(|(date1, _), (date2, _)| date1.cmp(date2))
            .expect("Failed to obtain earliest date!")
            .0
            ..data
                .iter()
                .max_by(|(date1, _), (date2, _)| date1.cmp(date2))
                .expect("Failed to obtain latest date!")
                .0,
        RangedDataPoint(value_range.start, value_range.end),
    );
}
