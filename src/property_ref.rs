//! //! Represents a reference to a property in the Homie MQTT convention.
//!
//! A `PropertyRef` identifies a property within a Homie device by referencing the node (`NodeRef`) it belongs to and its property-specific ID (`HomieID`).
//! This is used in parsing and interacting with Homie messages and topics related to properties.
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
//! ```
//!
//! # Methods
//! - `new`: Constructs a `PropertyRef` from a domain, device ID, node ID, and property ID.
//! - `from_node`: Creates a `PropertyRef` from an existing `NodeRef` and a property ID.
//! - `prop_id`: Returns a reference to the property ID.
//! - `node_id`: Returns a reference to the node ID the property belongs to.
//! - `device_id`: Returns a reference to the device ID that the property belongs to.
//!
//! These methods allow precise identification and referencing of Homie properties in MQTT topics.

use crate::{DeviceRef, HomieDomain, HomieID, NodeRef, ToTopic, HOMIE_VERSION};

/// Identifies a property of a node via its NodeRef and the property id
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyRef {
    /// Identifier of the node the property belongs to
    pub node: NodeRef,
    /// The property's id within the node
    pub id: HomieID,
}

impl PropertyRef {
    /// Create a new PropertyRef from a given homie_domain, device id, node id, and property id
    pub fn new(homie_domain: HomieDomain, device_id: HomieID, node_id: HomieID, prop_id: HomieID) -> Self {
        Self {
            node: NodeRef::new(homie_domain, device_id, node_id),
            id: prop_id,
        }
    }

    /// Create a new PropertyRef from an existing NodeRef and a property id
    pub fn from_node(node: NodeRef, prop_id: HomieID) -> Self {
        Self { node, id: prop_id }
    }

    /// Return a slice of the property id
    pub fn prop_id(&self) -> &HomieID {
        &self.id
    }

    /// Return a slice of the node id the property belongs to
    pub fn node_id(&self) -> &HomieID {
        &self.node.id
    }

    /// Return a slice of the device id the property belongs to
    pub fn device_id(&self) -> &HomieID {
        &self.node.device.id
    }
}

impl PartialEq<DeviceRef> for PropertyRef {
    fn eq(&self, other: &DeviceRef) -> bool {
        &self.node.device == other
    }
}
impl PartialEq<DeviceRef> for &PropertyRef {
    fn eq(&self, other: &DeviceRef) -> bool {
        &self.node.device == other
    }
}
impl PartialEq<&DeviceRef> for PropertyRef {
    fn eq(&self, other: &&DeviceRef) -> bool {
        &&self.node.device == other
    }
}

impl PartialEq<NodeRef> for PropertyRef {
    fn eq(&self, other: &NodeRef) -> bool {
        &self.node == other
    }
}

impl PartialEq<&NodeRef> for PropertyRef {
    fn eq(&self, other: &&NodeRef) -> bool {
        &&self.node == other
    }
}

impl PartialEq<NodeRef> for &PropertyRef {
    fn eq(&self, other: &NodeRef) -> bool {
        &self.node == other
    }
}

impl PropertyRef {
    pub fn match_with_node(&self, node: &NodeRef, prop_id: &HomieID) -> bool {
        self == node && &self.id == prop_id
    }
}

impl ToTopic for PropertyRef {
    fn to_topic(&self) -> String {
        format!(
            "{}/{HOMIE_VERSION}/{}/{}/{}",
            self.node.device.homie_domain, self.node.device.id, self.node.id, self.id
        )
    }
}
