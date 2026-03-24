//! Represents a reference to a device in the Homie MQTT convention.
//!
//! A `DeviceRef` identifies a Homie device by its domain (`HomieDomain`) and device ID (`HomieID`).
//!
//! # Display / FromStr
//!
//! The `Display` format is `"{domain}/{device_id}"` (e.g. `"homie/device-01"`).
//! `FromStr` parses both `"{device_id}"` (default domain) and `"{domain}/{device_id}"`.
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
//! assert_eq!(device_ref.to_string(), "homie/device-01");
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::{HomieDomain, Homie5ProtocolError, HomieID, ToTopic, TopicBuilder, get_fallback_homie_domain};

/// Identifies a device via homie-domain and the device id
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceRef {
    /// the homie_domain (e.g. homie) under which the device is published
    pub(crate) homie_domain: HomieDomain,
    /// the homie device ID
    pub(crate) id: HomieID,
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

    pub fn clone_with_id(&self, id: HomieID) -> DeviceRef {
        Self {
            homie_domain: self.homie_domain.clone(),
            id,
        }
    }

    pub fn into_parts(self) -> (HomieDomain, HomieID) {
        (self.homie_domain, self.id)
    }
}

// ---- Display / FromStr ----

impl fmt::Display for DeviceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.homie_domain, self.id)
    }
}

impl FromStr for DeviceRef {
    type Err = Homie5ProtocolError;

    /// Parse a DeviceRef from `"{domain}/{device_id}"` or `"{device_id}"` (default domain).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        match parts.len() {
            1 => Ok(DeviceRef::new(
                get_fallback_homie_domain(),
                parts[0].to_string().try_into()?,
            )),
            2 => Ok(DeviceRef::new(
                parts[0].to_string().try_into()?,
                parts[1].to_string().try_into()?,
            )),
            _ => Err(Homie5ProtocolError::InvalidRefFormat(format!(
                "expected 1-2 segments for DeviceRef, got {}",
                parts.len()
            ))),
        }
    }
}

// ---- Serde (string-based) ----

impl Serialize for DeviceRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DeviceRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DeviceRef::from_str(&s).map_err(de::Error::custom)
    }
}

// ---- ToTopic ----

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
