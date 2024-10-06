use std::{fmt::Display, str::FromStr};

use crate::{
    device_description::{ColorFormat, FloatRange, HomiePropertyDescription, HomiePropertyFormat, IntegerRange},
    HomieDataType,
};

#[derive(Debug, PartialEq)]
pub struct Homie5ValueConversionError;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum HomieColorValue {
    RGB(i64, i64, i64),
    HSV(i64, i64, i64),
    XYZ(f64, f64, f64),
}

impl HomieColorValue {
    pub fn new_xyz(x: f64, y: f64) -> Self {
        HomieColorValue::XYZ(x, y, 1.0 - x - y)
    }
}

impl From<HomieColorValue> for String {
    fn from(value: HomieColorValue) -> Self {
        value.to_string()
    }
}

impl Display for HomieColorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HomieColorValue::RGB(r, g, b) => write!(f, "rgb,{},{},{}", r, g, b),
            HomieColorValue::HSV(h, s, v) => write!(f, "hsv,{},{},{}", h, s, v),
            HomieColorValue::XYZ(x, y, _) => write!(f, "xyz,{},{}", x, y),
        }
    }
}

//impl ToString for HomieColorValue {
//    fn to_string(&self) -> String {
//        match self {
//            HomieColorValue::RGB(r, g, b) => format!("rgb,{},{},{}", r, g, b),
//            HomieColorValue::HSV(h, s, v) => format!("hsv,{},{},{}", h, s, v),
//            HomieColorValue::XYZ(x, y, _) => format!("xyz,{},{}", x, y),
//        }
//    }
//}

impl FromStr for HomieColorValue {
    type Err = Homie5ValueConversionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = str::split(s, ',');
        match tokens.next() {
            Some("rgb") => {
                if let (Some(Ok(r)), Some(Ok(g)), Some(Ok(b))) = (
                    tokens.next().map(|r| r.parse::<i64>()),
                    tokens.next().map(|g| g.parse::<i64>()),
                    tokens.next().map(|b| b.parse::<i64>()),
                ) {
                    return Ok(Self::RGB(r, g, b));
                }
            }
            Some("hsv") => {
                if let (Some(Ok(h)), Some(Ok(s)), Some(Ok(v))) = (
                    tokens.next().map(|h| h.parse::<i64>()),
                    tokens.next().map(|s| s.parse::<i64>()),
                    tokens.next().map(|v| v.parse::<i64>()),
                ) {
                    return Ok(Self::HSV(h, s, v));
                }
            }
            Some("xyz") => {
                if let (Some(Ok(x)), Some(Ok(y))) = (
                    tokens.next().map(|x| x.parse::<f64>()),
                    tokens.next().map(|y| y.parse::<f64>()),
                ) {
                    return Ok(Self::XYZ(x, y, 1.0 - x - y));
                }
            }
            _ => return Err(Homie5ValueConversionError),
        }
        Err(Homie5ValueConversionError)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum HomieValue {
    #[default]
    Empty,
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Enum(String),
    Color(HomieColorValue),
    DateTime(chrono::DateTime<chrono::Utc>),
    Duration(chrono::Duration),
    JSON(serde_json::Value),
}

//#[allow(clippy::from_over_into)]
//impl Into<String> for HomieValue {
//    fn into(self) -> String {
//        self.to_string()
//    }
//}
//
//#[allow(clippy::from_over_into)]
//impl Into<String> for &HomieValue {
//    fn into(self) -> String {
//        self.to_string()
//    }
//}

impl Display for HomieValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HomieValue::Empty => write!(f, ""),
            HomieValue::String(value) => write!(f, "{}", value),
            HomieValue::Integer(value) => write!(f, "{}", value),
            HomieValue::Float(value) => write!(f, "{}", value),
            HomieValue::Bool(value) => write!(f, "{}", value),
            HomieValue::Enum(value) => write!(f, "{}", value),
            HomieValue::Color(value) => write!(f, "{}", value),
            HomieValue::DateTime(value) => write!(f, "{}", value.to_rfc3339()),
            HomieValue::Duration(value) => write!(f, "{}", value),
            HomieValue::JSON(value) => {
                if let Ok(val) = serde_json::to_string(value) {
                    write!(f, "{}", val)
                } else {
                    write!(f, "")
                }
            }
        }
    }
}
impl From<HomieValue> for String {
    fn from(value: HomieValue) -> Self {
        value.to_string()
    }
}
impl From<&HomieValue> for String {
    fn from(value: &HomieValue) -> Self {
        value.to_string() // or define custom logic
    }
}

