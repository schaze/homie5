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

pub const DEFAULT_ROOT_TOPIC: &str = "homie";
pub const HOMIE_VERSION: &str = "5";
pub const HOMIE_VERSION_FULL: &str = "5.0";
pub const HOMIE_TOPIC_BROADCAST: &str = "$broadcast";

pub const DEVICE_ATTRIBUTE_STATE: &str = "$state";
pub const DEVICE_ATTRIBUTE_LOG: &str = "$log";
pub const DEVICE_ATTRIBUTE_DESCRIPTION: &str = "$description";
pub const DEVICE_ATTRIBUTE_ALERT: &str = "$alert";
pub const DEVICE_ATTRIBUTES: [&str; 4] = [
    DEVICE_ATTRIBUTE_STATE, // state MUST be first in this array due to use in device removal
    DEVICE_ATTRIBUTE_LOG,
    DEVICE_ATTRIBUTE_ALERT,
    DEVICE_ATTRIBUTE_DESCRIPTION,
];

pub const PROPERTY_SET_TOPIC: &str = "set";
pub const PROPERTY_ATTRIBUTE_TARGET: &str = "$target";

#[derive(Debug, Serialize, Deserialize)]
pub enum ValueMappingValue {
    Homie(HomieValuesTypes),
    String(String),
}
#[derive(Debug, Serialize, Deserialize)]
pub enum HomieValuesTypes {
    Null,
    String(String),
    Integer(i32),
    Float(f32),
    Bool(bool),
    Date(i32),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ValueMapping {
    from: Option<ValueMappingValue>,
    to: Option<ValueMappingValue>,
}

pub type ValueMappingList = Vec<ValueMapping>;
#[derive(Debug, Serialize, Deserialize)]
pub struct ValueMappingDefintion {
    incoming: ValueMappingList,
    outgoing: ValueMappingList,
}

#[derive(Serialize, Deserialize, Default, Clone, Hash, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HomieDataType {
    #[default]
    Integer,
    Float,
    Boolean,
    String,
    Enum,
    Color,
    Datetime,
    Duration,
    JSON,
}

impl HomieDataType {
    pub fn to_str(&self) -> &str {
        match self {
            HomieDataType::Integer => "integer",
            HomieDataType::Float => "float",
            HomieDataType::Boolean => "boolean",
            HomieDataType::String => "string",
            HomieDataType::Enum => "enum",
            HomieDataType::Color => "color",
            HomieDataType::Datetime => "datetime",
            HomieDataType::Duration => "duration",
            HomieDataType::JSON => "json",
        }
    }
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
    type Err = ();

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
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum HomieDeviceStatus {
    #[default]
    Init,
    Ready,
    Disconnected,
    Sleeping,
    Lost,
}
impl HomieDeviceStatus {
    pub fn to_str(&self) -> &str {
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "init" => Ok(HomieDeviceStatus::Init),
            "ready" => Ok(HomieDeviceStatus::Ready),
            "disconnected" => Ok(HomieDeviceStatus::Disconnected),
            "sleeping" => Ok(HomieDeviceStatus::Sleeping),
            "lost" => Ok(HomieDeviceStatus::Lost),
            _ => Err(()),
        }
    }
}

pub trait ToTopic {
    fn to_topic(&self) -> String;
    fn to_topic_with_subpath(&self, subpath: &str) -> String;
}

//===========================================================
//=== DEVICE
//===========================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct DeviceIdentifier {
    pub topic_root: String,
    pub id: String,
}
impl DeviceIdentifier {
    pub fn new(topic_root: String, device_id: String) -> Self {
        Self {
            topic_root,
            id: device_id,
        }
    }
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
    fn from(value: &PropertyIdentifier) -> Self {
        value.node.device.clone()
    }
}

impl From<&NodeIdentifier> for DeviceIdentifier {
    fn from(value: &NodeIdentifier) -> Self {
        value.device.clone()
    }
}

//===========================================================
//=== NODE
//===========================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct NodeIdentifier {
    pub device: DeviceIdentifier,
    pub id: String,
}

impl NodeIdentifier {
    pub fn new(topic_root: String, device_id: String, node_id: String) -> Self {
        Self {
            device: DeviceIdentifier::new(topic_root, device_id),
            id: node_id,
        }
    }
    pub fn from_device(device: DeviceIdentifier, node_id: String) -> Self {
        Self { device, id: node_id }
    }

    pub fn node_id(&self) -> &str {
        &self.id
    }

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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct PropertyIdentifier {
    pub node: NodeIdentifier,
    pub id: String,
}

impl PropertyIdentifier {
    pub fn new(topic_root: String, device_id: String, node_id: String, prop_id: String) -> Self {
        Self {
            node: NodeIdentifier::new(topic_root, device_id, node_id),
            id: prop_id,
        }
    }

    pub fn from_node(node: NodeIdentifier, prop_id: String) -> Self {
        Self { node, id: prop_id }
    }

    pub fn prop_id(&self) -> &str {
        &self.id
    }

    pub fn node_id(&self) -> &str {
        &self.node.id
    }

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

pub const HOMIE_UNIT_DEGREE_CELSIUS: &str = "°C";
pub const HOMIE_UNIT_DEGREE_FAHRENHEIT: &str = "°F";
pub const HOMIE_UNIT_DEGREE: &str = "°";
pub const HOMIE_UNIT_LITER: &str = "L";
pub const HOMIE_UNIT_GALLON: &str = "gal";
pub const HOMIE_UNIT_VOLT: &str = "V";
pub const HOMIE_UNIT_WATT: &str = "W";
pub const HOMIE_UNIT_KILOWATT: &str = "kW";
pub const HOMIE_UNIT_KILOWATTHOUR: &str = "kWh";
pub const HOMIE_UNIT_AMPERE: &str = "A";
pub const HOMIE_UNIT_HERTZ: &str = "Hz";
pub const HOMIE_UNIT_MILI_AMPERE: &str = "mA";
pub const HOMIE_UNIT_PERCENT: &str = "%";
pub const HOMIE_UNIT_METER: &str = "m";
pub const HOMIE_UNIT_CUBIC_METER: &str = "m³";
pub const HOMIE_UNIT_FEET: &str = "ft";
pub const HOMIE_UNIT_PASCAL: &str = "Pa";
pub const HOMIE_UNIT_KILOPASCAL: &str = "kPa";
pub const HOMIE_UNIT_PSI: &str = "psi";
pub const HOMIE_UNIT_SECONDS: &str = "s";
pub const HOMIE_UNIT_MINUTES: &str = "min";
pub const HOMIE_UNIT_HOURS: &str = "h";
pub const HOMIE_UNIT_LUX: &str = "lx";
pub const HOMIE_UNIT_KELVIN: &str = "K";
pub const HOMIE_UNIT_MIRED: &str = "MK⁻¹";
pub const HOMIE_UNIT_COUNT_AMOUNT: &str = "#";
