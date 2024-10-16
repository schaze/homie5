use core::fmt;
use std::borrow::Cow;

use crate::DEFAULT_HOMIE_DOMAIN;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub enum HomieDomain {
    #[default]
    Default,
    All,
    Custom(Cow<'static, str>),
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
            other => Ok(HomieDomain::Custom(Cow::Owned(other.to_string()))),
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

impl From<&'static str> for HomieDomain {
    fn from(value: &'static str) -> Self {
        if value == DEFAULT_HOMIE_DOMAIN {
            HomieDomain::Default
        } else if value == "+" {
            HomieDomain::All
        } else {
            HomieDomain::Custom(Cow::Borrowed(value))
        }
    }
}

impl From<String> for HomieDomain {
    fn from(value: String) -> Self {
        if value == DEFAULT_HOMIE_DOMAIN {
            HomieDomain::Default
        } else if value == "+" {
            HomieDomain::All
        } else {
            HomieDomain::Custom(Cow::Owned(value))
        }
    }
}

#[test]
fn test_homie_domain() {
    assert_eq!(HomieDomain::from("hello").to_string(), "hello");
    assert_eq!(HomieDomain::from("hello".to_string()).to_string(), "hello");
    assert_eq!(HomieDomain::from("homie"), HomieDomain::Default);
    assert_eq!(HomieDomain::from("+"), HomieDomain::All);
}
