//! Represents a reference to a property in the Homie MQTT convention.
//!
//! A `PropertyRef` identifies a property within a Homie device by the owned DeviceRef as well as a
//! pointer (`PropertyPointer`) to the property within the device consisting of a node id and property id.
//!
//! # Display / FromStr
//!
//! The `Display` format is `"{domain}/{device_id}/{node_id}/{prop_id}"`.
//! `FromStr` parses both `"{device_id}/{node_id}/{prop_id}"` (default domain) and
//! `"{domain}/{device_id}/{node_id}/{prop_id}"`.
//!
//! # Example
//!
//! ```rust
//! use homie5::{PropertyRef, NodeRef, HomieDomain, HomieID};
//!
//! let device_id = HomieID::try_from("device-01").unwrap();
//! let node_id = HomieID::try_from("node-01").unwrap();
//! let prop_id = HomieID::try_from("temperature").unwrap();
//! let prop_ref = PropertyRef::new(HomieDomain::Default, device_id, node_id, prop_id);
//!
//! assert_eq!(prop_ref.device_id().as_str(), "device-01");
//! assert_eq!(prop_ref.node_id().as_str(), "node-01");
//! assert_eq!(prop_ref.prop_id().as_str(), "temperature");
//! assert_eq!(prop_ref.to_string(), "homie/device-01/node-01/temperature");
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::AsPropPointer;
use crate::{AsNodeId, DeviceRef, HomieDomain, Homie5ProtocolError, HomieID, NodeRef, ToTopic, TopicBuilder, get_fallback_homie_domain};

use super::PropertyPointer;

/// Identifies a property of a node via its DeviceRef and the PropertyPointer (node_id + prop_id)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyRef {
    /// Identifier of the device the property belongs to
    pub(crate) device: DeviceRef,
    pub(crate) prop_pointer: PropertyPointer,
}

impl PropertyRef {
    /// Create a new PropertyRef from a given homie_domain, device id, node id, and property id
    pub fn new(homie_domain: HomieDomain, device_id: HomieID, node_id: HomieID, prop_id: HomieID) -> Self {
        Self {
            device: DeviceRef::new(homie_domain, device_id),
            prop_pointer: PropertyPointer::new(node_id, prop_id),
        }
    }

    /// Create a new PropertyRef from an existing NodeRef and a property id
    pub fn from_node(node: NodeRef, prop_id: HomieID) -> Self {
        Self {
            device: node.device,
            prop_pointer: PropertyPointer::new(node.id, prop_id),
        }
    }

    /// Return a slice of the property id
    pub fn prop_id(&self) -> &HomieID {
        &self.prop_pointer.prop_id
    }

    /// Return a slice of the node id the property belongs to
    pub fn node_id(&self) -> &HomieID {
        &self.prop_pointer.node_id
    }

    /// Return a slice of the device id the property belongs to
    pub fn device_id(&self) -> &HomieID {
        &self.device.id
    }

    /// Return a reference to the homie domain the property belongs to
    pub fn homie_domain(&self) -> &HomieDomain {
        &self.device.homie_domain
    }

    pub fn device_ref(&self) -> &DeviceRef {
        &self.device
    }

    pub fn prop_pointer(&self) -> &PropertyPointer {
        &self.prop_pointer
    }

    /// Construct a NodeRef for the node this property belongs to.
    ///
    /// This allocates (clones the inner DeviceRef and node HomieID).
    pub fn to_node_ref(&self) -> NodeRef {
        NodeRef::from_device(self.device.clone(), self.prop_pointer.node_id.clone())
    }

    /// Returns true if this property belongs to the given device
    pub fn belongs_to_device(&self, device: &DeviceRef) -> bool {
        &self.device == device
    }

    /// Returns true if this property belongs to the given node
    pub fn belongs_to_node(&self, node: &NodeRef) -> bool {
        &self.device == node.device_ref() && &self.prop_pointer.node_id == node.node_id()
    }

