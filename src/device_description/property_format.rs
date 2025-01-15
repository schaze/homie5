use std::fmt::Display;
use std::hash::Hash;
use std::iter::Iterator;
use std::str::FromStr;

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
    Boolean { false_val: String, true_val: String },
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
            HomiePropertyFormat::Boolean { false_val, true_val } => false_val.is_empty() || true_val.is_empty(),
            HomiePropertyFormat::Json(data) => data.is_empty(),
            HomiePropertyFormat::Custom(data) => data.is_empty(),
            HomiePropertyFormat::Empty => true,
        }
    }
}

// Implement string representation of HomiePropertyFormat for serialization
impl Display for HomiePropertyFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            HomiePropertyFormat::Boolean { false_val, true_val } => {
                write!(f, "{},{}", false_val, true_val)
            }
            HomiePropertyFormat::Json(data) => write!(f, "{}", data),
            HomiePropertyFormat::Custom(data) => write!(f, "{}", data),
            HomiePropertyFormat::Empty => write!(f, ""),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash, PartialOrd)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            HomieDataType::Boolean => {
                let tokens = raw.split(',').collect::<Vec<&str>>();
                if tokens.len() != 2 {
                    return Err(HomiePropertyFormatError::BooleanFormatError);
                }
                if tokens[0].is_empty() || tokens[1].is_empty() || tokens[0] == tokens[1] {
                    return Err(HomiePropertyFormatError::BooleanFormatError);
                }

                Ok(Self::Boolean {
                    false_val: tokens[0].to_owned(),
                    true_val: tokens[1].to_owned(),
                })
            }
            HomieDataType::JSON => Ok(Self::Json(raw.to_owned())), // todo: we need to check if
            // this contains valid json
            // string data
            _ => Ok(Self::Custom(raw.to_owned())),
        }
    }
}