//impl ToString for HomieValue {
//    fn to_string(&self) -> String {
//        match self {
//            HomieValue::Empty => "".to_owned(),
//            HomieValue::String(value) => value.to_owned(),
//            HomieValue::Integer(value) => value.to_string(),
//            HomieValue::Float(value) => value.to_string(),
//            HomieValue::Bool(value) => value.to_string(),
//            HomieValue::Enum(value) => value.to_owned(),
//            HomieValue::Color(value) => value.to_string(),
//            HomieValue::DateTime(value) => value.to_rfc3339(),
//            HomieValue::Duration(value) => value.to_string(),
//            HomieValue::JSON(value) => {
//                if let Ok(val) = serde_json::to_string(value) {
//                    val.to_owned()
//                } else {
//                    "".to_owned()
//                }
//            }
//        }
//    }
//}

impl HomieValue {
    pub fn parse(
        raw: &str,
        property_desc: &HomiePropertyDescription,
    ) -> Result<HomieValue, Homie5ValueConversionError> {
        if raw.is_empty() {
            return Ok(HomieValue::Empty);
        }
        match &property_desc.datatype {
            HomieDataType::Integer => {
                if let Ok(value) = raw.parse::<i64>() {
                    if let HomiePropertyFormat::IntegerRange(ir) = &property_desc.format {
                        let v = Self::validate_int(value, ir)?;
                        return Ok(HomieValue::Integer(v));
                    }
                    return Ok(HomieValue::Integer(value));
                }
            }
            HomieDataType::Float => {
                if let Ok(value) = raw.parse::<f64>() {
                    if let HomiePropertyFormat::FloatRange(fr) = &property_desc.format {
                        let v = Self::validate_float(value, fr)?;
                        return Ok(HomieValue::Float(v));
                    }
                    return Ok(HomieValue::Float(value));
                }
            }
            HomieDataType::Boolean => {
                if let Ok(value) = raw.parse::<bool>() {
                    return Ok(HomieValue::Bool(value));
                }
            }
            HomieDataType::String => {
                return Ok(HomieValue::String(raw.to_owned()));
            }
            HomieDataType::Enum => {
                let HomiePropertyFormat::Enum(values) = &property_desc.format else {
                    return Err(Homie5ValueConversionError);
                };
                let string_val = raw.to_owned();
                if values.contains(&string_val) {
                    return Ok(HomieValue::Enum(string_val));
                } else {
                    return Err(Homie5ValueConversionError);
                }
            }
            HomieDataType::Color => {
                let color_value = raw.parse::<HomieColorValue>().map(HomieValue::Color)?;
                let HomieValue::Color(color) = &color_value else {
                    return Err(Homie5ValueConversionError);
                };
                if !property_desc.format.is_empty() {
                    let HomiePropertyFormat::Color(formats) = &property_desc.format else {
                        return Err(Homie5ValueConversionError);
                    };
                    match color {
                        HomieColorValue::RGB(_, _, _) => {
                            if !formats.contains(&ColorFormat::Rgb) {
                                return Err(Homie5ValueConversionError);
                            }
                        }
                        HomieColorValue::HSV(_, _, _) => {
                            if !formats.contains(&ColorFormat::Hsv) {
                                return Err(Homie5ValueConversionError);
                            }
                        }
                        HomieColorValue::XYZ(_, _, _) => {
                            if !formats.contains(&ColorFormat::Xyz) {
                                return Err(Homie5ValueConversionError);
                            }
                        }
                    }
                    return Ok(color_value);
                } else {
                    return Ok(color_value);
                }
            }
            HomieDataType::Datetime => {
                return Self::flexible_datetime_parser(raw).map(HomieValue::DateTime);
            }
            HomieDataType::Duration => {
                return Self::parse_duration(raw).map(HomieValue::Duration);
            }
            HomieDataType::JSON => {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
                    return Ok(HomieValue::JSON(value));
                }
            }
        }

