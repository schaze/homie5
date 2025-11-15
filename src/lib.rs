//! this is a very low level implemenation of the homie5 protocol in rust.
//! it aims to be as flexible and unopinionated as possible. there is no direct dependency to a mqtt library.
//! homie5 provides a basic support for a protocol implementation for homie v5 with a clearly defined interface
//! point to a mqtt library. the library provides fully typed support for all homie5 datatypes.
//!
//! due to this, the usage of the library is a bit more involved as with a completly ready to use homie library.
//! benefit is however that you can use the library basically everywhere from a simple esp32, raspberrypi to a x86 machine.
//!
//! you can find working examples for both device and controller use case in the examples/ folder:
//!
//!    - controller_example.rs:
//!      implements a homie5 controller that will discover all homie5 devices on a
//!      mqtt broker and print out the devices and their property updates (more information).
//!    - device_example.rs:
//!      implements a simple lightdevice with state and brightness control properties (more information).
//!
//! both examples use rumqttc as a mqtt client implementation and provide a best practice in homie5
//! usage and in how to integrate the 2 libraries.
//!

#![cfg_attr(not(feature = "std"), feature(core_float_math))]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod client;
mod controller_proto;
pub mod device_description;
mod device_proto;
mod error;
pub mod extensions;
mod homie5_message;
mod homie_domain;
mod homie_id;
mod homie_ref;
mod statemachine;
mod value;

pub use controller_proto::*;
pub use device_proto::*;
pub use error::Homie5ProtocolError;
pub use homie5_message::*;
pub use homie_domain::*;
pub use homie_id::*;
pub use homie_ref::*;
pub use value::*;

use serde::{Deserialize, Serialize};

use core::fmt;
use core::fmt::{Debug, Display};
use core::str::FromStr;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
};

// https://github.com/rust-lang/rust/issues/137578
#[cfg(not(feature = "std"))]
#[allow(dead_code)]
trait CoreFloatMath {
    fn floor(self: Self) -> Self;
}

#[cfg(not(feature = "std"))]
impl CoreFloatMath for f64 {
    fn floor(self: f64) -> f64 {
        core::f64::math::floor(self)
    }
}

/// The default mqtt root topic: "homie"
pub const DEFAULT_HOMIE_DOMAIN: &str = "homie";
/// Homie major version used in the mqtt topic creation: "5"
pub const HOMIE_VERSION: &str = "5";
/// Homie protocol verison used in the device description: "5.0"
pub const HOMIE_VERSION_FULL: &str = "5.0";
/// Broadcast topic: "$broadcast"
pub const HOMIE_TOPIC_BROADCAST: &str = "$broadcast";

/// Device state attribute topic: "$state"
pub const DEVICE_ATTRIBUTE_STATE: &str = "$state";
/// Device log attribute topic: "$log"
pub const DEVICE_ATTRIBUTE_LOG: &str = "$log";
/// Device description attribute topic: "$description"
pub const DEVICE_ATTRIBUTE_DESCRIPTION: &str = "$description";
/// Device alert attribute topic: "$state"
pub const DEVICE_ATTRIBUTE_ALERT: &str = "$alert";
/// A list of all the device attributes to be published or subscribed to
pub const DEVICE_ATTRIBUTES: [&str; 4] = [
    DEVICE_ATTRIBUTE_STATE, // state MUST be first in this array due to use in device removal
    DEVICE_ATTRIBUTE_LOG,
    DEVICE_ATTRIBUTE_ALERT,
    DEVICE_ATTRIBUTE_DESCRIPTION,
];

/// Property set attribute topic under which a set action is published to alter the devices state: "set"
pub const PROPERTY_SET_TOPIC: &str = "set";
/// Property $target attribute topic under which the device can publish the desired target state
pub const PROPERTY_ATTRIBUTE_TARGET: &str = "$target";

/// Datatypes in the homie protocol
#[derive(Serialize, Deserialize, Default, Clone, Hash, PartialEq, Eq, PartialOrd, Copy)]
#[serde(rename_all = "lowercase")]
pub enum HomieDataType {
    /// - Integer types are string literal representations of 64-bit signed whole numbers
    /// - Integers range from -9,223,372,036,854,775,808 (-263) to 9,223,372,036,854,775,807 (263-1)
    /// - The payload may only contain whole numbers and the negation character “-”. No other characters including spaces (" “) are permitted
    /// - A string with just a negation sign (”-") is not a valid payload
    /// - An empty string ("") is not a valid payload
    #[default]
    Integer,

