//! Provides all types and functions for parsing and creating homie property values
//!
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    str::FromStr,
};

use serde::{de, Serialize, Serializer};
use serde::{Deserialize, Deserializer};

use crate::{
    device_description::{ColorFormat, FloatRange, HomiePropertyDescription, HomiePropertyFormat, IntegerRange},
    Homie5ProtocolError, HomieDataType,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Homie5ValueConversionError {
    InvalidColorFormat {
        value: String,
        reason: &'static str,
    },
    InvalidColorValue {
        value: HomieColorValue,
        reason: &'static str,
    },
    InvalidIntegerFormat(String),
    IntegerOutOfRange(i64, IntegerRange),
    InvalidFloatFormat(String),
    FloatOutOfRange(f64, FloatRange),
    InvalidEnumFormat(String, Vec<String>),
    InvalidDateTimeFormat(String),
    InvalidDurationFormat(String),
    UnsupportedColorFormat(ColorFormat, Vec<ColorFormat>),
    InvalidBooleanFormat(String),
    JsonParseError(String),
}

impl Homie5ValueConversionError {
    fn invalid_color_format(value: impl Into<String>, reason: &'static str) -> Self {
        Self::InvalidColorFormat {
            value: value.into(),
            reason,
        }
    }
}

impl fmt::Display for Homie5ValueConversionError {
    /// Formats the error message for display purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Homie5ValueConversionError::InvalidColorFormat { value, reason } => {
                write!(f, "'{}' is not a valid color format: {}", value, reason)
            }
            Homie5ValueConversionError::InvalidIntegerFormat(value) => {
                write!(f, "'{}' is not a valid integer value", value)
            }
            Homie5ValueConversionError::InvalidFloatFormat(value) => {
                write!(f, "'{}' is not a valid float value", value)
            }
            Homie5ValueConversionError::InvalidEnumFormat(value, allowed_values) => {
                write!(
                    f,
                    "'{}' is not allowed enum values: {}",
                    value,
                    allowed_values.join(",")
                )
            }
            Homie5ValueConversionError::IntegerOutOfRange(value, range) => {
                write!(f, "Integer '{}' is out of allowed range: {}", value, range)
            }
            Homie5ValueConversionError::FloatOutOfRange(value, range) => {
                write!(f, "Float '{}' is out of allowed range: {}", value, range)
            }
            Homie5ValueConversionError::InvalidColorValue { value, reason } => {
                write!(f, "'{:?}' is not a valid color value: {}", value, reason)
            }
            Homie5ValueConversionError::InvalidDateTimeFormat(value) => {
                write!(f, "'{}' is not a valid date/time value", value)
            }
            Homie5ValueConversionError::InvalidDurationFormat(value) => {
                write!(f, "'{}' is not a valid duration value", value)
            }
            Homie5ValueConversionError::UnsupportedColorFormat(color_format, formats) => {
                write!(
                    f,
                    "'{}' is not in supported formats: {:?}",
                    color_format,
                    formats.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(",")
                )
            }
            Homie5ValueConversionError::InvalidBooleanFormat(value) => {
                write!(f, "'{}' is not a valid boolean value", value)
            }
            Homie5ValueConversionError::JsonParseError(error) => {
                write!(f, "Error parsing json value: {}", error)
            }
        }
    }
}

// Implement the std::error::Error trait
impl std::error::Error for Homie5ValueConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // This error type doesn't wrap any other errors
        None
    }
}

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
    /// Represents Homie's historical `xyz` color format, using two CIE 1931 chromaticity
    /// coordinates. The third coordinate is calculated as `z = 1 - x - y`, and all values
    /// range from 0.0 to 1.0.
    ///   - Example: `"xyz,0.25,0.34"`.
    XYZ(f64, f64, f64),
}

impl HomieColorValue {
    const RGB_RULES: &'static str =
        "RGB requires three whole numbers: red, green, and blue must each be between 0 and 255";
    const HSV_RULES: &'static str =
        "HSV requires three whole numbers: hue must be between 0 and 360, saturation and value between 0 and 100";
    const XYZ_RULES: &'static str =
        "XYZ requires numbers between 0.0 and 1.0; x + y must be at most 1.0, and z must equal 1.0 - x - y";
    const COLOR_RULES: &'static str = "expected color format: rgb,R,G,B, hsv,H,S,V, or xyz,X,Y";

