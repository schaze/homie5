use std::{fmt::Display, str::FromStr};

use crate::{
    device_description::{ColorFormat, HomiePropertyDescription, HomiePropertyFormat},
    HomieDataType,
};

#[derive(Debug, PartialEq)]
pub struct Homie5ValueConversionError;

/// Represents color values supported by the Homie protocol.
///
/// The `HomieColorValue` enum encapsulates the three main color formats supported by Homie:
/// RGB, HSV, and XYZ. Each format has specific rules about how the values should be encoded
/// and transmitted as payloads in MQTT messages.
///
/// # Validity
///
/// - The encoded string must contain whole numbers for `RGB` and `HSV` formats and floating-point values
///   for `XYZ` format. These values are separated by commas, with no additional spaces allowed.
/// - Only the specific values defined in the Homie property format for a device should be accepted.
/// - An empty string or incorrectly formatted payloads (e.g., out-of-range values or invalid characters)
///   are not valid and should result in an error.
///
/// # Usage
///
/// This enum is used to represent the value of color properties in Homie-compliant devices, such as
/// lighting systems. The specific color format supported by a device is declared in the `$format`
/// attribute of the property, and the value must conform to that format.
///
/// For more details on the color formats and their constraints, refer to the Homie specification.
#[derive(Debug, Clone, Copy)]
pub enum HomieColorValue {
    /// Represents a color in the RGB format, using three integers for red, green, and blue channels.
    /// Each value must be an integer between 0 and 255.
    ///   - Example: `"rgb,255,0,0"` for red.
    RGB(i64, i64, i64),
    /// Represents a color in the HSV format, using three integers for hue, saturation, and value.
    /// Hue ranges from 0 to 360, while saturation and value range from 0 to 100.
    ///   - Example: `"hsv,120,100,100"` for bright green.
    HSV(i64, i64, i64),
    /// Represents a color in the XYZ color space, using two floating-point values for X and Y.
    /// The Z value is calculated as `1 - X - Y`, and all values range from 0.0 to 1.0.
    ///   - Example: `"xyz,0.25,0.34"`.
    XYZ(f64, f64, f64),
}

impl PartialEq for HomieColorValue {
    fn eq(&self, other: &Self) -> bool {
        const EPSILON: f64 = 1e-6; // this is already way to precise for color values

        match (self, other) {
            (HomieColorValue::RGB(r1, g1, b1), HomieColorValue::RGB(r2, g2, b2)) => r1 == r2 && g1 == g2 && b1 == b2,
            (HomieColorValue::HSV(h1, s1, v1), HomieColorValue::HSV(h2, s2, v2)) => h1 == h2 && s1 == s2 && v1 == v2,
            (HomieColorValue::XYZ(x1, y1, z1), HomieColorValue::XYZ(x2, y2, z2)) => {
                (x1 - x2).abs() < EPSILON && (y1 - y2).abs() < EPSILON && (z1 - z2).abs() < EPSILON
            }
            _ => false,
        }
    }
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

/// Represents the various data types supported by the Homie protocol.
///
/// Each variant corresponds to a specific data type allowed in the Homie MQTT convention for
/// properties. These include basic types like integers and strings, as well as more complex
/// types such as colors and JSON objects.
///
/// The Homie protocol imposes specific rules on how these types should be represented in
/// MQTT payloads, and this enum models those types.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum HomieValue {
    /// Represents an empty value, often used for uninitialized states.
    #[default]
    Empty,

    /// Represents a string value.
    ///
    /// Example: `"temperature"`, `"sensor1"`.
    ///
    /// - String payloads can be empty or contain up to 268,435,456 characters.
    String(String),

    /// Represents an integer value.
    ///
    /// Example: `21`, `-5`.
    ///
    /// - Must be a 64-bit signed integer.
    /// - Only whole numbers and the negation character `-` are allowed.
    Integer(i64),

