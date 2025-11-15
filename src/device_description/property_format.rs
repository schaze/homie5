use core::fmt::Display;
use core::hash::Hash;
use core::iter::Iterator;
use core::ops::{RangeFrom, RangeInclusive, RangeTo};
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::HomieDataType;

use super::number_ranges::{FloatRange, IntegerRange};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, PartialOrd)]
pub enum HomiePropertyFormat {
    FloatRange(FloatRange),
    IntegerRange(IntegerRange),
    Enum(Vec<String>),
    Color(Vec<ColorFormat>),
    Boolean(BooleanFormat),
    Json(String), // Raw JSON schema as string
    Custom(String),
    Empty,
}

impl HomiePropertyFormat {
    pub fn format_is_empty(f: &HomiePropertyFormat) -> bool {
        f.is_empty()
    }
    pub fn is_empty(&self) -> bool {
        match self {
            HomiePropertyFormat::FloatRange(r) => r.is_empty(),
            HomiePropertyFormat::IntegerRange(r) => r.is_empty(),
            HomiePropertyFormat::Enum(values) => values.is_empty(),
            HomiePropertyFormat::Color(formats) => formats.is_empty(),
            HomiePropertyFormat::Boolean(bf) => bf.is_empty(),
            HomiePropertyFormat::Json(data) => data.is_empty(),
            HomiePropertyFormat::Custom(data) => data.is_empty(),
            HomiePropertyFormat::Empty => true,
        }
    }
}

// Implement string representation of HomiePropertyFormat for serialization
impl Display for HomiePropertyFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HomiePropertyFormat::FloatRange(fr) => fr.fmt(f),
            HomiePropertyFormat::IntegerRange(ir) => ir.fmt(f),
            HomiePropertyFormat::Enum(values) => {
                write!(f, "{}", values.join(","))
            }
            HomiePropertyFormat::Color(formats) => {
                write!(
                    f,
                    "{}",
                    formats.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(",")
                )
            }
            HomiePropertyFormat::Boolean(bf) => bf.fmt(f),
            HomiePropertyFormat::Json(data) => write!(f, "{}", data),
            HomiePropertyFormat::Custom(data) => write!(f, "{}", data),
            HomiePropertyFormat::Empty => write!(f, ""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, PartialOrd)]
pub struct BooleanFormat {
    pub false_val: String,
    pub true_val: String,
}

impl BooleanFormat {
    pub fn is_empty(&self) -> bool {
        self.false_val.is_empty() && self.true_val.is_empty()
    }
}

impl Display for BooleanFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{},{}", self.false_val, self.true_val)
    }
}

impl FromStr for BooleanFormat {
    type Err = HomiePropertyFormatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens = s.split(',').collect::<Vec<&str>>();
        if tokens.len() != 2 {
            return Err(HomiePropertyFormatError::BooleanFormatError);
        }
        if tokens[0].is_empty() || tokens[1].is_empty() || tokens[0] == tokens[1] {
            return Err(HomiePropertyFormatError::BooleanFormatError);
        }

        Ok(Self {
            false_val: tokens[0].to_owned(),
            true_val: tokens[1].to_owned(),
        })
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum ColorFormat {
    Rgb,
    Hsv,
    Xyz,
}

impl FromStr for ColorFormat {
    type Err = HomiePropertyFormatError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rgb" => Ok(Self::Rgb),
            "hsv" => Ok(Self::Hsv),
            "xyz" => Ok(Self::Xyz),
            _ => Err(HomiePropertyFormatError::ColorFormatError),
        }
    }
}

impl Display for ColorFormat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ColorFormat::Rgb => write!(f, "rgb"),
            ColorFormat::Hsv => write!(f, "hsv"),
            ColorFormat::Xyz => write!(f, "xyz"),
        }
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum HomiePropertyFormatError {
    #[error("Cannot parse number range format")]
    RangeFormatError,
    #[error("Cannot parse color format")]
    ColorFormatError,
    #[error("Cannot parsen boolean format")]
    BooleanFormatError,
}

