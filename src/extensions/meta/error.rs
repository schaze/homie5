use thiserror::Error;

use crate::{InvalidHomieDomainError, InvalidHomieIDError};

/// Errors that can occur when working with the `$meta` overlay extension.
#[derive(Debug, Error)]
pub enum MetaError {
    /// The overlay document payload could not be parsed as valid JSON.
    #[error("Error parsing meta overlay document")]
    InvalidDocument(#[from] serde_json::Error),

    /// The MQTT topic does not match the `$meta` extension topic pattern.
    #[error("Message for invalid $meta MQTT topic received.")]
    InvalidTopic,

    /// The MQTT payload could not be converted to a valid UTF-8 string.
    #[error(transparent)]
    PayloadConversionError(#[from] std::string::FromUtf8Error),

    /// The homie domain segment of the topic is invalid.
    #[error("Invalid homie domain: {0}")]
    InvalidHomieDomain(#[from] InvalidHomieDomainError),

    /// The provider or device ID segment of the topic is invalid.
    #[error("Invalid homie id: {0}")]
    InvalidHomieID(#[from] InvalidHomieIDError),
}
