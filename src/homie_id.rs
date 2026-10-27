//! This module defines and validates Homie IDs used in the Homie MQTT convention.
//!
//! # HomieID
//!
//! `HomieID` ensures that the ID strings for devices, nodes, and properties follow the Homie specification:
//! - IDs must only include lowercase letters (`a-z`), digits (`0-9`), and hyphens (`-`).
//! - IDs must not be empty or contain any other characters.
//!
//! A `HomieID` can be created via `TryFrom<&'static str>` or `TryFrom<String>`.
//!
//! # Storage Backend
//!
//! `HomieID` stores either a `&'static str` or an `Arc<str>`, which makes cloning O(1)
//! (atomic reference count increment instead of string copy). This is ideal when IDs are cloned
//! frequently (e.g., as HashMap keys in reference types).
//!
//! Targets without atomic compare-and-swap (e.g. ESP32-C3, RP2040) have no `Arc`. There a
//! `HomieID` stores a `Box<str>` instead, and cloning an owned ID copies the string.
//!
//! # Errors
//!
//! If an ID fails to meet the specifications, an `InvalidHomieIDError` is returned with a message indicating the issue.
//!
//! # Examples
//!
//! ```rust
//! use homie5::*;
//! use std::convert::TryFrom;
//!
//! let valid_id = HomieID::try_from("device-01").unwrap();
//! assert_eq!(valid_id.as_str(), "device-01");
//!
//! let invalid_id = HomieID::try_from("Device-01"); // Returns an error due to uppercase letter
//! assert!(invalid_id.is_err());
//! ```

use core::convert::TryFrom;
use core::fmt;
use core::hash::{Hash, Hasher};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::prelude::*;
use crate::AsNodeId;

/// Owned string storage: `Arc<str>` where the target has atomic compare-and-swap on pointers,
/// `Box<str>` otherwise (e.g. ESP32-C3, RP2040), where `alloc::sync` does not exist.
#[cfg(target_has_atomic = "ptr")]
pub(crate) type SharedStr = alloc::sync::Arc<str>;
#[cfg(not(target_has_atomic = "ptr"))]
pub(crate) type SharedStr = alloc::boxed::Box<str>;

// ---- Inner representation ----

#[derive(Debug, Clone)]
enum HomieIDInner {
    Static(&'static str),
    Shared(SharedStr),
}

impl HomieIDInner {
    const fn new_static(s: &'static str) -> Self {
        Self::Static(s)
    }
    fn new_owned(s: String) -> Self {
        Self::Shared(SharedStr::from(s))
    }
    fn as_str(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Shared(s) => s,
        }
    }
}

// ---- Error type ----

/// Error type returned when a string fails to validate as a Homie ID.
///
/// Carries the offending value and the reason why the validation failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidHomieIDError {
    /// The value that failed validation.
    pub value: String,
    /// Why the value is not a valid Homie ID.
    pub reason: &'static str,
}

impl fmt::Display for InvalidHomieIDError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Homie ID {:?}: {}", self.value, self.reason)
    }
}

impl core::error::Error for InvalidHomieIDError {}

// ---- HomieID ----

/// Represents a validated Homie ID.
///
/// A `HomieID` ensures that the ID string conforms to the Homie specification:
/// - Contains only lowercase letters `a` to `z`, numbers `0` to `9`, and hyphens `-`.
/// - Does not contain any other characters.
/// - Is not empty.
///
/// # Examples
///
/// ```
/// use homie5::HomieID;
///
/// let id = HomieID::try_from("sensor-01".to_string()).unwrap();
/// assert_eq!(id.as_str(), "sensor-01");
///
/// let id = HomieID::try_from("sensor-01").unwrap();
/// assert_eq!(id.as_str(), "sensor-01");
/// ```
#[derive(Debug, Clone)]
pub struct HomieID(HomieIDInner);

impl HomieID {
    /// Wrap a statically known string into a `HomieID`.
    ///
    /// Panics if the `id` is not a valid `HomieID`.
    pub const fn new_const(id: &'static str) -> Self {
        if let Err(reason) = Self::check(id) {
            panic!("{}", reason);
        }
        Self(HomieIDInner::new_static(id))
    }

    /// Allows borrowing the inner string slice of the `HomieID`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Validates `id` against the Homie ID rules.
    pub fn validate(id: &str) -> Result<(), InvalidHomieIDError> {
        Self::check(id).map_err(|reason| InvalidHomieIDError {
            value: id.to_owned(),
            reason,
        })
    }

    const fn check(id: &str) -> Result<(), &'static str> {
        if id.is_empty() {
            return Err("Homie ID cannot be empty");
        }
        let mut bytes = id.as_bytes();
        while let [first, remainder @ ..] = bytes {
            if !matches!(first, b'a'..=b'z' | b'0'..=b'9' | b'-') {
                return Err("Homie ID can only contain lowercase letters a-z, numbers 0-9, and hyphens (-)");
            }
            bytes = remainder;
        }
        Ok(())
    }
}

// ---- Trait implementations based on string content ----

impl PartialEq for HomieID {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for HomieID {}

impl Hash for HomieID {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for HomieID {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HomieID {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

// ---- Conversion traits ----

impl TryFrom<&'static str> for HomieID {
    type Error = InvalidHomieIDError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        HomieID::validate(value)?;
        Ok(HomieID(HomieIDInner::new_static(value)))
    }
}

impl TryFrom<String> for HomieID {
    type Error = InvalidHomieIDError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        HomieID::validate(&value)?;
        Ok(HomieID(HomieIDInner::new_owned(value)))
    }
}

impl core::str::FromStr for HomieID {
    type Err = InvalidHomieIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <String as TryInto<Self>>::try_into(s.to_string())
    }
}

// ---- Display ----

impl fmt::Display for HomieID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---- Serde ----

impl Serialize for HomieID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for HomieID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        HomieID::try_from(s).map_err(de::Error::custom)
    }
}

// ---- JsonSchema ----

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for HomieID {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        alloc::borrow::Cow::Borrowed("HomieID")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <String as schemars::JsonSchema>::json_schema(generator)
    }
}

// ---- AsNodeId ----

impl AsNodeId for HomieID {
    fn as_node_id(&self) -> &HomieID {
        self
    }
}

impl AsNodeId for &HomieID {
    fn as_node_id(&self) -> &HomieID {
        self
    }
}