        Err(Homie5ValueConversionError)
    }

    fn parse_duration(s: &str) -> Result<chrono::Duration, Homie5ValueConversionError> {
        let re = regex::Regex::new(r"^PT(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?$").unwrap();
        if let Some(captures) = re.captures(s) {
            let hours: i64 = captures.get(1).map_or(0, |m| m.as_str().parse().unwrap());
            let minutes: i64 = captures.get(2).map_or(0, |m| m.as_str().parse().unwrap());
            let seconds: i64 = captures.get(3).map_or(0, |m| m.as_str().parse().unwrap());

            return Ok(chrono::Duration::seconds(hours * 3600 + minutes * 60 + seconds));
        }
        Err(Homie5ValueConversionError)
    }

    // flexible deserialization approach as timestamps are hard and we want to keep compatibility
    // high
    fn flexible_datetime_parser(s: &str) -> Result<chrono::DateTime<chrono::Utc>, Homie5ValueConversionError> {
        // try standard RFC3339 compliant parsing
        chrono::DateTime::parse_from_rfc3339(s).map_or_else(
            |_| {
                // if it does not work we try parsing it from a string representation without
                // seconds (we strip the last character as this is supposed to be a Z for UTC
                // timezone
                let s = if let Some(rest) = s.strip_suffix('Z') { rest } else { s };
                chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").map_or_else(
                    |_| {
                        // if this also does not work we try parsing it from a string representation with
                        // fractional seconds
                        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").map_or_else(
                            |_| Err(Homie5ValueConversionError), // if this also does not work, we give
                            // up
                            |ndt| {
                                Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                                    ndt,
                                    chrono::Utc,
                                ))
                            },
                        )
                    },
                    |ndt| {
                        Ok(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                            ndt,
                            chrono::Utc,
                        ))
                    },
                )
            },
            |d| Ok(chrono::DateTime::<chrono::Utc>::from(d)),
        )
    }

    fn validate_float(value: f64, fr: &FloatRange) -> Result<f64, Homie5ValueConversionError> {
        let base = fr.min.or(fr.max).unwrap_or(value);
        let rounded = match fr.step {
            Some(s) if s > 0.0 => (value - base).div_euclid(s).round().mul_add(s, base),
            _ => value,
        };

        if fr.min.map_or(true, |m| rounded >= m) && fr.max.map_or(true, |m| rounded <= m) {
            Ok(rounded)
        } else {
            Err(Homie5ValueConversionError)
        }
    }

    fn validate_int(value: i64, ir: &IntegerRange) -> Result<i64, Homie5ValueConversionError> {
        let base = ir.min.or(ir.max).unwrap_or(value);
        let rounded = match ir.step {
            Some(s) if s > 0 => ((value - base) as f64 / s as f64).round() as i64 * s + base,
            _ => value,
        };

        if ir.min.map_or(true, |m| rounded >= m) && ir.max.map_or(true, |m| rounded <= m) {
            Ok(rounded)
        } else {
            Err(Homie5ValueConversionError)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::device_description::PropertyDescriptionBuilder;

    use super::*;

    #[test]
    fn test_integer_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Integer)
            .name(Some("test".to_owned()))
            .build();
        assert_eq!(HomieValue::parse("122", &desc), Ok(HomieValue::Integer(122)));
    }

    #[test]
    fn test_integer_nok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Integer)
            .name(Some("test".to_owned()))
            .build();
        assert_eq!(HomieValue::parse("bla2", &desc), Err(Homie5ValueConversionError));
        assert_eq!(HomieValue::parse("122.22", &desc), Err(Homie5ValueConversionError));
        assert_eq!(HomieValue::parse("122,22", &desc), Err(Homie5ValueConversionError));
        assert_eq!(HomieValue::parse(" 122", &desc), Err(Homie5ValueConversionError));
    }

    #[test]
    fn test_float_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Float).build();
        assert_eq!(HomieValue::parse("122", &desc), Ok(HomieValue::Float(122.0)));
        assert_eq!(HomieValue::parse("122.12", &desc), Ok(HomieValue::Float(122.12)));
    }

    #[test]
    fn test_float_nok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Float).build();
        assert_eq!(HomieValue::parse("bla2", &desc), Err(Homie5ValueConversionError));
        assert_eq!(HomieValue::parse("122,22", &desc), Err(Homie5ValueConversionError));
        assert_eq!(HomieValue::parse(" 122", &desc), Err(Homie5ValueConversionError));
    }

    #[test]
    fn test_bool_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Boolean).build();
        assert_eq!(HomieValue::parse("true", &desc), Ok(HomieValue::Bool(true)));
        assert_eq!(HomieValue::parse("false", &desc), Ok(HomieValue::Bool(false)));
    }

    #[test]
    fn test_bool_nok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Boolean).build();
        assert_eq!(HomieValue::parse("bla2", &desc), Err(Homie5ValueConversionError));
        assert_eq!(HomieValue::parse("TRUE", &desc), Err(Homie5ValueConversionError));
        assert_eq!(HomieValue::parse("False", &desc), Err(Homie5ValueConversionError));
    }

    #[test]
    fn test_string_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::String).build();
        assert_eq!(
            HomieValue::parse("blah", &desc),
            Ok(HomieValue::String("blah".to_owned()))
        );
    }

    #[test]
    fn test_enum_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Enum)
            .format(HomiePropertyFormat::Enum(vec!["blah".to_owned()]))
            .build();
        assert_eq!(
            HomieValue::parse("blah", &desc),
            Ok(HomieValue::Enum("blah".to_owned()))
        );
    }

    #[test]
    fn test_color_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Color).build();
        assert_eq!(
            HomieValue::parse("rgb,12,55,14", &desc),
            Ok(HomieValue::Color(HomieColorValue::RGB(12, 55, 14)))
        );
        assert_eq!(
            HomieValue::parse("hsv,112,155,55", &desc),
            Ok(HomieValue::Color(HomieColorValue::HSV(112, 155, 55)))
        );
        assert_eq!(
            HomieValue::parse("xyz,0.33453,0.123456", &desc),
            Ok(HomieValue::Color(HomieColorValue::XYZ(
                0.33453,
                0.123456,
                1.0 - 0.33453 - 0.123456
            )))
        );
    }

    #[test]
    fn test_color_nok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Color).build();
        assert_eq!(HomieValue::parse("rgb,12,55", &desc), Err(Homie5ValueConversionError));
        assert_eq!(
            HomieValue::parse("HSV,12,55,14", &desc),
            Err(Homie5ValueConversionError)
        );
        assert_eq!(
            HomieValue::parse("rgb ,12,55,14", &desc),
            Err(Homie5ValueConversionError)
        );
        assert_eq!(HomieValue::parse("xyz/12,55", &desc), Err(Homie5ValueConversionError));
    }

    #[test]
    fn test_datetime_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Datetime).build();
        assert_eq!(
            HomieValue::parse("2023-09-26T10:54:59+00:00", &desc),
            Ok(HomieValue::DateTime(
                chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
            ))
        );
        assert_eq!(
            HomieValue::parse("2023-09-26T11:54:59+01:00", &desc),
            Ok(HomieValue::DateTime(
                chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
            ))
        );
        assert_eq!(
            HomieValue::parse("2023-09-26T10:54:59Z", &desc),
            Ok(HomieValue::DateTime(
                chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
            ))
        );
        assert_eq!(
            HomieValue::parse("2023-09-26T10:54:59", &desc),
            Ok(HomieValue::DateTime(
                chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
            ))
        );
        assert_eq!(
            HomieValue::parse("2023-09-26T10:54:59.100", &desc),
            Ok(HomieValue::DateTime(
                chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 100000000).unwrap()
            ))
        );
    }
    #[test]
    fn test_duration_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Duration).build();
        assert_eq!(
            HomieValue::parse("PT12H4M2S", &desc),
            Ok(HomieValue::Duration(
                chrono::Duration::from_std(std::time::Duration::from_secs(12 * 60 * 60 + 4 * 60 + 2)).unwrap()
            ))
        );
        assert_eq!(
            HomieValue::parse("PT43442S", &desc),
            Ok(HomieValue::Duration(
                chrono::Duration::from_std(std::time::Duration::from_secs(12 * 60 * 60 + 4 * 60 + 2)).unwrap()
            ))
        );
    }

    #[test]
    fn test_json_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::JSON).build();
        let json = HomieValue::parse("{ \"test\": \"test\" }", &desc);
        assert!(json.is_ok());
        assert_eq!(
            match json {
                Ok(HomieValue::JSON(data)) => data.get("test").unwrap().as_str().unwrap().to_owned(),
                _ => "".to_owned(),
            },
            "test"
        );
    }

    #[test]
    fn test_json_nok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::JSON).build();
        let json = HomieValue::parse("{ \"test\": failure }", &desc);
        assert!(json.is_err());
    }

    #[test]
    fn test_validation_integer_ok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Float)
            .format(
                // Test cases for FloatRange
                HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(-6.0),
                    max: Some(6.0),
                    step: Some(4.0),
                }),
            )
            .build();

        let float_values = vec![
            "-7.5", "-6.1", "-6.0", "-5.9", "-4.5", "-0.5", "0.0", "1.5", "2.0", "2.5", "4.7", "6.0", "6.5", "8.0",
        ];

        println!("FloatRange Format: {:?}", &desc.format);
        println!("Value | Rounded Value | Result");
        println!("------|---------------|-------");
        for value in float_values {
            match HomieValue::parse(value, &desc) {
                Ok(rounded) => println!("{}   | {:?}          | Success", value, rounded),
                Err(err) => println!("{}   |               | {:?}", value, err),
            }
        }
    }

    #[test]
    fn test_validation_integer_nok() {
        let desc = PropertyDescriptionBuilder::new(HomieDataType::Integer).build();
        let json = HomieValue::parse("{ \"test\": failure }", &desc);
        assert!(json.is_err());
    }
}
