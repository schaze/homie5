//===========================================================
//=== NODE
//===========================================================

use crate::{DeviceRef, HomieDomain, HomieID, PropertyRef, ToTopic, HOMIE_VERSION};

/// Identifies a node of a device via its DeviceRef and its node id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct NodeRef {
    /// Identifier of the device the node belongs to
    pub device: DeviceRef,
    /// then node's id
    pub id: HomieID,
}

impl NodeRef {
    /// Create a new NodeRef from a given topic_root, device id, and node id
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
        *self == other.node
    }
}

impl PartialEq<PropertyRef> for &NodeRef {
    fn eq(&self, other: &PropertyRef) -> bool {
        **self == other.node
    }
}

impl ToTopic for NodeRef {
    fn to_topic(&self) -> String {
        format!(
            "{}/{HOMIE_VERSION}/{}/{}",
            self.device.homie_domain, self.device.id, self.id
        )
    }
}

impl From<&PropertyRef> for NodeRef {
    fn from(value: &PropertyRef) -> Self {
        value.node.clone()
    }
}