    pub fn color_format(&self) -> ColorFormat {
        match self {
            HomieColorValue::RGB(_, _, _) => ColorFormat::Rgb,
            HomieColorValue::HSV(_, _, _) => ColorFormat::Hsv,
            HomieColorValue::XYZ(_, _, _) => ColorFormat::Xyz,
        }
    }
}

/// Serialize the ColorValue to its string representation
///
///   - Example: `"rgb,255,0,0"` for red.
///   - Example: `"hsv,120,100,100"` for bright green.
///   - Example: `"xyz,0.25,0.34"`.
impl Serialize for HomieColorValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize the enum as its string representation.
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserialize the ColorValue from its string representation
///
///   - Example: `"rgb,255,0,0"` for red.
///   - Example: `"hsv,120,100,100"` for bright green.
///   - Example: `"xyz,0.25,0.34"`.
impl<'de> Deserialize<'de> for HomieColorValue {
    fn deserialize<D>(deserializer: D) -> Result<HomieColorValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a string first.
        let s = String::deserialize(deserializer)?;
        // Use the FromStr implementation to parse the string.
        HomieColorValue::from_str(&s).map_err(serde::de::Error::custom)
    }
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

impl PartialOrd<HomieColorValue> for HomieColorValue {
    fn partial_cmp(&self, other: &HomieColorValue) -> Option<Ordering> {
        if self.eq(other) {
            Some(Ordering::Equal)
        } else {
            None
        }
    }
}

impl HomieColorValue {
    /// Constructs a Homie `xyz` color by deriving the internal `z` chromaticity coordinate.
    ///
    /// This is a convenience constructor for trusted coordinates and intentionally does not
    /// validate. Use [`Self::validate`] before publishing values derived from external sources.
    pub fn new_xyz(x: f64, y: f64) -> Self {
        HomieColorValue::XYZ(x, y, 1.0 - x - y)
    }

