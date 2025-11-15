//! This module provides validation and handling for the `HomieDomain` used in the Homie 5 protocol.
//!
//! # HomieDomain
//!
//! The `HomieDomain` represents the first segment of the MQTT topic used by devices in the Homie 5 protocol.
//! By default, the domain is `"homie"`, but it can be customized to meet specific needs (e.g., using a public broker or for branding).
//! The domain must be a single topic segment without characters like `/`, `+`, or `#`. The major version segment (`"5"`) is fixed.
//!
//! - `Default`: The default domain, `"homie"`.
//! - `All`: Represents the MQTT `+` wildcard for subscribing to all domains.
//! - `Custom`: A user-defined domain validated to ensure compliance with Homie 5 rules.
//!
//! # CustomDomain
//!
//! `CustomDomain` wraps a user-provided domain and ensures it meets the required validation criteria.
//! It can be created using `TryFrom<&str>` or `TryFrom<String>`. Domains must not be empty and must be single-segment topics.
//!
//! # Errors
//!
//! `InvalidHomieDomainError` is returned when a domain fails validation due to invalid characters or an empty string.
//!
//! # Example
//!
//! ```rust
//! use homie5::*;
//! use core::convert::TryFrom;
//!
//! let custom_domain = HomieDomain::try_from("my-brand").unwrap();
//! assert_eq!(custom_domain.to_string(), "my-brand");
//! ```
//!

use core::fmt;

use alloc::{
    borrow::Cow,
    string::{String, ToString},
};

use crate::DEFAULT_HOMIE_DOMAIN;

/// Error type returned when a string fails to validate as a custom homie-domain.
///
/// Provides details about why the validation failed.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidHomieDomainError {
    details: String,
}

impl InvalidHomieDomainError {
    /// Creates a new `InvalidHomieDomainError` with a specific message.
    ///
    /// # Arguments
    ///
    /// * `msg` - A string slice that holds the error message.
    fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for InvalidHomieDomainError {
    /// Formats the error message for display purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.details)
    }
}

impl core::error::Error for InvalidHomieDomainError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord, serde::Serialize)]
pub struct CustomDomain(Cow<'static, str>);

impl CustomDomain {
    pub fn validate(id: &str) -> Result<(), InvalidHomieDomainError> {
        if id.is_empty() {
            return Err(InvalidHomieDomainError::new("HomieDomain  cannot be empty"));
        }
        if id.contains('/') || id.contains('+') || id.contains('#') {
            return Err(InvalidHomieDomainError::new(
                "The homie-domain must be a single segment topic.",
            ));
        }

        Ok(())
    }
}

impl TryFrom<&'static str> for CustomDomain {
    type Error = InvalidHomieDomainError;

    /// Attempts to create a `CustomDomain` from a `&str`, returning an error if validation fails.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the custom homie-domain to be validated.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidHomieDomainError` if the input string does not conform to the homie5 specifications.
    ///
    /// # Examples
    ///
    /// ```
    /// use homie5::CustomDomain;
    /// use core::convert::TryFrom;
    ///
    /// let id = CustomDomain::try_from("homie-dev").unwrap();
    /// ```
    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        CustomDomain::validate(value)?;
        Ok(CustomDomain(Cow::Borrowed(value)))
    }
}

impl TryFrom<String> for CustomDomain {
    type Error = InvalidHomieDomainError;

    /// Attempts to create a `CustomDomain` from an owned `string`, returning an error if validation fails.
    ///
    /// # Arguments
    ///
    /// * `value` - A string that holds the custom homie-domain to be validated.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidHomieDomainError` if the input string does not conform to the homie5 specifications.
    ///
    /// # Examples
    ///
    /// ```
    /// ```
    /// use homie5::CustomDomain;
    /// use core::convert::TryFrom;
    ///
    /// let id = CustomDomain::try_from("homie-dev".to_string()).unwrap();
    /// ```
    fn try_from(value: String) -> Result<Self, Self::Error> {
        CustomDomain::validate(&value)?;
        Ok(CustomDomain(Cow::Owned(value)))
    }
}

