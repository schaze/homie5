//! Represents a device-relative pointer to a property in the Homie MQTT convention.
//!
//! A `PropertyPointer` identifies a property within a device by its node ID and property ID,
//! without carrying the device identity. This makes it ideal as a HashMap key in device-scoped
//! property stores.
//!
//! # Display / FromStr
//!
//! The `Display` format is `"{node_id}/{prop_id}"` (e.g. `"node-01/temperature"`).
//! `FromStr` parses the same format.

use std::fmt;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::{Homie5ProtocolError, HomieID};

use super::AsPropPointer;

/// Identifies a property relative to its device via node_id and prop_id.
///
/// This is useful as a HashMap key in device-scoped property stores where the
/// device identity is already known from context.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PropertyPointer {
    pub(crate) node_id: HomieID,
    pub(crate) prop_id: HomieID,
}

impl PropertyPointer {
    pub fn new(node_id: HomieID, prop_id: HomieID) -> Self {
        Self { node_id, prop_id }
    }
    pub fn node_id(&self) -> &HomieID {
        &self.node_id
    }
    pub fn prop_id(&self) -> &HomieID {
        &self.prop_id
    }
    pub fn into_parts(self) -> (HomieID, HomieID) {
        (self.node_id, self.prop_id)
    }
}

// ---- Display / FromStr ----

impl fmt::Display for PropertyPointer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.node_id, self.prop_id)
    }
}

impl FromStr for PropertyPointer {
    type Err = Homie5ProtocolError;

    /// Parse a PropertyPointer from `"{node_id}/{prop_id}"`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        match parts.len() {
            2 => Ok(PropertyPointer::new(
                parts[0].to_string().try_into()?,
                parts[1].to_string().try_into()?,
            )),
            _ => Err(Homie5ProtocolError::InvalidRefFormat(format!(
                "expected 2 segments for PropertyPointer, got {}",
                parts.len()
            ))),
        }
    }
}

// ---- Serde (string-based) ----

impl Serialize for PropertyPointer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PropertyPointer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PropertyPointer::from_str(&s).map_err(de::Error::custom)
    }
}

// ---- AsPropPointer ----

impl AsPropPointer for PropertyPointer {
    fn as_prop_pointer(&self) -> &PropertyPointer {
        self
    }
}

impl AsPropPointer for &PropertyPointer {
    fn as_prop_pointer(&self) -> &PropertyPointer {
        self
    }
}
