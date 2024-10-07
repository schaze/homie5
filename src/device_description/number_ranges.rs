use std::fmt::Display;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

use super::property_format::HomiePropertyFormatError;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FloatRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
}

impl FloatRange {
    pub fn is_empty(&self) -> bool {
        self.min.is_none() && self.max.is_none() && self.step.is_none()
    }

    pub fn validate_float_range(min: Option<f64>, max: Option<f64>, step: Option<f64>) -> bool {
        if let Some(step) = step {
            if step <= 0.0 {
                return false;
            }
        }
        match (min, max, step) {
            (Some(min), Some(max), None) => {
                if min > max {
                    return false;
                }
            }
            (Some(min), Some(max), Some(step)) => {
                if min > max {
                    return false;
                }
                if step > max - min {
                    return false;
                }
            }
            _ => {}
        }
        true
    }

    pub fn parse(raw: &str) -> Result<Self, HomiePropertyFormatError> {
        let mut start = 0;
        let mut res_index = 0;
        let mut res: [Option<f64>; 3] = [None, None, None];
        for (index, char) in raw.char_indices() {
            if char == ':' {
                let slice = &raw[start..index];
                start = index + 1; // this is safe as a ':' will only use one byte
                if !slice.is_empty() {
                    if let Ok(num) = slice.parse::<f64>() {
                        res[res_index] = Some(num);
                    } else {
                        return Err(HomiePropertyFormatError::RangeFormatError);
                    }
                }
                res_index += 1;
                if res_index > 2 {
                    return Err(HomiePropertyFormatError::RangeFormatError);
                }
            }
        }

        let slice = &raw[start..];
        if !slice.is_empty() {
            if let Ok(num) = slice.parse::<f64>() {
                res[res_index] = Some(num);
            } else {
                return Err(HomiePropertyFormatError::RangeFormatError);
            }
        }
        if !FloatRange::validate_float_range(res[0], res[1], res[2]) {
            return Err(HomiePropertyFormatError::RangeFormatError);
        }
        Ok(Self {
            min: res[0],
            max: res[1],
            step: res[2],
        })
    }
}

// Implement custom Hashing for RangeFormat.
// This does a trivial hashing of f64s as bytes which may cause problems for NaN values and for
// rounding errors.
// However according to the homie convention such numbers are not allowed and rounding should be
// negligeble. Worst case this will lead to an unstable version number generation for the device
// description.
impl Hash for FloatRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(min) = self.min {
            min.to_bits().hash(state);
        }
        if let Some(max) = self.max {
            max.to_bits().hash(state);
        }
        if let Some(step) = self.step {
            step.to_bits().hash(state);
        }
    }
}

impl Display for FloatRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return Err(std::fmt::Error);
        }
        if let Some(min) = self.min {
            if self.max.is_none() && self.step.is_none() {
                write!(f, "{}:", min)?;
            } else {
                write!(f, "{}", min)?;
            }
        }
        if let Some(max) = self.max {
            write!(f, ":{}", max)?;
        } else if self.step.is_some() {
            write!(f, ":")?;
        }
        if let Some(step) = self.step {
            write!(f, ":{}", step)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq)]
pub struct IntegerRange {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub step: Option<i64>,
}

impl IntegerRange {
    pub fn is_empty(&self) -> bool {
        self.min.is_none() && self.max.is_none() && self.step.is_none()
    }

    pub fn validate_integer_range(min: Option<i64>, max: Option<i64>, step: Option<i64>) -> bool {
        if let Some(step) = step {
            if step <= 0 {
                return false;
            }
        }
        match (min, max, step) {
            (Some(min), Some(max), None) => {
                if min > max {
                    return false;
                }
            }
            (Some(min), Some(max), Some(step)) => {
                if min > max {
                    return false;
                }
                if step > max - min {
                    return false;
                }
            }
            _ => {}
        }
        true
    }

    pub fn parse(raw: &str) -> Result<Self, HomiePropertyFormatError> {
        let mut start = 0;
        let mut res_index = 0;
        let mut res: [Option<i64>; 3] = [None, None, None];
        for (index, char) in raw.char_indices() {
            if char == ':' {
                let slice = &raw[start..index];
                start = index + 1; // this is safe as a ':' will only use one byte
                if !slice.is_empty() {
                    if let Ok(num) = slice.parse::<i64>() {
                        res[res_index] = Some(num);
                    } else {
                        return Err(HomiePropertyFormatError::RangeFormatError);
                    }
                }
                res_index += 1;
                if res_index > 2 {
                    return Err(HomiePropertyFormatError::RangeFormatError);
                }
            }
        }

        let slice = &raw[start..];
        if !slice.is_empty() {
            if let Ok(num) = slice.parse::<i64>() {
                res[res_index] = Some(num);
            } else {
                return Err(HomiePropertyFormatError::RangeFormatError);
            }
        }
        if !Self::validate_integer_range(res[0], res[1], res[2]) {
            return Err(HomiePropertyFormatError::RangeFormatError);
        }
        Ok(Self {
            min: res[0],
            max: res[1],
            step: res[2],
        })
    }
}

impl Display for IntegerRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return Err(std::fmt::Error);
        }
        if let Some(min) = self.min {
            if self.max.is_none() && self.step.is_none() {
                write!(f, "{}:", min)?;
            } else {
                write!(f, "{}", min)?;
            }
        }
        if let Some(max) = self.max {
            write!(f, ":{}", max)?;
        } else if self.step.is_some() {
            write!(f, ":")?;
        }
        if let Some(step) = self.step {
            write!(f, ":{}", step)?;
        }
        Ok(())
    }
}
