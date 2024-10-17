use core::fmt;
use std::borrow::Cow;

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

impl std::error::Error for InvalidHomieDomainError {}

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
    /// use std::convert::TryFrom;
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
    /// use std::convert::TryFrom;
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
    /// use std::convert::TryFrom;
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
    /// use std::convert::TryFrom;
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

#[test]
fn test_homie_domain() {
    assert_eq!(HomieDomain::try_from("hello").unwrap().to_string(), "hello");
    assert_eq!(HomieDomain::try_from("hello".to_string()).unwrap().to_string(), "hello");
    assert_eq!(HomieDomain::try_from("homie").unwrap(), HomieDomain::Default);
    assert_eq!(HomieDomain::try_from("+").unwrap(), HomieDomain::All);
}