    /// Validates that the color components satisfy the intrinsic Homie color constraints.
    ///
    /// This only checks the value itself. Whether the color format is allowed for a specific
    /// property is still checked against the property description by [`HomieValue::validate`].
    pub fn validate(&self) -> Result<(), Homie5ValueConversionError> {
        const XYZ_EPSILON: f64 = 1e-6;
        let invalid = |reason| Homie5ValueConversionError::InvalidColorValue { value: *self, reason };

        match self {
            HomieColorValue::RGB(r, g, b) => {
                if (0..=255).contains(r) && (0..=255).contains(g) && (0..=255).contains(b) {
                    Ok(())
                } else {
                    Err(invalid(Self::RGB_RULES))
                }
            }
            HomieColorValue::HSV(h, s, v) => {
                if (0..=360).contains(h) && (0..=100).contains(s) && (0..=100).contains(v) {
                    Ok(())
                } else {
                    Err(invalid(Self::HSV_RULES))
                }
            }
            HomieColorValue::XYZ(x, y, z) => {
                let expected_z = 1.0 - x - y;
                if x.is_finite()
                    && y.is_finite()
                    && z.is_finite()
                    && (0.0..=1.0).contains(x)
                    && (0.0..=1.0).contains(y)
                    && (0.0..=1.0).contains(z)
                    && x + y <= 1.0
                    && (z - expected_z).abs() <= XYZ_EPSILON
                {
                    Ok(())
                } else {
                    Err(invalid(Self::XYZ_RULES))
                }
            }
        }
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
        let mut tokens = s.split(',');
        let invalid = |reason| Homie5ValueConversionError::invalid_color_format(s, reason);

        match tokens.next() {
            Some("rgb") => {
                let (Some(r), Some(g), Some(b), None) = (tokens.next(), tokens.next(), tokens.next(), tokens.next())
                else {
                    return Err(invalid(Self::RGB_RULES));
                };

                let color = Self::RGB(
                    r.parse().map_err(|_| invalid(Self::RGB_RULES))?,
                    g.parse().map_err(|_| invalid(Self::RGB_RULES))?,
                    b.parse().map_err(|_| invalid(Self::RGB_RULES))?,
                );
                color.validate()?;
                Ok(color)
            }
            Some("hsv") => {
                let (Some(h), Some(saturation), Some(value), None) =
                    (tokens.next(), tokens.next(), tokens.next(), tokens.next())
                else {
                    return Err(invalid(Self::HSV_RULES));
                };

                let color = Self::HSV(
                    h.parse().map_err(|_| invalid(Self::HSV_RULES))?,
                    saturation.parse().map_err(|_| invalid(Self::HSV_RULES))?,
                    value.parse().map_err(|_| invalid(Self::HSV_RULES))?,
                );
                color.validate()?;
                Ok(color)
            }
            Some("xyz") => {
                let (Some(x), Some(y), None) = (tokens.next(), tokens.next(), tokens.next()) else {
                    return Err(invalid(Self::XYZ_RULES));
                };

                let color = Self::new_xyz(
                    x.parse().map_err(|_| invalid(Self::XYZ_RULES))?,
                    y.parse().map_err(|_| invalid(Self::XYZ_RULES))?,
                );
                color.validate()?;
                Ok(color)
            }
            _ => Err(invalid(Self::COLOR_RULES)),
        }
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
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
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
    #[serde(serialize_with = "serialize_duration")]
    Duration(chrono::Duration),

    /// Represents a complex JSON object or array.
    ///
    /// - Must be a valid JSON array or object.
    ///
    /// Example: `{"temperature": 21.5, "humidity": 60}`.
    JSON(serde_json::Value),
}

fn serialize_duration<S>(duration: &chrono::Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let iso_str = duration.to_string();
    serializer.serialize_str(&iso_str)
}

/// All `HomieValue` variant names, used for serde error reporting.
const HOMIE_VALUE_VARIANTS: &[&str] = &[
    "Empty", "String", "Integer", "Float", "Bool", "Enum", "Color", "DateTime", "Duration", "JSON",
];

/// Custom `Deserialize` implementation for [`HomieValue`].
///
/// A derived (externally tagged) implementation behaves inconsistently across serde's two
/// deserialization paths in YAML:
///
/// - On the *direct* path, `serde_yaml`-family crates require the YAML tag form (`!Bool true`)
///   for externally tagged enums and reject the single-key map form (`{ Bool: true }`).
/// - On the *buffered* path (inside `#[serde(untagged)]`, `#[serde(tag = "...")]` or
///   `#[serde(flatten)]` containers), only the single-key map form works, because serde's
///   internal `Content` buffer cannot hold enum values.
///
/// This implementation accepts **both** representations wherever the format provides them, so
/// the documented map form (which is also the JSON representation) parses in every context:
///
/// - the single-key map form: `{ Bool: true }`, `{ Integer: 5 }`, `{ Empty: null }`, ...
/// - the enum/tag form where the format supports it: `!Bool true` (YAML direct path)
/// - the bare string `"Empty"` for the unit variant
///
/// `DateTime` and `Duration` payloads are parsed from owned strings via
/// [`HomieValue::flexible_datetime_parser`] and [`HomieValue::parse_duration`], which also
/// avoids "expected borrowed string" failures inside buffered contexts.
impl<'de> Deserialize<'de> for HomieValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct HomieValueVisitor;

