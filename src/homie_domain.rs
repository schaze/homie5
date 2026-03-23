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
//! use std::convert::TryFrom;
//!
//! let custom_domain = HomieDomain::try_from("my-brand").unwrap();
//! assert_eq!(custom_domain.to_string(), "my-brand");
//! ```
//!

use core::fmt;
use std::hash::{Hash, Hasher};

use schemars::JsonSchema;

use crate::DEFAULT_HOMIE_DOMAIN;

// ---- Feature-gated inner representation ----

#[cfg(not(feature = "legacy-cow"))]
mod domain_inner {
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    pub(super) enum CustomDomainInner {
        Static(&'static str),
        Shared(Arc<str>),
    }

    impl CustomDomainInner {
        pub(super) fn new_static(s: &'static str) -> Self {
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
mod domain_inner {
    use std::borrow::Cow;

    #[derive(Debug, Clone)]
    pub(super) struct CustomDomainInner(Cow<'static, str>);

    impl CustomDomainInner {
        pub(super) fn new_static(s: &'static str) -> Self {
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

use domain_inner::CustomDomainInner;

// ---- Error type ----

/// Error type returned when a string fails to validate as a custom homie-domain.
///
/// Provides details about why the validation failed.
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidHomieDomainError {
    details: String,
}

impl InvalidHomieDomainError {
    /// Creates a new `InvalidHomieDomainError` with a specific message.
    fn new(msg: &str) -> Self {
        Self {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for InvalidHomieDomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.details)
    }
}

impl std::error::Error for InvalidHomieDomainError {}

// ---- CustomDomain ----

#[derive(Debug, Clone)]
pub struct CustomDomain(CustomDomainInner);

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

    /// Allows borrowing the inner string slice of the `CustomDomain`.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl PartialEq for CustomDomain {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for CustomDomain {}

impl Hash for CustomDomain {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for CustomDomain {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CustomDomain {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Default for CustomDomain {
    fn default() -> Self {
        Self(CustomDomainInner::new_static(""))
    }
}

impl TryFrom<&'static str> for CustomDomain {
    type Error = InvalidHomieDomainError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        CustomDomain::validate(value)?;
        Ok(CustomDomain(CustomDomainInner::new_static(value)))
    }
}

impl TryFrom<String> for CustomDomain {
    type Error = InvalidHomieDomainError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CustomDomain::validate(&value)?;
        Ok(CustomDomain(CustomDomainInner::new_owned(value)))
    }
}

impl fmt::Display for CustomDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl serde::Serialize for CustomDomain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
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

impl JsonSchema for CustomDomain {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("CustomDomain")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <String as JsonSchema>::json_schema(generator)
    }
}

// ---- HomieDomain ----

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord, JsonSchema)]
pub enum HomieDomain {
    #[default]
    Default,
    All,
    Custom(CustomDomain),
}

impl HomieDomain {
    /// Allows borrowing the inner string slice of the `HomieDomain`.
    pub fn as_str(&self) -> &str {
        match self {
            HomieDomain::Default => DEFAULT_HOMIE_DOMAIN,
            HomieDomain::All => "+",
            HomieDomain::Custom(custom) => custom.as_str(),
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
        let s: std::borrow::Cow<'de, str> = serde::Deserialize::deserialize(deserializer)?;
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

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            DEFAULT_HOMIE_DOMAIN => Ok(HomieDomain::Default),
            "+" => Ok(HomieDomain::All),
            _ => Ok(HomieDomain::Custom(value.try_into()?)),
        }
    }
}

impl std::str::FromStr for HomieDomain {
    type Err = InvalidHomieDomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            DEFAULT_HOMIE_DOMAIN => Ok(HomieDomain::Default),
            "+" => Ok(HomieDomain::All),
            _ => Ok(HomieDomain::Custom(s.to_owned().try_into()?)),
        }
    }
}

#[test]
fn test_homie_domain() {
    assert_eq!(HomieDomain::try_from("hello").unwrap().to_string(), "hello");
    assert_eq!(HomieDomain::try_from("hello".to_string()).unwrap().to_string(), "hello");
    assert_eq!(HomieDomain::try_from("homie").unwrap(), HomieDomain::Default);
    assert_eq!(HomieDomain::try_from("+").unwrap(), HomieDomain::All);
}