    /// - Float types are string literal representations of 64-bit signed floating point numbers
    /// - Floats range from +/-(2^-1074) to +/-((2 - 2^-52) * 2^1023)
    /// - The payload may only contain whole numbers, the negation character “-”, the exponent character “e” or “E” and the decimal separator “.”, no other characters, including spaces (" “) are permitted
    /// - The dot character (”.") is the decimal separator (used if necessary) and may only have a single instance present in the payload
    /// - Representations of numeric concepts such as “NaN” (Not a Number) and “Infinity” are not a valid payload
    /// - A string with just a negation sign ("-") is not a valid payload
    /// - An empty string ("") is not a valid payload
    Float,

    /// - Booleans must be converted to the string literals “true” or “false”
    /// - Representation is case sensitive, e.g. “TRUE” or “FALSE” are not valid payloads.
    /// - An empty string ("") is not a valid payload
    Boolean,

    /// - String types are limited to 268,435,456 characters
    /// - An empty string ("") is a valid payloads
    String,

    /// - Enum payloads must be one of the values specified in the format definition of the property
    /// - Enum payloads are case sensitive, e.g. “Car” will not match a format definition of “car”
    /// - Leading- and trailing-whitespace is significant, e.g. “Car” will not match " Car".
    /// - An empty string ("") is not a valid payload
    Enum,

    /// - Color payload validity varies depending on the property format definition of either “rgb”, “hsv”, or “xyz”
    /// - All payload types contain comma-separated data of differing restricted ranges. The first being the type, followed by numbers. The numbers must conform to the float format
    /// - The encoded string may only contain the type, the float numbers and the comma character “,”, no other characters are permitted, including spaces (" “)
    /// - Payloads for type “rgb” contain 3 comma-separated values of floats (r, g, b) with a valid range between 0 and 255 (inclusive). e.g. "rgb,100,100,100"
    /// - Payloads for type “hsv” contain 3 comma-separated values of floats. The first number (h) has a range of 0 to 360 (inclusive), and the second and third numbers (s and v) have a range of 0 to 100 (inclusive). e.g. "hsv,300,50,75"
    /// - Payloads for type “xyz” contain 2 comma separated values of floats (x, y) with a valid range between 0 and 1 (inclusive). The “z” value can be calculated via z=1-x-y and is therefore not transmitted. (see CIE_1931_color_space). e.g. "xyz,0.25,0.34"
    /// - Note: The rgb and hsv formats encode both color and brightness, whereas xyz only encodes the color, so;
    ///    - when brightness encoding is required: do not use xyz, or optionally add another property for the brightness (such that setting hsv and rgb values changes both the color property and the brightness one if required)
    ///    - if color only is encoded: ignore the v value in hsv, and use the relative colors of rgb eg. color_only_r = 255 * r / max(r, g, b), etc.
    /// - An empty string (”") is not a valid payload
    Color,

    /// - DateTime payloads must use the ISO 8601 format.
    /// - An empty string ("") is not a valid payload
    Datetime,

    /// - Duration payloads must use the ISO 8601 duration format
    /// - The format is PTxHxMxS, where: P: Indicates a period/duration (required). T: Indicates a time (required). xH: Hours, where x represents the number of hours (optional). xM: Minutes, where x represents the number of minutes (optional). xS: Seconds, where x represents the number of seconds (optional).
    /// - Examples: PT12H5M46S (12 hours, 5 minutes, 46 seconds), PT5M (5 minutes)
    /// - An empty string ("") is not a valid payload
    Duration,

    /// - Contains a JSON string for transporting complex data formats that cannot be exposed as single value attributes.
    /// - The payload MUST be either a JSON-Array or JSON-Object type, for other types the standard Homie types should be used.
    JSON,
}

impl Debug for HomieDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for HomieDataType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HomieDataType::Integer => write!(f, "integer"),
            HomieDataType::Float => write!(f, "float"),
            HomieDataType::Boolean => write!(f, "boolean"),
            HomieDataType::String => write!(f, "string"),
            HomieDataType::Enum => write!(f, "enum"),
            HomieDataType::Color => write!(f, "color"),
            HomieDataType::Datetime => write!(f, "datetime"),
            HomieDataType::Duration => write!(f, "duration"),
            HomieDataType::JSON => write!(f, "json"),
        }
    }
}

impl FromStr for HomieDataType {
    type Err = Homie5ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "integer" => Ok(HomieDataType::Integer),
            "float" => Ok(HomieDataType::Float),
            "boolean" => Ok(HomieDataType::Boolean),
            "string" => Ok(HomieDataType::String),
            "enum" => Ok(HomieDataType::Enum),
            "color" => Ok(HomieDataType::Color),
            "datetime" => Ok(HomieDataType::Datetime),
            "duration" => Ok(HomieDataType::Duration),
            "json" => Ok(HomieDataType::JSON),
            _ => Err(Homie5ProtocolError::InvalidHomieDataType),
        }
    }
}

