//! //! Represents a reference to a property in the Homie MQTT convention.
//!
//! A `PropertyRef` identifies a property within a Homie device by the owned DeviceRef as well as a pointer (`PropertyPointer`) to the property within the device consisting of a node id and property id.
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

use crate::AsPropPointer;
use crate::{AsNodeId, DeviceRef, HomieDomain, HomieID, NodeRef, ToTopic, TopicBuilder};

use super::PropertyPointer;

/// Identifies a property of a node via its NodeRef and the property id
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyRef {
    /// Identifier of the node the property belongs to
    pub(crate) device: DeviceRef,
    pub(crate) prop_pointer: PropertyPointer,
}

impl PropertyRef {
    /// Create a new PropertyRef from a given homie_domain, device id, node id, and property id
    pub fn new(homie_domain: HomieDomain, device_id: HomieID, node_id: HomieID, prop_id: HomieID) -> Self {
        Self {
            device: DeviceRef::new(homie_domain, device_id),
            prop_pointer: PropertyPointer { node_id, prop_id },
        }
    }

    /// Create a new PropertyRef from an existing NodeRef and a property id
    pub fn from_node(node: NodeRef, prop_id: HomieID) -> Self {
        Self {
            device: node.device,
            prop_pointer: PropertyPointer {
                node_id: node.id,
                prop_id,
            },
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

    pub fn match_with_node(&self, node: &NodeRef, prop_id: &HomieID) -> bool {
        self == node && &self.prop_pointer.prop_id == prop_id
    }
    pub fn match_with_device(&self, device: &DeviceRef, node_id: &HomieID, prop_id: &HomieID) -> bool {
        &self.device == device && &self.prop_pointer.node_id == node_id && &self.prop_pointer.prop_id == prop_id
    }
}

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

// Partial impls
// ================================

impl PartialEq<DeviceRef> for PropertyRef {
    fn eq(&self, other: &DeviceRef) -> bool {
        &self.device == other
    }
}
impl PartialEq<DeviceRef> for &PropertyRef {
    fn eq(&self, other: &DeviceRef) -> bool {
        &self.device == other
    }
}
impl PartialEq<&DeviceRef> for PropertyRef {
    fn eq(&self, other: &&DeviceRef) -> bool {
        &&self.device == other
    }
}

impl PartialEq<NodeRef> for PropertyRef {
    fn eq(&self, other: &NodeRef) -> bool {
        self.device == other.device && self.prop_pointer.node_id == other.id
    }
}

impl PartialEq<&NodeRef> for PropertyRef {
    fn eq(&self, other: &&NodeRef) -> bool {
        self.device == other.device && self.prop_pointer.node_id == other.id
    }
}

impl PartialEq<NodeRef> for &PropertyRef {
    fn eq(&self, other: &NodeRef) -> bool {
        self.device == other.device && self.prop_pointer.node_id == other.id
    }
}

// ToTopic impls
// ===================================

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
