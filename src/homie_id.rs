//! This module defines and validates Homie IDs used in the Homie MQTT convention.
//!
//! # HomieID
//!
//! `HomieID` ensures that the ID strings for devices, nodes, and properties follow the Homie specification:
//! - IDs must only include lowercase letters (`a-z`), digits (`0-9`), and hyphens (`-`).
//! - IDs must not be empty or contain any other characters.
//!
//! A `HomieID` can be created via `TryFrom<&'static str>` or `TryFrom<String>`. The `'static` lifetime is used for string slices to ensure the ID can be safely sent across threads or through channels, where the ownership or lifetime of the data must be guaranteed for the duration of the program if needed.
//!
//! # Why Only `&'static str`?
//!
//! The use of `&'static str` ensures that any string slice used to create a `HomieID` has a lifetime that is valid for the entire runtime of the program. This is particularly important because IDs will be passed between different threads (e.g., through channels), and allowing a non-`'static` lifetime would risk referencing invalid or deallocated memory.
//!
//! By using `Cow<'static, str>`, `HomieID` can either hold an owned `String` or a borrowed `&'static str`, providing flexibility while ensuring thread safety when the ID is shared or sent across channels.
//!
//! # Errors
//!
//! If an ID fails to meet the specifications, an `InvalidHomieIDError` is returned with a message indicating the issue.
//!
//! # Examples
//!
//! ```rust
//! use homie5::*;
//! use core::convert::TryFrom;
//!
//! let valid_id = HomieID::try_from("device-01").unwrap();
//! assert_eq!(valid_id.as_str(), "device-01");
//!
//! let invalid_id = HomieID::try_from("Device-01"); // Returns an error due to uppercase letter
//! assert!(invalid_id.is_err());
//! ```

use core::convert::TryFrom;
use core::fmt;
use std::borrow::Cow;

use serde::{de, Deserialize, Deserializer, Serialize};

use crate::AsNodeId;

/// Error type returned when a string fails to validate as a Homie ID.
///
/// Provides details about why the validation failed.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidHomieIDError {
    details: &'static str,
}

impl InvalidHomieIDError {
    /// Creates a new `InvalidHomieIDError` with a specific message.
    ///
    /// # Arguments
    ///
    /// * `msg` - A string slice that holds the error message.
    const fn new(msg: &'static str) -> Self {
        InvalidHomieIDError { details: msg }
    }
}

impl fmt::Display for InvalidHomieIDError {
    /// Formats the error message for display purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.details)
    }
}

impl core::error::Error for InvalidHomieIDError {}

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct HomieID(Cow<'static, str>);

impl HomieID {
    /// Wrap a statically known string into a `HomieID`.
    ///
    /// Panics if the `id` is not a valid `HomieID`.
    pub const fn new_const(id: &'static str) -> Self {
        if let Err(e) = Self::validate(id) {
            panic!("{}", e.details);
        }
        Self(Cow::Borrowed(id))
    }

    /// Allows borrowing the inner string slice of the `HomieID`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const fn validate(id: &str) -> Result<(), InvalidHomieIDError> {
        if id.is_empty() {
            return Err(InvalidHomieIDError::new("Homie ID cannot be empty"));
        }
        // Since IDs may only be ASCII it is fine to iterate over the bytes of this ID, rather than
        // trying to decode full characters. Unfortunately the following code is the only thing we
        // can do for const functions.
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

impl TryFrom<&'static str> for HomieID {
    type Error = InvalidHomieIDError;

    /// Attempts to create a `HomieID` from a `&str`, returning an error if validation fails.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the ID to be validated.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidHomieIDError` if the input string does not conform to the Homie ID specifications.
    ///
    /// # Examples
    ///
    /// ```
    /// use homie5::HomieID;
    /// use core::convert::TryFrom;
    ///
    /// let id = HomieID::try_from("sensor-01").unwrap();
    /// ```
    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        HomieID::validate(value)?;
        Ok(HomieID(Cow::Borrowed(value)))
    }
}

impl TryFrom<String> for HomieID {
    type Error = InvalidHomieIDError;

    /// Attempts to create a `HomieID` from a `&str`, returning an error if validation fails.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the ID to be validated.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidHomieIDError` if the input string does not conform to the Homie ID specifications.
    ///
    /// # Examples
    ///
    /// ```
    /// use homie5::HomieID;
    /// use core::convert::TryFrom;
    ///
    /// let id = HomieID::try_from("sensor-01").unwrap();
    /// ```
    fn try_from(value: String) -> Result<Self, Self::Error> {
        HomieID::validate(&value)?;
        Ok(HomieID(Cow::Owned(value)))
    }
}

impl core::str::FromStr for HomieID {
    type Err = InvalidHomieIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <String as TryInto<Self>>::try_into(s.to_string())
    }
}

impl fmt::Display for HomieID {
    /// Formats the `HomieID` as a string for display purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
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
