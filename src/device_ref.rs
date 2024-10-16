//===========================================================
//=== DEVICE
//===========================================================

use crate::{HomieID, NodeRef, PropertyRef, ToTopic, HOMIE_VERSION};

/// Identifies a device via topic_root and the device id
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct DeviceRef {
    /// the homie_domain (e.g. homie) under which the device is published
    pub homie_domain: String,
    /// the homie device ID
    pub id: HomieID,
}
impl DeviceRef {
    /// Create a new DeviceRef from a given topic_root and a device id
    pub fn new(homie_domain: String, id: HomieID) -> Self {
        Self { homie_domain, id }
    }
    /// return a slice to the device id
    pub fn device_id(&self) -> &HomieID {
        &self.id
    }
}

impl PartialEq<PropertyRef> for DeviceRef {
    fn eq(&self, other: &PropertyRef) -> bool {
        other.node.device == *self
    }
}

impl PartialEq<PropertyRef> for &DeviceRef {
    fn eq(&self, other: &PropertyRef) -> bool {
        other.node.device == **self
    }
}
impl PartialEq<NodeRef> for DeviceRef {
    fn eq(&self, other: &NodeRef) -> bool {
        other.device == *self
    }
}

impl PartialEq<NodeRef> for &DeviceRef {
    fn eq(&self, other: &NodeRef) -> bool {
        other.device == **self
    }
}
impl ToTopic for DeviceRef {
    fn to_topic(&self) -> String {
        format!("{}/{HOMIE_VERSION}/{}", self.homie_domain, self.id)
    }
}

impl From<&PropertyRef> for DeviceRef {
    /// Create a DeviceRef from a PropertyRef
    fn from(value: &PropertyRef) -> Self {
        value.node.device.clone()
    }
}

impl From<&NodeRef> for DeviceRef {
    /// Create a DeviceRef from a NodeRef
    fn from(value: &NodeRef) -> Self {
        value.device.clone()
    }
}