    /// Represents a floating-point value.
    ///
    /// Example: `21.5`, `-10.25`.
    ///
    /// - Must be a 64-bit floating point number, adhering to standard scientific notation rules.
    Float(f64),

    /// Represents a boolean value.
    ///
    /// Example: `true`, `false`.
    ///
    /// - Must be either `"true"` or `"false"`. Case-sensitive.
    Bool(bool),

    /// Represents an enumerated value.
    ///
    /// Example: `"low"`, `"medium"`, `"high"`.
    ///
    /// - Enum values must match the predefined values set in the property format.
    Enum(String),

    /// Represents a color value.
    ///
    /// - Can be in `RGB`, `HSV`, or `XYZ` format, depending on the property definition.
    ///
    /// - `RGB`: 3 comma-separated values (0-255).
    /// - `HSV`: 3 comma-separated values, H (0-360), S and V (0-100).
    /// - `XYZ`: 2 comma-separated values (0-1); `Z` is calculated.
    ///
    /// Example: `"rgb,100,100,100"`, `"hsv,300,50,75"`, `"xyz,0.25,0.34"`.
    Color(HomieColorValue),

    /// Represents a datetime value.
    ///
    /// - Must adhere to ISO 8601 format.
    ///
    /// Example: `2024-10-08T10:15:30Z`.
    DateTime(chrono::DateTime<chrono::Utc>),

    /// Represents a duration value.
    ///
    /// - Must use ISO 8601 duration format (`PTxHxMxS`).
    ///
    /// Example: `"PT12H5M46S"` (12 hours, 5 minutes, 46 seconds).
    Duration(chrono::Duration),

    /// Represents a complex JSON object or array.
    ///
    /// - Must be a valid JSON array or object.
    ///
    /// Example: `{"temperature": 21.5, "humidity": 60}`.
    JSON(serde_json::Value),
}

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

impl HomieValue {
    /// Parses a raw string value into a `HomieValue` based on the provided property description.
    ///
    /// This function attempts to convert a string representation of a property value into
    /// a specific `HomieValue` type, depending on the data type and format defined in the
    /// associated `HomiePropertyDescription`. Supported data types include integers, floats,
    /// booleans, strings, enums, colors, datetime, duration, and JSON.
    ///
    /// # Arguments
    ///
    /// - `raw`: The raw string value to be parsed.
    /// - `property_desc`: A reference to the property description that defines the expected data type
    ///    and format of the property.
    ///
    /// # Returns
    ///
    /// - `Ok(HomieValue)`: If the parsing is successful and the value conforms to the expected type.
    /// - `Err(Homie5ValueConversionError)`: If parsing fails, or the value is not valid for the given type.
    ///
    /// # Errors
    ///
    /// The function returns `Err(Homie5ValueConversionError)` in the following cases:
    ///
    /// - The raw string cannot be parsed into the expected type (e.g., invalid integer or float).
    /// - The parsed value does not conform to the expected range or set of valid values.
    /// - The property format does not match the expected format for certain types, like enums or colors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use homie5::device_description::*;
    /// use homie5::{HomieValue, HomieDataType};
    ///
    /// let property_desc = PropertyDescriptionBuilder::new(HomieDataType::Integer)
    /// .format(
    ///     HomiePropertyFormat::IntegerRange(
    ///         IntegerRange { min: Some(0), max: Some(100), step: None })
    /// ).build();
    ///
    /// let value = HomieValue::parse("42", &property_desc);
    /// assert_eq!(value, Ok(HomieValue::Integer(42)));
    /// ```