        impl<'de> de::Visitor<'de> for HomieValueVisitor {
            type Value = HomieValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a HomieValue: a single-key map like { Bool: true }, an externally tagged enum, or the string \"Empty\"",
                )
            }

            // Unit variant as bare string: "Empty"
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v == "Empty" {
                    Ok(HomieValue::Empty)
                } else {
                    Err(de::Error::unknown_variant(v, HOMIE_VALUE_VARIANTS))
                }
            }

            // Single-key map form: { Bool: true } — works on both direct and buffered paths.
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let Some(variant) = map.next_key::<String>()? else {
                    return Err(de::Error::invalid_length(
                        0,
                        &"a map with exactly one HomieValue variant key",
                    ));
                };
                let value = match variant.as_str() {
                    "Empty" => {
                        map.next_value::<de::IgnoredAny>()?;
                        HomieValue::Empty
                    }
                    "String" => HomieValue::String(map.next_value()?),
                    "Integer" => HomieValue::Integer(map.next_value()?),
                    "Float" => HomieValue::Float(map.next_value()?),
                    "Bool" => HomieValue::Bool(map.next_value()?),
                    "Enum" => HomieValue::Enum(map.next_value()?),
                    "Color" => HomieValue::Color(map.next_value()?),
                    "DateTime" => {
                        let s: String = map.next_value()?;
                        HomieValue::DateTime(HomieValue::flexible_datetime_parser(&s).map_err(de::Error::custom)?)
                    }
                    "Duration" => {
                        let s: String = map.next_value()?;
                        HomieValue::Duration(HomieValue::parse_duration(&s).map_err(de::Error::custom)?)
                    }
                    "JSON" => HomieValue::JSON(map.next_value()?),
                    other => return Err(de::Error::unknown_variant(other, HOMIE_VALUE_VARIANTS)),
                };
                if map.next_key::<de::IgnoredAny>()?.is_some() {
                    return Err(de::Error::custom(
                        "expected a map with exactly one HomieValue variant key",
                    ));
                }
                Ok(value)
            }

            // Enum/tag form: `!Bool true` on the serde_yaml direct path.
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                use serde::de::VariantAccess;

                let (variant, access) = data.variant::<String>()?;
                match variant.as_str() {
                    "Empty" => {
                        access.unit_variant()?;
                        Ok(HomieValue::Empty)
                    }
                    "String" => Ok(HomieValue::String(access.newtype_variant()?)),
                    "Integer" => Ok(HomieValue::Integer(access.newtype_variant()?)),
                    "Float" => Ok(HomieValue::Float(access.newtype_variant()?)),
                    "Bool" => Ok(HomieValue::Bool(access.newtype_variant()?)),
                    "Enum" => Ok(HomieValue::Enum(access.newtype_variant()?)),
                    "Color" => Ok(HomieValue::Color(access.newtype_variant()?)),
                    "DateTime" => {
                        let s: String = access.newtype_variant()?;
                        Ok(HomieValue::DateTime(
                            HomieValue::flexible_datetime_parser(&s).map_err(de::Error::custom)?,
                        ))
                    }
                    "Duration" => {
                        let s: String = access.newtype_variant()?;
                        Ok(HomieValue::Duration(
                            HomieValue::parse_duration(&s).map_err(de::Error::custom)?,
                        ))
                    }
                    "JSON" => Ok(HomieValue::JSON(access.newtype_variant()?)),
                    other => Err(de::Error::unknown_variant(other, HOMIE_VALUE_VARIANTS)),
                }
            }
        }

        deserializer.deserialize_any(HomieValueVisitor)
    }
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
impl From<i64> for HomieValue {
    fn from(value: i64) -> Self {
        HomieValue::Integer(value)
    }
}
impl From<f64> for HomieValue {
    fn from(value: f64) -> Self {
        HomieValue::Float(value)
    }
}
impl From<String> for HomieValue {
    fn from(value: String) -> Self {
        HomieValue::String(value)
    }
}
impl From<bool> for HomieValue {
    fn from(value: bool) -> Self {
        HomieValue::Bool(value)
    }
}
impl From<HomieColorValue> for HomieValue {
    fn from(value: HomieColorValue) -> Self {
        HomieValue::Color(value)
    }
}
impl From<chrono::DateTime<chrono::Utc>> for HomieValue {
    fn from(value: chrono::DateTime<chrono::Utc>) -> Self {
        HomieValue::DateTime(value)
    }
}
impl From<chrono::Duration> for HomieValue {
    fn from(value: chrono::Duration) -> Self {
        HomieValue::Duration(value)
    }
}
impl From<serde_json::Value> for HomieValue {
    fn from(value: serde_json::Value) -> Self {
        HomieValue::JSON(value)
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

impl From<HomieValue> for Vec<u8> {
    fn from(value: HomieValue) -> Self {
        homie_str_to_vecu8(value.to_string())
    }
}

impl From<&HomieValue> for Vec<u8> {
    fn from(value: &HomieValue) -> Self {
        homie_str_to_vecu8(value.to_string())
    }
}

pub fn homie_str_to_vecu8(value: impl Into<String>) -> Vec<u8> {
    let value_string = value.into();
    // empty strings are published as a String with a 0 byte value as first character according
    // to homie convention
    if value_string.is_empty() {
        vec![0_u8]
    } else {
        value_string.into_bytes()
    }
}

impl PartialOrd<HomieValue> for HomieValue {
    fn partial_cmp(&self, other: &HomieValue) -> Option<Ordering> {
        match (self, other) {
            (HomieValue::Empty, HomieValue::Empty) => Some(Ordering::Equal),
            (HomieValue::Empty, _) => Some(Ordering::Less),
            (_, HomieValue::Empty) => Some(Ordering::Greater),
            (HomieValue::String(self_str), HomieValue::String(other_str)) => self_str.partial_cmp(other_str),
            (HomieValue::Integer(self_int), HomieValue::Integer(other_int)) => self_int.partial_cmp(other_int),
            (HomieValue::Float(self_float), HomieValue::Float(other_float)) => self_float.partial_cmp(other_float),
            (HomieValue::Bool(self_bool), HomieValue::Bool(other_bool)) => self_bool.partial_cmp(other_bool),
            (HomieValue::Enum(self_enum), HomieValue::Enum(other_enum)) => self_enum.partial_cmp(other_enum),
            (HomieValue::Enum(self_enum), HomieValue::String(other_string)) => self_enum.partial_cmp(other_string),
            (HomieValue::String(self_string), HomieValue::Enum(other_enum)) => self_string.partial_cmp(other_enum),
            (HomieValue::Color(self_homie_color_value), HomieValue::Color(other_homie_color_value)) => {
                self_homie_color_value.partial_cmp(other_homie_color_value)
            }
            (HomieValue::DateTime(self_date_time), HomieValue::DateTime(other_date_time)) => {
                self_date_time.partial_cmp(other_date_time)
            }
            (HomieValue::Duration(self_time_delta), HomieValue::Duration(other_time_delte)) => {
                self_time_delta.partial_cmp(other_time_delte)
            }
            (HomieValue::JSON(self_value), HomieValue::JSON(other_value)) => {
                self_value.to_string().partial_cmp(&other_value.to_string())
            }
            _ => None,
        }
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
    ///   and format of the property.
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
    /// let property_desc = PropertyDescriptionBuilder::integer()
    ///     .integer_range(IntegerRange { min: Some(0), max: Some(100), step: None })
    ///     .build();
    ///
    /// let value = HomieValue::parse("42", &property_desc);
    /// assert_eq!(value.ok(), Some(HomieValue::Integer(42)));
    /// ```
    pub fn parse(raw: &str, property_desc: &HomiePropertyDescription) -> Result<HomieValue, Homie5ProtocolError> {
        //if raw
        //    .first()
        //    .map(|first| matches!(property_desc.datatype, HomieDataType::String) && *first == 0)
        //    == Some(true)
        //{
        //    return Ok(HomieValue::Empty);
        //}
        match &property_desc.datatype {
            HomieDataType::Integer => raw
                .parse::<i64>()
                .map_err(|_| Homie5ValueConversionError::InvalidIntegerFormat(raw.to_string()))
                .and_then(|d| Self::validate_int(d, property_desc))
                .map(HomieValue::Integer),
            HomieDataType::Float => raw
                .parse::<f64>()
                .map_err(|_| Homie5ValueConversionError::InvalidFloatFormat(raw.to_string()))
                .and_then(|d| Self::validate_float(d, property_desc))
                .map(HomieValue::Float),
            HomieDataType::Boolean => raw
                .parse::<bool>()
                .map_err(|_| Homie5ValueConversionError::InvalidBooleanFormat(raw.to_string()))
                .map(HomieValue::Bool),
            HomieDataType::String => Ok(HomieValue::String(raw.to_owned())),
            HomieDataType::Enum => {
                if let HomiePropertyFormat::Enum(values) = &property_desc.format {
                    if values.is_empty() || values.iter().any(|v| v.is_empty()) {
                        return Err(
                            Homie5ValueConversionError::InvalidEnumFormat(raw.to_string(), values.clone()).into(),
                        );
                    }

                    let string_val = raw.to_owned();
                    values
                        .contains(&string_val)
                        .then_some(HomieValue::Enum(string_val))
                        .ok_or(Homie5ValueConversionError::InvalidEnumFormat(
                            raw.to_string(),
                            values.clone(),
                        ))
                } else {
                    Err(Homie5ValueConversionError::InvalidEnumFormat(raw.to_string(), vec![]))
                }
            }
            HomieDataType::Color => raw
                .parse::<HomieColorValue>()
                .and_then(|color_value| {
                    if let HomiePropertyFormat::Color(formats) = &property_desc.format {
                        if formats.is_empty() {
                            return Err(Homie5ValueConversionError::UnsupportedColorFormat(
                                color_value.color_format(),
                                vec![],
                            ));
                        }

                        match color_value {
                            HomieColorValue::RGB(_, _, _) if formats.contains(&ColorFormat::Rgb) => Ok(color_value),
                            HomieColorValue::HSV(_, _, _) if formats.contains(&ColorFormat::Hsv) => Ok(color_value),
                            HomieColorValue::XYZ(_, _, _) if formats.contains(&ColorFormat::Xyz) => Ok(color_value),
                            color => Err(Homie5ValueConversionError::UnsupportedColorFormat(
                                color.color_format(),
                                formats.clone(),
                            )),
                        }
                    } else {
                        Err(Homie5ValueConversionError::UnsupportedColorFormat(
                            color_value.color_format(),
                            vec![],
                        ))
                    }
                })
                .map(HomieValue::Color),
            HomieDataType::Datetime => Self::flexible_datetime_parser(raw).map(HomieValue::DateTime),
            HomieDataType::Duration => Self::parse_duration(raw).map(HomieValue::Duration),
            HomieDataType::JSON => serde_json::from_str::<serde_json::Value>(raw)
                .map_err(|e| Homie5ValueConversionError::JsonParseError(e.to_string()))
                .and_then(|v| {
                    if v.is_object() || v.is_array() {
                        Ok(HomieValue::JSON(v))
                    } else {
                        Err(Homie5ValueConversionError::JsonParseError(
                            "JSON payload must be an object or array".to_string(),
                        ))
                    }
                }),
        }
        .map_err(Homie5ProtocolError::InvalidHomieValue)
    }

    pub fn parse_duration(s: &str) -> Result<chrono::Duration, Homie5ValueConversionError> {
        static RE: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new(r"^PT(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?$").unwrap());
        if let Some(captures) = RE.captures(s) {
            let hours: i64 = captures.get(1).map_or(0, |m| m.as_str().parse().unwrap());
            let minutes: i64 = captures.get(2).map_or(0, |m| m.as_str().parse().unwrap());
            let seconds: i64 = captures.get(3).map_or(0, |m| m.as_str().parse().unwrap());

            // Require at least one component (reject bare "PT")
            if captures.get(1).is_none() && captures.get(2).is_none() && captures.get(3).is_none() {
                return Err(Homie5ValueConversionError::InvalidDurationFormat(s.to_string()));
            }

            return Ok(chrono::Duration::seconds(hours * 3600 + minutes * 60 + seconds));
        }
        Err(Homie5ValueConversionError::InvalidDurationFormat(s.to_string()))
    }

    // flexible deserialization approach as timestamps are hard and we want to keep compatibility
    // high
    pub fn flexible_datetime_parser(s: &str) -> Result<chrono::DateTime<chrono::Utc>, Homie5ValueConversionError> {
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
                            |_| Err(Homie5ValueConversionError::InvalidDateTimeFormat(s.to_string())), // if this also does not work, we give
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
        let HomiePropertyFormat::FloatRange(range) = &property_desc.format else {
            return Ok(value);
        };

        // Use min as base, if not present, use max, otherwise use the current value
        let base = range.min.or(range.max).unwrap_or(value);

        // Adjust rounding logic: floor((x + 0.5)) to always round up
        let rounded = match range.step {
            Some(s) if s > 0.0 => ((value - base) / s + 0.5).floor() * s + base,
            _ => value,
        };

        // Check if the rounded value is within the min/max bounds
        if range.min.is_none_or(|m| rounded >= m) && range.max.is_none_or(|m| rounded <= m) {
            Ok(rounded)
        } else {
            Err(Homie5ValueConversionError::FloatOutOfRange(value, range.clone()))
        }
    }

    fn validate_int(value: i64, property_desc: &HomiePropertyDescription) -> Result<i64, Homie5ValueConversionError> {
        let HomiePropertyFormat::IntegerRange(range) = &property_desc.format else {
            return Ok(value);
        };

        // Use min as base, if not present, use max, otherwise use the current value
        let base = range.min.or(range.max).unwrap_or(value);

        // Adjust rounding logic: floor((x + 0.5)) to always round up
        let rounded = match range.step {
            Some(s) if s > 0 => ((value - base) as f64 / s as f64 + 0.5).floor() as i64 * s + base,
            _ => value,
        };

        // Check if the rounded value is within the min/max bounds
        if range.min.is_none_or(|m| rounded >= m) && range.max.is_none_or(|m| rounded <= m) {
            Ok(rounded)
        } else {
            Err(Homie5ValueConversionError::IntegerOutOfRange(value, range.clone()))
        }
    }

    pub fn validate(&self, property_desc: &HomiePropertyDescription) -> bool {
        match (self, property_desc.datatype) {
            (HomieValue::Empty, HomieDataType::String) => true,
            (HomieValue::String(_), HomieDataType::String) => true,
            (HomieValue::Integer(value), HomieDataType::Integer) => Self::validate_int(*value, property_desc)
                .map(|v| v == *value)
                .unwrap_or(false),
            (HomieValue::Float(value), HomieDataType::Float) => {
                value.is_finite()
                    && Self::validate_float(*value, property_desc)
                        .map(|v| v == *value)
                        .unwrap_or(false)
            }
            (HomieValue::Bool(_), HomieDataType::Boolean) => true,
            (HomieValue::Enum(value), HomieDataType::Enum) => {
                let HomiePropertyFormat::Enum(variants) = &property_desc.format else {
                    return false;
                };
                !variants.is_empty() && variants.iter().all(|v| !v.is_empty()) && variants.contains(value)
            }
            (HomieValue::Color(value), HomieDataType::Color) => {
                let HomiePropertyFormat::Color(color_formats) = &property_desc.format else {
                    return false;
                };
                if color_formats.is_empty() {
                    return false;
                }
                if value.validate().is_err() {
                    return false;
                }
                match value {
                    HomieColorValue::RGB(_, _, _) => color_formats.contains(&ColorFormat::Rgb),
                    HomieColorValue::HSV(_, _, _) => color_formats.contains(&ColorFormat::Hsv),
                    HomieColorValue::XYZ(_, _, _) => color_formats.contains(&ColorFormat::Xyz),
                }
            }
            (HomieValue::DateTime(_), HomieDataType::Datetime) => true,
            (HomieValue::Duration(_), HomieDataType::Duration) => true,
            (HomieValue::JSON(v), HomieDataType::JSON) => v.is_object() || v.is_array(),
            _ => false,
        }
    }

    pub fn datatype(&self) -> HomieDataType {
        match self {
            HomieValue::Empty => HomieDataType::String,
            HomieValue::String(_) => HomieDataType::String,
            HomieValue::Integer(_) => HomieDataType::Integer,
            HomieValue::Float(_) => HomieDataType::Float,
            HomieValue::Bool(_) => HomieDataType::Boolean,
            HomieValue::Enum(_) => HomieDataType::Enum,
            HomieValue::Color(_) => HomieDataType::Color,
            HomieValue::DateTime(_) => HomieDataType::Datetime,
            HomieValue::Duration(_) => HomieDataType::Duration,
            HomieValue::JSON(_) => HomieDataType::JSON,
        }
    }

    pub fn matches(&self, datatype: HomieDataType) -> bool {
        self.datatype() == datatype
    }
}