/// Reflects the current state of the device.
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum HomieDeviceStatus {
    /// this is the state the device is in when it is connected to the MQTT broker, but has not yet sent all Homie messages and is not yet ready to operate. This state is optional and may be sent if the device takes a long time to initialize, but wishes to announce to consumers that it is coming online. A device may fall back into this state to do some reconfiguration.
    #[default]
    Init,
    /// this is the state the device is in when it is connected to the MQTT broker and has sent all Homie messages describing the device attributes, nodes, properties, and their values. The device has subscribed to all appropriate /set topics and is ready to receive messages.
    Ready,
    /// this is the state the device is in when it is cleanly disconnected from the MQTT broker. You must send this message before cleanly disconnecting.
    Disconnected,
    /// this is the state the device is in when the device is sleeping. You have to send this message before sleeping.
    Sleeping,
    /// this is the state the device is in when the device has been “badly” disconnected. Important: If a root-device $state is "lost" then the state of every child device in its tree is also "lost". You must define this message as the last will (LWT) for root devices.
    Lost,
}
impl HomieDeviceStatus {
    /// Returns the &str representation of the device status
    pub fn as_str(&self) -> &str {
        match self {
            HomieDeviceStatus::Init => "init",
            HomieDeviceStatus::Ready => "ready",
            HomieDeviceStatus::Disconnected => "disconnected",
            HomieDeviceStatus::Sleeping => "sleeping",
            HomieDeviceStatus::Lost => "lost",
        }
    }
}
impl Debug for HomieDeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Display for HomieDeviceStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            HomieDeviceStatus::Init => write!(f, "init"),
            HomieDeviceStatus::Ready => write!(f, "ready"),
            HomieDeviceStatus::Disconnected => write!(f, "disconnected"),
            HomieDeviceStatus::Sleeping => write!(f, "sleeping"),
            HomieDeviceStatus::Lost => write!(f, "lost"),
        }
    }
}

impl FromStr for HomieDeviceStatus {
    type Err = Homie5ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init" => Ok(HomieDeviceStatus::Init),
            "ready" => Ok(HomieDeviceStatus::Ready),
            "disconnected" => Ok(HomieDeviceStatus::Disconnected),
            "sleeping" => Ok(HomieDeviceStatus::Sleeping),
            "lost" => Ok(HomieDeviceStatus::Lost),
            _ => Err(Homie5ProtocolError::InvalidDeviceState(s.to_string())),
        }
    }
}

impl TryFrom<String> for HomieDeviceStatus {
    type Error = Homie5ProtocolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "init" => Ok(HomieDeviceStatus::Init),
            "ready" => Ok(HomieDeviceStatus::Ready),
            "disconnected" => Ok(HomieDeviceStatus::Disconnected),
            "sleeping" => Ok(HomieDeviceStatus::Sleeping),
            "lost" => Ok(HomieDeviceStatus::Lost),
            _ => Err(Homie5ProtocolError::InvalidDeviceState(value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceLogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

const DEVICE_LOG_LEVELS: [DeviceLogLevel; 5] = [
    DeviceLogLevel::Debug,
    DeviceLogLevel::Info,
    DeviceLogLevel::Warn,
    DeviceLogLevel::Error,
    DeviceLogLevel::Fatal,
];

impl DeviceLogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeviceLogLevel::Debug => "debug",
            DeviceLogLevel::Info => "info",
            DeviceLogLevel::Warn => "warn",
            DeviceLogLevel::Error => "error",
            DeviceLogLevel::Fatal => "fatal",
        }
    }
}

impl fmt::Display for DeviceLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for DeviceLogLevel {
    type Error = Homie5ProtocolError;

    fn try_from(value: String) -> Result<Self, <DeviceLogLevel as TryFrom<String>>::Error> {
        match value.as_str() {
            "debug" => Ok(DeviceLogLevel::Debug),
            "info" => Ok(DeviceLogLevel::Info),
            "warn" => Ok(DeviceLogLevel::Warn),
            "error" => Ok(DeviceLogLevel::Error),
            "fatal" => Ok(DeviceLogLevel::Fatal),
            _ => Err(Homie5ProtocolError::InvalidDeviceLogLevel(value)),
        }
    }
}

impl TryFrom<&str> for DeviceLogLevel {
    type Error = Homie5ProtocolError;

