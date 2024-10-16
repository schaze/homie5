//! A module for representing and validating Homie IDs according to the Homie MQTT convention.
//!
//! This module provides the `HomieID` struct, which ensures that any ID used in the context of Homie devices,
//! nodes, and properties conforms to the specification that IDs must only contain lowercase letters `a` to `z`,
//! numbers `0` to `9`, and hyphens `-`.

use std::fmt;
use std::{borrow::Cow, convert::TryFrom};

use serde::{de, Deserialize, Deserializer, Serialize};

/// Error type returned when a string fails to validate as a Homie ID.
///
/// Provides details about why the validation failed.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidHomieIDError {
    details: String,
}

impl InvalidHomieIDError {
    /// Creates a new `InvalidHomieIDError` with a specific message.
    ///
    /// # Arguments
    ///
    /// * `msg` - A string slice that holds the error message.
    fn new(msg: &str) -> Self {
        InvalidHomieIDError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for InvalidHomieIDError {
    /// Formats the error message for display purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.details)
    }
}

impl std::error::Error for InvalidHomieIDError {}

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord, Serialize)]
pub struct HomieID(Cow<'static, str>);

impl HomieID {
    /// Allows borrowing the inner string slice of the `HomieID`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn validate(id: &str) -> Result<(), InvalidHomieIDError> {
        if id.is_empty() {
            return Err(InvalidHomieIDError::new("Homie ID cannot be empty"));
        }
        if id.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '-')) {
            Ok(())
        } else {
            Err(InvalidHomieIDError::new(
                "Homie ID can only contain lowercase letters a-z, numbers 0-9, and hyphens (-)",
            ))
        }
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
    /// use std::convert::TryFrom;
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
    /// use std::convert::TryFrom;
    ///
    /// let id = HomieID::try_from("sensor-01").unwrap();
    /// ```
    fn try_from(value: String) -> Result<Self, Self::Error> {
        HomieID::validate(&value)?;
        Ok(HomieID(Cow::Owned(value)))
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
