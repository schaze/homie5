//! Represents a reference to a node in the Homie MQTT convention.
//!
//! A `NodeRef` identifies a node on a Homie device through its associated `DeviceRef` and a
//! node-specific ID (`HomieID`).
//!
//! # Display / FromStr
//!
//! The `Display` format is `"{domain}/{device_id}/{node_id}"` (e.g. `"homie/device-01/node-01"`).
//! `FromStr` parses both `"{device_id}/{node_id}"` (default domain) and `"{domain}/{device_id}/{node_id}"`.
//!
//! # Example
//!
//! ```rust
//! use homie5::{NodeRef, DeviceRef, HomieDomain, HomieID};
//!
//! let device_id = HomieID::try_from("device-01").unwrap();
//! let node_id = HomieID::try_from("node-01").unwrap();
//! let node_ref = NodeRef::new(HomieDomain::Default, device_id, node_id);
//!
//! assert_eq!(node_ref.device_id().as_str(), "device-01");
//! assert_eq!(node_ref.to_string(), "homie/device-01/node-01");
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::AsNodeId;
use crate::{DeviceRef, HomieDomain, Homie5ProtocolError, HomieID, ToTopic, TopicBuilder, get_fallback_homie_domain};

/// Identifies a node of a device via its DeviceRef and its node id
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeRef {
    /// Identifier of the device the node belongs to
    pub(crate) device: DeviceRef,
    /// the node's id
    pub(crate) id: HomieID,
}

impl NodeRef {
    /// Create a new NodeRef from a given homie_domain, device id, and node id
    pub fn new(homie_domain: HomieDomain, device_id: HomieID, node_id: HomieID) -> Self {
        Self {
            device: DeviceRef::new(homie_domain, device_id),
            id: node_id,
        }
    }

    /// Create a new NodeRef from an existing DeviceRef and a node id
    pub fn from_device(device: DeviceRef, node_id: HomieID) -> Self {
        Self { device, id: node_id }
    }

    /// Return a slice of the node id
    pub fn node_id(&self) -> &HomieID {
        &self.id
    }

    /// Return a slice of the device id the node belongs to
    pub fn device_id(&self) -> &HomieID {
        &self.device.id
    }

    /// Return a reference to the homie domain the node belongs to
    pub fn homie_domain(&self) -> &HomieDomain {
        &self.device.homie_domain
    }

    pub fn device_ref(&self) -> &DeviceRef {
        &self.device
    }

    pub fn into_parts(self) -> (HomieDomain, HomieID, HomieID) {
        let (homie_domain, device_id) = self.device.into_parts();
        (homie_domain, device_id, self.id)
    }

    /// Returns true if this node belongs to the given device
    pub fn belongs_to(&self, device: &DeviceRef) -> bool {
        &self.device == device
    }
}

impl AsNodeId for NodeRef {
    fn as_node_id(&self) -> &HomieID {
        self.node_id()
    }
}

// ---- Display / FromStr ----

impl fmt::Display for NodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.device.homie_domain, self.device.id, self.id)
    }
}

impl FromStr for NodeRef {
    type Err = Homie5ProtocolError;

    /// Parse a NodeRef from `"{device_id}/{node_id}"` (default domain)
    /// or `"{domain}/{device_id}/{node_id}"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        match parts.len() {
            2 => Ok(NodeRef::new(
                get_fallback_homie_domain(),
                parts[0].to_string().try_into()?,
                parts[1].to_string().try_into()?,
            )),
            3 => Ok(NodeRef::new(
                parts[0].to_string().try_into()?,
                parts[1].to_string().try_into()?,
                parts[2].to_string().try_into()?,
            )),
            _ => Err(Homie5ProtocolError::InvalidRefFormat(format!(
                "expected 2-3 segments for NodeRef, got {}",
                parts.len()
            ))),
        }
    }
}

// ---- Serde (string-based) ----

impl Serialize for NodeRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for NodeRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NodeRef::from_str(&s).map_err(de::Error::custom)
    }
}

// ---- ToTopic ----

impl ToTopic for NodeRef {
    fn to_topic(&self) -> TopicBuilder {
        TopicBuilder::new_for_node(self.homie_domain(), self.device_id(), self.node_id())
    }
}

impl ToTopic for (&HomieDomain, &HomieID, &HomieID) {
    fn to_topic(&self) -> TopicBuilder {
        TopicBuilder::new_for_node(self.0, self.1, self.2)
    }
}

// ---- From conversions ----

impl From<&DeviceRef> for DeviceRef {
    fn from(value: &DeviceRef) -> Self {
        value.clone()
    }
}