impl HomiePropertyFormat {
    pub fn parse(raw: &str, datatype: &HomieDataType) -> Result<Self, HomiePropertyFormatError> {
        if raw.is_empty() {
            return Ok(HomiePropertyFormat::Empty);
        }
        match datatype {
            HomieDataType::Float => {
                let fr = FloatRange::parse(raw)?;
                if fr.is_empty() {
                    Ok(HomiePropertyFormat::Empty)
                } else {
                    Ok(HomiePropertyFormat::FloatRange(fr))
                }
            }
            HomieDataType::Integer => {
                let ir = IntegerRange::parse(raw)?;
                if ir.is_empty() {
                    Ok(HomiePropertyFormat::Empty)
                } else {
                    Ok(HomiePropertyFormat::IntegerRange(ir))
                }
            }
            HomieDataType::Enum => Ok(HomiePropertyFormat::Enum(
                raw.split(',').map(|s| s.to_owned()).collect(),
            )),
            HomieDataType::Color => {
                let mut formats = Vec::new();
                for format in raw.split(',') {
                    if let Ok(cf) = format.parse::<ColorFormat>() {
                        formats.push(cf);
                    } else {
                        return Err(HomiePropertyFormatError::ColorFormatError);
                    }
                }
                Ok(Self::Color(formats))
            }
            HomieDataType::Boolean => Ok(Self::Boolean(BooleanFormat::from_str(raw)?)),
            HomieDataType::JSON => Ok(Self::Json(raw.to_owned())), // todo: we need to check if
            // this contains valid json
            // string data
            _ => Ok(Self::Custom(raw.to_owned())),
        }
    }
}

impl From<FloatRange> for HomiePropertyFormat {
    fn from(value: FloatRange) -> Self {
        HomiePropertyFormat::FloatRange(value)
    }
}

impl From<RangeInclusive<f64>> for HomiePropertyFormat {
    fn from(value: RangeInclusive<f64>) -> Self {
        HomiePropertyFormat::FloatRange(FloatRange {
            min: Some(*value.start()),
            max: Some(*value.end()),
            step: None,
        })
    }
}

impl From<RangeTo<f64>> for HomiePropertyFormat {
    fn from(value: RangeTo<f64>) -> Self {
        HomiePropertyFormat::FloatRange(FloatRange {
            min: None,
            max: Some(value.end),
            step: None,
        })
    }
}

impl From<RangeFrom<f64>> for HomiePropertyFormat {
    fn from(value: RangeFrom<f64>) -> Self {
        HomiePropertyFormat::FloatRange(FloatRange {
            min: Some(value.start),
            max: None,
            step: None,
        })
    }
}

impl From<IntegerRange> for HomiePropertyFormat {
    fn from(value: IntegerRange) -> Self {
        HomiePropertyFormat::IntegerRange(value)
    }
}

impl From<RangeInclusive<i64>> for HomiePropertyFormat {
    fn from(value: RangeInclusive<i64>) -> Self {
        HomiePropertyFormat::IntegerRange(IntegerRange {
            min: Some(*value.start()),
            max: Some(*value.end()),
            step: None,
        })
    }
}

impl From<RangeTo<i64>> for HomiePropertyFormat {
    fn from(value: RangeTo<i64>) -> Self {
        HomiePropertyFormat::IntegerRange(IntegerRange {
            min: None,
            max: Some(value.end),
            step: None,
        })
    }
}

impl From<RangeFrom<i64>> for HomiePropertyFormat {
    fn from(value: RangeFrom<i64>) -> Self {
        HomiePropertyFormat::IntegerRange(IntegerRange {
            min: Some(value.start),
            max: None,
            step: None,
        })
    }
}

impl From<BooleanFormat> for HomiePropertyFormat {
    fn from(value: BooleanFormat) -> Self {
        HomiePropertyFormat::Boolean(value)
    }
}

impl From<&[ColorFormat]> for HomiePropertyFormat {
    fn from(value: &[ColorFormat]) -> Self {
        HomiePropertyFormat::Color(value.to_vec())
    }
}

impl From<Vec<ColorFormat>> for HomiePropertyFormat {
    fn from(value: Vec<ColorFormat>) -> Self {
        HomiePropertyFormat::Color(value)
    }
}