impl fmt::Display for CustomDomain {
    /// Formats the `CustomDomain` as a string for display purposes.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for CustomDomain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        CustomDomain::try_from(s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub enum HomieDomain {
    #[default]
    Default,
    All,
    Custom(CustomDomain),
}

impl HomieDomain {
    /// Allows borrowing the inner string slice of the `HomieID`.
    pub fn as_str(&self) -> &str {
        match self {
            HomieDomain::Default => DEFAULT_HOMIE_DOMAIN,
            HomieDomain::All => "+",
            HomieDomain::Custom(custom) => &custom.0,
        }
    }
}

// Implement Serialize manually to use the Display trait's output
impl serde::Serialize for HomieDomain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for HomieDomain {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Cow<'de, str> = serde::Deserialize::deserialize(deserializer)?;
        match s.as_ref() {
            DEFAULT_HOMIE_DOMAIN => Ok(HomieDomain::Default),
            "+" => Ok(HomieDomain::All),
            _ => Ok(HomieDomain::Custom(
                s.to_string().try_into().map_err(serde::de::Error::custom)?,
            )),
        }
    }
}

impl fmt::Display for HomieDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HomieDomain::Default => write!(f, "{}", DEFAULT_HOMIE_DOMAIN),
            HomieDomain::All => write!(f, "+"),
            HomieDomain::Custom(value) => write!(f, "{}", value),
        }
    }
}

impl TryFrom<&'static str> for HomieDomain {
    type Error = InvalidHomieDomainError;

    /// Attempts to create a `HomieDomain` from a `&str`, returning an error if validation fails.
    ///
    /// # Arguments
    ///
    /// * `value` - A string slice that holds the custom homie-domain to be validated.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidHomieDomainError` if the input string does not conform to the homie5 specifications.
    ///
    /// # Examples
    ///
    /// ```
    /// use homie5::HomieDomain;
    /// use core::convert::TryFrom;
    ///
    /// let id = HomieDomain::try_from("homie-dev").unwrap();
    /// ```
    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        match value {
            DEFAULT_HOMIE_DOMAIN => Ok(HomieDomain::Default),
            "+" => Ok(HomieDomain::All),
            _ => Ok(HomieDomain::Custom(value.try_into()?)),
        }
    }
}

impl TryFrom<String> for HomieDomain {
    type Error = InvalidHomieDomainError;

    /// Attempts to create a `HomieDomain` from an owned `string`, returning an error if validation fails.
    ///
    /// # Arguments
    ///
    /// * `value` - A string that holds the custom homie-domain to be validated.
    ///
    /// # Errors
    ///
    /// Returns an `InvalidHomieDomainError` if the input string does not conform to the homie5 specifications.
    ///
    /// # Examples
    ///
    /// ```
    /// ```
    /// use homie5::HomieDomain;
    /// use core::convert::TryFrom;
    ///
    /// let id = HomieDomain::try_from("homie-dev".to_string()).unwrap();
    /// ```
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            DEFAULT_HOMIE_DOMAIN => Ok(HomieDomain::Default),
            "+" => Ok(HomieDomain::All),
            _ => Ok(HomieDomain::Custom(value.try_into()?)),
        }
    }
}

// impl FromStr for HomieDomain {
//     type Err = InvalidHomieDomainError;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             DEFAULT_HOMIE_DOMAIN => Ok(HomieDomain::Default),
//             "+" => Ok(HomieDomain::All),
//             _ => Ok(HomieDomain::Custom(s.to_owned().try_into()?)),
//         }
//     }
// }
//
#[test]
fn test_homie_domain() {
    assert_eq!(HomieDomain::try_from("hello").unwrap().to_string(), "hello");
    assert_eq!(HomieDomain::try_from("hello".to_string()).unwrap().to_string(), "hello");
    assert_eq!(HomieDomain::try_from("homie").unwrap(), HomieDomain::Default);
    assert_eq!(HomieDomain::try_from("+").unwrap(), HomieDomain::All);
}
