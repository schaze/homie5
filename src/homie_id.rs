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
//! By default, `HomieID` uses an `Arc<str>`-based inner representation that makes cloning O(1)
//! (atomic reference count increment instead of string copy). This is ideal when IDs are cloned
//! frequently (e.g., as HashMap keys in reference types).
//!
//! Enable the `legacy-cow` feature to use the original `Cow<'static, str>` backend, which offers
//! zero-cost construction from `&'static str` but O(n) cloning for owned strings.
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

use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};

use schemars::JsonSchema;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::AsNodeId;

// ---- Feature-gated inner representation ----

#[cfg(not(feature = "legacy-cow"))]
mod id_inner {
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    pub(super) enum HomieIDInner {
        Static(&'static str),
        Shared(Arc<str>),
    }

    impl HomieIDInner {
        pub(super) const fn new_static(s: &'static str) -> Self {
            Self::Static(s)
        }
        pub(super) fn new_owned(s: String) -> Self {
            Self::Shared(Arc::from(s.as_str()))
        }
        pub(super) fn as_str(&self) -> &str {
            match self {
                Self::Static(s) => s,
                Self::Shared(s) => s,
            }
        }
    }
}

#[cfg(feature = "legacy-cow")]
mod id_inner {
    use std::borrow::Cow;

    #[derive(Debug, Clone)]
    pub(super) struct HomieIDInner(Cow<'static, str>);

    impl HomieIDInner {
        pub(super) const fn new_static(s: &'static str) -> Self {
            Self(Cow::Borrowed(s))
        }
        pub(super) fn new_owned(s: String) -> Self {
            Self(Cow::Owned(s))
        }
        pub(super) fn as_str(&self) -> &str {
            &self.0
        }
    }
}

use id_inner::HomieIDInner;

// ---- Error type ----

/// Error type returned when a string fails to validate as a Homie ID.
///
/// Provides details about why the validation failed.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidHomieIDError {
    details: &'static str,
}

impl InvalidHomieIDError {
    /// Creates a new `InvalidHomieIDError` with a specific message.
    const fn new(msg: &'static str) -> Self {
        InvalidHomieIDError { details: msg }
    }
}

impl fmt::Display for InvalidHomieIDError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.details)
    }
}

impl std::error::Error for InvalidHomieIDError {}

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
        if let Err(e) = Self::validate(id) {
            panic!("{}", e.details);
        }
        Self(HomieIDInner::new_static(id))
    }

    /// Allows borrowing the inner string slice of the `HomieID`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub const fn validate(id: &str) -> Result<(), InvalidHomieIDError> {
        if id.is_empty() {
            return Err(InvalidHomieIDError::new("Homie ID cannot be empty"));
        }
        let mut bytes = id.as_bytes();
        while !bytes.is_empty() {
            let [b'a'..=b'z' | b'0'..=b'9' | b'-', remainder @ ..] = bytes else {
                return Err(InvalidHomieIDError::new(
                    "Homie ID can only contain lowercase letters a-z, numbers 0-9, and hyphens (-)",
                ));
            };
            bytes = remainder;
            continue;
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
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HomieID {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
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

impl std::str::FromStr for HomieID {
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

impl JsonSchema for HomieID {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("HomieID")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <String as JsonSchema>::json_schema(generator)
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