    fn try_from(value: &str) -> Result<Self, <DeviceLogLevel as TryFrom<String>>::Error> {
        match value {
            "debug" => Ok(DeviceLogLevel::Debug),
            "info" => Ok(DeviceLogLevel::Info),
            "warn" => Ok(DeviceLogLevel::Warn),
            "error" => Ok(DeviceLogLevel::Error),
            "fatal" => Ok(DeviceLogLevel::Fatal),
            _ => Err(Homie5ProtocolError::InvalidDeviceLogLevel(value.to_owned())),
        }
    }
}
/// This trait provides the capability to provide a mqtt topic for an object defining where it is
/// published on the broker
pub trait ToTopic {
    /// return the mqtt topic under which the object is published
    fn to_topic(&self) -> TopicBuilder;
}

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct TopicBuilder {
    topic: String,
}

impl Display for TopicBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the topic string to the formatter
        write!(f, "{}", self.topic)
    }
}

impl TopicBuilder {
    pub fn new(homie_domain: &HomieDomain) -> Self {
        let mut topic = String::with_capacity(96);
        topic.push_str(homie_domain.as_str());
        topic.push('/');
        topic.push_str(HOMIE_VERSION);
        Self { topic }
    }

    pub fn new_for_device(homie_domain: &HomieDomain, device_id: &HomieID) -> Self {
        Self::new(homie_domain).add_id(device_id)
    }

    pub fn new_for_node(homie_domain: &HomieDomain, device_id: &HomieID, node_id: &HomieID) -> Self {
        Self::new_for_device(homie_domain, device_id).add_id(node_id)
    }
    pub fn new_for_property(
        homie_domain: &HomieDomain,
        device_id: &HomieID,
        node_id: &HomieID,
        property_id: &HomieID,
    ) -> Self {
        Self::new_for_node(homie_domain, device_id, node_id).add_id(property_id)
    }
    pub fn add_attr(mut self, attr: &str) -> Self {
        self.topic.push('/');
        self.topic.push_str(attr);
        self
    }

    pub fn add_id(mut self, id: &HomieID) -> Self {
        self.topic.push('/');
        self.topic.push_str(id.as_str());
        self
    }
    pub fn build(self) -> String {
        self.topic
    }
}

/// unit for degrees in Celsius
pub const HOMIE_UNIT_DEGREE_CELSIUS: &str = "°C";
/// unit for degrees in Fahrenheit
pub const HOMIE_UNIT_DEGREE_FAHRENHEIT: &str = "°F";
/// unit for generic degrees
pub const HOMIE_UNIT_DEGREE: &str = "°";
/// unit for volume in liters
pub const HOMIE_UNIT_LITER: &str = "L";
/// unit for volume in gallons
pub const HOMIE_UNIT_GALLON: &str = "gal";
/// unit for voltage in volts
pub const HOMIE_UNIT_VOLT: &str = "V";
/// unit for power in watts
pub const HOMIE_UNIT_WATT: &str = "W";
/// unit for power in kilowatts
pub const HOMIE_UNIT_KILOWATT: &str = "kW";
/// unit for energy in kilowatt-hours
pub const HOMIE_UNIT_KILOWATTHOUR: &str = "kWh";
/// unit for electric current in amperes
pub const HOMIE_UNIT_AMPERE: &str = "A";
/// unit for frequency in hertz
pub const HOMIE_UNIT_HERTZ: &str = "Hz";
/// unit for electric current in milliamperes
pub const HOMIE_UNIT_MILI_AMPERE: &str = "mA";
/// unit for percentage
pub const HOMIE_UNIT_PERCENT: &str = "%";
/// unit for length in meters
pub const HOMIE_UNIT_METER: &str = "m";
/// unit for volume in cubic meters
pub const HOMIE_UNIT_CUBIC_METER: &str = "m³";
/// unit for length in feet
pub const HOMIE_UNIT_FEET: &str = "ft";
/// unit for pressure in pascals
pub const HOMIE_UNIT_PASCAL: &str = "Pa";
/// unit for pressure in kilopascals
pub const HOMIE_UNIT_KILOPASCAL: &str = "kPa";
/// unit for pressure in pounds per square inch
pub const HOMIE_UNIT_PSI: &str = "psi";
/// unit for time in seconds
pub const HOMIE_UNIT_SECONDS: &str = "s";
/// unit for time in minutes
pub const HOMIE_UNIT_MINUTES: &str = "min";
/// unit for time in hours
pub const HOMIE_UNIT_HOURS: &str = "h";
/// unit for illuminance in lux
pub const HOMIE_UNIT_LUX: &str = "lx";
/// unit for temperature in kelvin
pub const HOMIE_UNIT_KELVIN: &str = "K";
/// unit for color temperature in mireds (reciprocal megakelvin)
pub const HOMIE_UNIT_MIRED: &str = "MK⁻¹";
/// unit for countable amounts
pub const HOMIE_UNIT_COUNT_AMOUNT: &str = "#";
