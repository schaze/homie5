//! A module for representing and validating Homie IDs according to the Homie MQTT convention.
//!
//! This module provides the `HomieID` struct, which ensures that any ID used in the context of Homie devices,
//! nodes, and properties conforms to the specification that IDs must only contain lowercase letters `a` to `z`,
//! numbers `0` to `9`, and hyphens `-`.

use std::convert::TryFrom;
use std::fmt;

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
/// let id = HomieID::new("sensor-01").unwrap();
/// assert_eq!(id.as_ref(), "sensor-01");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct HomieID(String);

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

impl HomieID {
    /// Creates a new `HomieID` after validating the input string.
    ///
    /// # Arguments
    ///
    /// * `id` - A string slice that holds the ID to be validated.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidHomieIDError` if the input string does not conform to the Homie ID specifications.
    ///
    /// # Examples
    ///
    /// ```
    /// use homie5::HomieID;
    ///
    /// let valid_id = HomieID::new("device-01");
    /// assert!(valid_id.is_ok());
    ///
    /// let invalid_id = HomieID::new("Device_01");
    /// assert!(invalid_id.is_err());
    /// ```
    pub fn new(id: &str) -> Result<Self, InvalidHomieIDError> {
        if id.is_empty() {
            return Err(InvalidHomieIDError::new("Homie ID cannot be empty"));
        }
        if id.chars().all(|c| matches!(c, 'a'..='z' | '0'..='9' | '-')) {
            Ok(HomieID(id.to_string()))
        } else {
            Err(InvalidHomieIDError::new(
                "Homie ID can only contain lowercase letters a-z, numbers 0-9, and hyphens (-)",
            ))
        }
    }

    pub fn new_unchecked(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl TryFrom<&str> for HomieID {
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
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        HomieID::new(value)
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

impl AsRef<str> for HomieID {
    /// Allows borrowing the inner string slice of the `HomieID`.
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    //! Tests for the `HomieID` struct and associated functionality.

    use super::*;

    #[test]
    fn test_valid_homie_id() {
        let id = HomieID::new("valid-id-123");
        assert!(id.is_ok());
        assert_eq!(id.unwrap().as_ref(), "valid-id-123");
    }

    #[test]
    fn test_invalid_homie_id_uppercase() {
        let id = HomieID::new("InvalidID");
        assert!(id.is_err());
    }

    #[test]
    fn test_invalid_homie_id_special_chars() {
        let id = HomieID::new("invalid$id!");
        assert!(id.is_err());
    }

    #[test]
    fn test_empty_homie_id() {
        let id = HomieID::new("");
        assert!(id.is_err());
    }
}