    pub fn parse(
        raw: &str,
        property_desc: &HomiePropertyDescription,
    ) -> Result<HomieValue, Homie5ValueConversionError> {
        //if raw.is_empty() {
        //    return Ok(HomieValue::Empty);
        //}
        match &property_desc.datatype {
            HomieDataType::Integer => raw
                .parse::<i64>()
                .map_err(|_| Homie5ValueConversionError)
                .and_then(|d| Self::validate_int(d, property_desc))
                .map(HomieValue::Integer),
            HomieDataType::Float => raw
                .parse::<f64>()
                .map_err(|_| Homie5ValueConversionError)
                .and_then(|d| Self::validate_float(d, property_desc))
                .map(HomieValue::Float),
            HomieDataType::Boolean => raw
                .parse::<bool>()
                .map_err(|_| Homie5ValueConversionError)
                .map(HomieValue::Bool),
            HomieDataType::String => Ok(HomieValue::String(raw.to_owned())),
            HomieDataType::Enum => {
                if let HomiePropertyFormat::Enum(values) = &property_desc.format {
                    let string_val = raw.to_owned();
                    values
                        .contains(&string_val)
                        .then_some(HomieValue::Enum(string_val))
                        .ok_or(Homie5ValueConversionError)
                } else {
                    Err(Homie5ValueConversionError)
                }
            }
            HomieDataType::Color => raw
                .parse::<HomieColorValue>()
                .map(HomieValue::Color)
                .and_then(|color_value| {
                    if !property_desc.format.is_empty() {
                        if let HomiePropertyFormat::Color(formats) = &property_desc.format {
                            match color_value {
                                HomieValue::Color(HomieColorValue::RGB(_, _, _))
                                    if formats.contains(&ColorFormat::Rgb) =>
                                {
                                    Ok(color_value)
                                }
                                HomieValue::Color(HomieColorValue::HSV(_, _, _))
                                    if formats.contains(&ColorFormat::Hsv) =>
                                {
                                    Ok(color_value)
                                }
                                HomieValue::Color(HomieColorValue::XYZ(_, _, _))
                                    if formats.contains(&ColorFormat::Xyz) =>
                                {
                                    Ok(color_value)
                                }
                                _ => Err(Homie5ValueConversionError),
                            }
                        } else {
                            Err(Homie5ValueConversionError)
                        }
                    } else {
                        Ok(color_value)
                    }
                }),
            HomieDataType::Datetime => Self::flexible_datetime_parser(raw).map(HomieValue::DateTime),
            HomieDataType::Duration => Self::parse_duration(raw).map(HomieValue::Duration),
            HomieDataType::JSON => serde_json::from_str::<serde_json::Value>(raw)
                .map(HomieValue::JSON)
                .map_err(|_| Homie5ValueConversionError),
        }
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

    fn validate_float(value: f64, property_desc: &HomiePropertyDescription) -> Result<f64, Homie5ValueConversionError> {
        let HomiePropertyFormat::FloatRange(fr) = &property_desc.format else {
            return Ok(value);
        };
        // Use the minimum, max, or current value as base (in that priority order)
        let base = fr.min.or(fr.max).unwrap_or(value);

        // Calculate the rounded value based on the step
        let rounded = match fr.step {
            Some(s) if s > 0.0 => ((value - base) / s).round() * s + base,
            _ => value,
        };

        // Check if the rounded value is within the min/max bounds
        if fr.min.map_or(true, |m| rounded >= m) && fr.max.map_or(true, |m| rounded <= m) {
            Ok(rounded)
        } else {
            Err(Homie5ValueConversionError)
        }
    }

    fn validate_int(value: i64, property_desc: &HomiePropertyDescription) -> Result<i64, Homie5ValueConversionError> {
        let HomiePropertyFormat::IntegerRange(ir) = &property_desc.format else {
            return Ok(value);
        };

        // Use the minimum or maximum as the base, or use the current value
        let base = ir.min.or(ir.max).unwrap_or(value);

        // Calculate the rounded value based on the step
        let rounded = match ir.step {
            Some(s) if s > 0 => ((value - base) as f64 / s as f64).round() as i64 * s + base,
            _ => value,
        };

        // Check if the rounded value is within the min/max bounds
        if ir.min.map_or(true, |m| rounded >= m) && ir.max.map_or(true, |m| rounded <= m) {
            Ok(rounded)
        } else {
            Err(Homie5ValueConversionError)
        }
    }
}
