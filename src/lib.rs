//! This is a very low level implemenation of the homie5 protocol in rust.
//! It aims to be as flexible and unopinionated as possible. There is no direct dependency to a mqtt library.
//! homie5 provides a basic support for a protocol implementation for homie v5 with a clearly defined interface
//! point to a mqtt library. The library provides fully typed support for all homie5 datatypes.
//!
//! Due to this, the usage of the library is a bit more involved as with a completly ready to use homie library.
//! Benefit is however that you can use the library basically everywhere from a simple esp32, raspberrypi to a x86 machine.
//!
//! You can find working examples for both device and controller use case in the examples/ folder:
//!
//!    - controller_example.rs:
//!      Implements a homie5 controller that will discover all homie5 devices on a
//!      mqtt broker and print out the devices and their property updates (more information).
//!    - device_example.rs:
//!      Implements a simple LightDevice with state and brightness control properties (more information).
//!
//! Both examples use rumqttc as a mqtt client implementation and provide a best practice in homie5
//! usage and in how to integrate the 2 libraries.
//!

pub mod client;
mod controller_proto;
pub mod device_description;
mod device_proto;
mod error;
mod homie5_message;
mod statemachine;
mod value;

pub use controller_proto::*;
pub use device_proto::*;
pub use error::Homie5ProtocolError;
pub use homie5_message::*;
pub use value::*;

use serde::{Deserialize, Serialize};

use std::fmt;
use std::fmt::{Debug, Display};
use std::str::FromStr;

/// The default mqtt root topic: "homie"
pub const DEFAULT_ROOT_TOPIC: &str = "homie";
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
#[derive(Serialize, Deserialize, Default, Clone, Hash, PartialEq, Eq)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// This trait provide the capability to provide a mqtt topic for an object defining where it is
/// published on the broker
pub trait ToTopic {
    /// return the mqtt topic under which the object is published
    fn to_topic(&self) -> String;
    /// return the mqtt topic under which the object is published with the addition of a subpath
    fn to_topic_with_subpath(&self, subpath: &str) -> String;
}

//===========================================================
//=== DEVICE
//===========================================================

/// Identifies a device via topic_root and the device id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct DeviceIdentifier {
    /// the mqtt topic_root (e.g. homie) under which the device is published
    pub topic_root: String,
    /// the homie device ID
    pub id: String,
}
impl DeviceIdentifier {
    /// Create a new DeviceIdentifier from a given topic_root and a device id
    pub fn new(topic_root: String, device_id: String) -> Self {
        Self {
            topic_root,
            id: device_id,
        }
    }
    /// return a slice to the device id
    pub fn device_id(&self) -> &str {
        &self.id
    }
}

impl PartialEq<PropertyIdentifier> for DeviceIdentifier {
    fn eq(&self, other: &PropertyIdentifier) -> bool {
        other.node.device == *self
    }
}

impl PartialEq<PropertyIdentifier> for &DeviceIdentifier {
    fn eq(&self, other: &PropertyIdentifier) -> bool {
        other.node.device == **self
    }
}
impl PartialEq<NodeIdentifier> for DeviceIdentifier {
    fn eq(&self, other: &NodeIdentifier) -> bool {
        other.device == *self
    }
}

impl PartialEq<NodeIdentifier> for &DeviceIdentifier {
    fn eq(&self, other: &NodeIdentifier) -> bool {
        other.device == **self
    }
}
impl ToTopic for DeviceIdentifier {
    fn to_topic(&self) -> String {
        format!("{}/{HOMIE_VERSION}/{}", self.topic_root, self.id)
    }
    fn to_topic_with_subpath(&self, subpath: &str) -> String {
        format!("{}/{HOMIE_VERSION}/{}/{}", self.topic_root, self.id, subpath)
    }
}

impl From<&PropertyIdentifier> for DeviceIdentifier {
    /// Create a DeviceIdentifier from a PropertyIdentifier
    fn from(value: &PropertyIdentifier) -> Self {
        value.node.device.clone()
    }
}

impl From<&NodeIdentifier> for DeviceIdentifier {
    /// Create a DeviceIdentifier from a NodeIdentifier
    fn from(value: &NodeIdentifier) -> Self {
        value.device.clone()
    }
}

//===========================================================
//=== NODE
//===========================================================

/// Identifies a node of a device via its DeviceIdentifier and its node id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct NodeIdentifier {
    /// Identifier of the device the node belongs to
    pub device: DeviceIdentifier,
    /// then node's id
    pub id: String,
}

impl NodeIdentifier {
    /// Create a new NodeIdentifier from a given topic_root, device id, and node id
    pub fn new(topic_root: String, device_id: String, node_id: String) -> Self {
        Self {
            device: DeviceIdentifier::new(topic_root, device_id),
            id: node_id,
        }
    }

