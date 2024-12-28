//! //! Represents a reference to a node in the Homie MQTT convention.
//!
//! A `NodeRef` identifies a node on a Homie device through its associated `DeviceRef` and a node-specific ID (`HomieID`).
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
//! ```
//!
//! # Methods
//! - `new`: Constructs a `NodeRef` from a domain, device ID, and node ID.
//! - `from_device`: Creates a `NodeRef` from an existing `DeviceRef` and a node ID.
//! - `node_id`: Returns a reference to the node ID.
//! - `device_id`: Returns a reference to the device ID that the node belongs to.
//!
//! These methods allow precise identification and referencing of Homie nodes in MQTT topics.

use crate::AsNodeId;
use crate::{DeviceRef, HomieDomain, HomieID, PropertyRef, ToTopic, TopicBuilder};

/// Identifies a node of a device via its DeviceRef and its node id
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeRef {
    /// Identifier of the device the node belongs to
    pub(crate) device: DeviceRef,
    /// then node's id
    pub(crate) id: HomieID,
}

impl NodeRef {
    /// Create a new NodeRef from a given homie_domain, device id, and node id
    pub fn new(homie_domain: HomieDomain, device_i: HomieID, node_id: HomieID) -> Self {
        Self {
            device: DeviceRef::new(homie_domain, device_i),
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
}

impl AsNodeId for NodeRef {
    fn as_node_id(&self) -> &HomieID {
        self.node_id()
    }
}

impl PartialEq<DeviceRef> for NodeRef {
    fn eq(&self, other: &DeviceRef) -> bool {
        &self.device == other
    }
}

impl PartialEq<&DeviceRef> for NodeRef {
    fn eq(&self, other: &&DeviceRef) -> bool {
        &&self.device == other
    }
}

impl PartialEq<DeviceRef> for &NodeRef {
    fn eq(&self, other: &DeviceRef) -> bool {
        &self.device == other
    }
}
impl PartialEq<PropertyRef> for NodeRef {
    fn eq(&self, other: &PropertyRef) -> bool {
        self.device_ref() == other.device_ref() && self.node_id() == other.node_id()
    }
}

impl PartialEq<PropertyRef> for &NodeRef {
    fn eq(&self, other: &PropertyRef) -> bool {
        self.device_ref() == other.device_ref() && self.node_id() == other.node_id()
    }
}

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

impl From<&PropertyRef> for NodeRef {
    fn from(value: &PropertyRef) -> Self {
        Self {
            device: value.device_ref().clone(),
            id: value.node_id().clone(),
        }
    }
}
