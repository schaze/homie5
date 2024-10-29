//! Represents a reference to a device in the Homie MQTT convention.
//!
//! A `DeviceRef` identifies a Homie device by its domain (`HomieDomain`) and device ID (`HomieID`). This struct is used to interact with and reference Homie devices in the MQTT topic structure.
//!
//! # Example
//!
//! ```rust
//! use homie5::{DeviceRef, HomieDomain, HomieID};
//!
//! let device_id = HomieID::try_from("device-01").unwrap();
//! let device_ref = DeviceRef::new(HomieDomain::Default, device_id);
//!
//! assert_eq!(device_ref.device_id().as_str(), "device-01");
//! ```
//!
//! # Methods
//! - `new`: Constructs a `DeviceRef` from a domain and device ID.
//! - `device_id`: Returns a reference to the device ID.
//!
//! These methods enable referencing Homie devices within the MQTT topic structure.

use crate::{HomieDomain, HomieID, NodeRef, PropertyRef, ToTopic, TopicBuilder};

/// Identifies a device via homie-domain and the device id
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceRef {
    /// the homie_domain (e.g. homie) under which the device is published
    pub homie_domain: HomieDomain,
    /// the homie device ID
    pub id: HomieID,
}
impl DeviceRef {
    /// Create a new DeviceRef from a given homie-domain and a device id
    pub fn new(homie_domain: HomieDomain, id: HomieID) -> Self {
        Self { homie_domain, id }
    }
    /// return a slice to the device id
    pub fn device_id(&self) -> &HomieID {
        &self.id
    }
    /// Return a reference to the homie domain the device belongs to
    pub fn homie_domain(&self) -> &HomieDomain {
        &self.homie_domain
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
    fn to_topic(&self) -> TopicBuilder {
        TopicBuilder::new_for_device(&self.homie_domain, &self.id)
    }
}

impl ToTopic for (&HomieDomain, &HomieID) {
    fn to_topic(&self) -> TopicBuilder {
        TopicBuilder::new_for_device(self.0, self.1)
    }
}

impl From<DeviceRef> for TopicBuilder {
    fn from(value: DeviceRef) -> Self {
        Self::new_for_device(&value.homie_domain, value.device_id())
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