    /// Create a new NodeIdentifier from an existing DeviceIdentifier and a node id
    pub fn from_device(device: DeviceIdentifier, node_id: String) -> Self {
        Self { device, id: node_id }
    }

    /// Return a slice of the node id
    pub fn node_id(&self) -> &str {
        &self.id
    }

    /// Return a slice of the device id the node belongs to
    pub fn device_id(&self) -> &str {
        &self.device.id
    }
}

impl PartialEq<DeviceIdentifier> for NodeIdentifier {
    fn eq(&self, other: &DeviceIdentifier) -> bool {
        &self.device == other
    }
}

impl PartialEq<&DeviceIdentifier> for NodeIdentifier {
    fn eq(&self, other: &&DeviceIdentifier) -> bool {
        &&self.device == other
    }
}

impl PartialEq<DeviceIdentifier> for &NodeIdentifier {
    fn eq(&self, other: &DeviceIdentifier) -> bool {
        &self.device == other
    }
}
impl PartialEq<PropertyIdentifier> for NodeIdentifier {
    fn eq(&self, other: &PropertyIdentifier) -> bool {
        *self == other.node
    }
}

impl PartialEq<PropertyIdentifier> for &NodeIdentifier {
    fn eq(&self, other: &PropertyIdentifier) -> bool {
        **self == other.node
    }
}

impl ToTopic for NodeIdentifier {
    fn to_topic(&self) -> String {
        format!(
            "{}/{HOMIE_VERSION}/{}/{}",
            self.device.topic_root, self.device.id, self.id
        )
    }
    fn to_topic_with_subpath(&self, subpath: &str) -> String {
        format!(
            "{}/{HOMIE_VERSION}/{}/{}/{}",
            self.device.topic_root, self.device.id, self.id, subpath
        )
    }
}

impl From<&PropertyIdentifier> for NodeIdentifier {
    fn from(value: &PropertyIdentifier) -> Self {
        value.node.clone()
    }
}

//===========================================================
//=== PROPERTY
//===========================================================

/// Identifies a property of a node via its NodeIdentifier and the property id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct PropertyIdentifier {
    /// Identifier of the node the property belongs to
    pub node: NodeIdentifier,
    /// The property's id within the node
    pub id: String,
}

impl PropertyIdentifier {
    /// Create a new PropertyIdentifier from a given topic_root, device id, node id, and property id
    pub fn new(topic_root: String, device_id: String, node_id: String, prop_id: String) -> Self {
        Self {
            node: NodeIdentifier::new(topic_root, device_id, node_id),
            id: prop_id,
        }
    }

    /// Create a new PropertyIdentifier from an existing NodeIdentifier and a property id
    pub fn from_node(node: NodeIdentifier, prop_id: String) -> Self {
        Self { node, id: prop_id }
    }

    /// Return a slice of the property id
    pub fn prop_id(&self) -> &str {
        &self.id
    }

    /// Return a slice of the node id the property belongs to
    pub fn node_id(&self) -> &str {
        &self.node.id
    }

    /// Return a slice of the device id the property belongs to
    pub fn device_id(&self) -> &str {
        &self.node.device.id
    }
}

impl PartialEq<DeviceIdentifier> for PropertyIdentifier {
    fn eq(&self, other: &DeviceIdentifier) -> bool {
        &self.node.device == other
    }
}
impl PartialEq<DeviceIdentifier> for &PropertyIdentifier {
    fn eq(&self, other: &DeviceIdentifier) -> bool {
        &self.node.device == other
    }
}
impl PartialEq<&DeviceIdentifier> for PropertyIdentifier {
    fn eq(&self, other: &&DeviceIdentifier) -> bool {
        &&self.node.device == other
    }
}

impl PartialEq<NodeIdentifier> for PropertyIdentifier {
    fn eq(&self, other: &NodeIdentifier) -> bool {
        &self.node == other
    }
}

impl PartialEq<&NodeIdentifier> for PropertyIdentifier {
    fn eq(&self, other: &&NodeIdentifier) -> bool {
        &&self.node == other
    }
}

impl PartialEq<NodeIdentifier> for &PropertyIdentifier {
    fn eq(&self, other: &NodeIdentifier) -> bool {
        &self.node == other
    }
}

impl PropertyIdentifier {
    pub fn match_with_node(&self, node: &NodeIdentifier, prop_id: &str) -> bool {
        self == node && self.id == prop_id
    }
}

impl ToTopic for PropertyIdentifier {
    fn to_topic(&self) -> String {
        format!(
            "{}/{HOMIE_VERSION}/{}/{}/{}",
            self.node.device.topic_root, self.node.device.id, self.node.id, self.id
        )
    }
    fn to_topic_with_subpath(&self, subpath: &str) -> String {
        format!(
            "{}/{HOMIE_VERSION}/{}/{}/{}/{}",
            self.node.device.topic_root, self.node.device.id, self.node.id, self.id, subpath
        )
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