    pub fn match_with_node(&self, node: &NodeRef, prop_id: &HomieID) -> bool {
        self.belongs_to_node(node) && &self.prop_pointer.prop_id == prop_id
    }

    pub fn match_with_device(&self, device: &DeviceRef, node_id: &HomieID, prop_id: &HomieID) -> bool {
        &self.device == device && &self.prop_pointer.node_id == node_id && &self.prop_pointer.prop_id == prop_id
    }

    pub fn into_parts(self) -> (HomieDomain, HomieID, HomieID, HomieID) {
        let (homie_domain, device_id) = self.device.into_parts();
        let (node_id, prop_id) = self.prop_pointer.into_parts();
        (homie_domain, device_id, node_id, prop_id)
    }
}

// ---- Trait impls ----

impl AsPropPointer for PropertyRef {
    fn as_prop_pointer(&self) -> &PropertyPointer {
        self.prop_pointer()
    }
}

impl AsPropPointer for &PropertyRef {
    fn as_prop_pointer(&self) -> &PropertyPointer {
        self.prop_pointer()
    }
}

impl AsNodeId for PropertyRef {
    fn as_node_id(&self) -> &HomieID {
        self.node_id()
    }
}

impl AsNodeId for &PropertyRef {
    fn as_node_id(&self) -> &HomieID {
        self.node_id()
    }
}

// ---- Display / FromStr ----

impl fmt::Display for PropertyRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/{}",
            self.device.homie_domain, self.device.id, self.prop_pointer.node_id, self.prop_pointer.prop_id
        )
    }
}

impl FromStr for PropertyRef {
    type Err = Homie5ProtocolError;

    /// Parse a PropertyRef from `"{device_id}/{node_id}/{prop_id}"` (default domain)
    /// or `"{domain}/{device_id}/{node_id}/{prop_id}"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        match parts.len() {
            3 => Ok(PropertyRef::new(
                get_fallback_homie_domain(),
                parts[0].to_string().try_into()?,
                parts[1].to_string().try_into()?,
                parts[2].to_string().try_into()?,
            )),
            4 => Ok(PropertyRef::new(
                parts[0].to_string().try_into()?,
                parts[1].to_string().try_into()?,
                parts[2].to_string().try_into()?,
                parts[3].to_string().try_into()?,
            )),
            _ => Err(Homie5ProtocolError::InvalidRefFormat(format!(
                "expected 3-4 segments for PropertyRef, got {}",
                parts.len()
            ))),
        }
    }
}

// ---- Serde (string-based) ----

impl Serialize for PropertyRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PropertyRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PropertyRef::from_str(&s).map_err(de::Error::custom)
    }
}

// ---- ToTopic ----

impl ToTopic for PropertyRef {
    fn to_topic(&self) -> TopicBuilder {
        TopicBuilder::new_for_property(self.homie_domain(), self.device_id(), self.node_id(), self.prop_id())
    }
}

impl ToTopic for (&HomieDomain, &HomieID, &HomieID, &HomieID) {
    fn to_topic(&self) -> TopicBuilder {
        TopicBuilder::new_for_property(self.0, self.1, self.2, self.3)
    }
}

impl ToTopic for (&HomieDomain, &HomieID, &HomieID, &HomieID, &str) {
    fn to_topic(&self) -> TopicBuilder {
        TopicBuilder::new_for_property(self.0, self.1, self.2, self.3).add_attr(self.4)
    }
}

// ---- From conversions ----

impl From<&PropertyRef> for DeviceRef {
    /// Create a DeviceRef from a PropertyRef
    fn from(value: &PropertyRef) -> Self {
        value.device.clone()
    }
}

impl From<&NodeRef> for DeviceRef {
    /// Create a DeviceRef from a NodeRef
    fn from(value: &NodeRef) -> Self {
        value.device_ref().clone()
    }
}

impl From<&PropertyRef> for NodeRef {
    fn from(value: &PropertyRef) -> Self {
        value.to_node_ref()
    }
}
