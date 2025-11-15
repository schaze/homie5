//! Defines all errors the homie 5 library recognizes
//!

use alloc::string::{FromUtf8Error, String};

use thiserror::Error;

use crate::{Homie5ValueConversionError, InvalidHomieDomainError, InvalidHomieIDError};

/// Represents various errors that can occur while handling the Homie v5 protocol.
///
/// This error enum is used to encapsulate all possible errors encountered during
/// interaction with devices and MQTT messages under the Homie v5 protocol. It
/// covers errors related to subscribing, publishing, message formatting, and more.
#[derive(Debug, Error)]
pub enum Homie5ProtocolError {
    /// Error occurred while subscribing to a topic.
    #[error("Error Subscribing to topic")]
    SubscribeError,

    /// An unknown error occurred within the MQTT client.
    #[error("Unknown MqttClient Error.")]
    MqttClientError,

    /// Error occurred while unsubscribing from a topic.
    #[error("Error Unsubscribing to topic.")]
    UnsubscribeError,

    /// Error occurred while sending a set command to a device.
    #[error("Error sending set command")]
    SetCommandError,

    /// An MQTT message was received for a topic that does not conform to the Homie convention.
    #[error("Message for invalid homie MQTT topic received.")]
    InvalidTopic,

    /// Error occurred while converting a payload from bytes to UTF-8.
    ///
    /// This typically happens when the payload contains invalid UTF-8 bytes.
    #[error(transparent)]
    PayloadConversionError(#[from] FromUtf8Error),

    /// The device description received is invalid and could not be parsed.
    #[error("Cannot parse DeviceDescription. Invalid format.")]
    InvalidDeviceDescription,

    /// An invalid message payload was received.
    #[error("Invalid message payload received.")]
    InvalidPayload,

    /// The root topic of the publish request does not match the expected Homie root topic.
    #[error("Publish request for wrong homie root topic.")]
    RootMismatch,

    /// A root device has a non-empty root attribute, which is not allowed.
    ///
    /// Root devices must have an empty root attribute in the Homie convention.
    #[error("Root device cannot refer to another root device. root attribute must be empty.")]
    NonEmptyRootForRootDevice,

    /// The requested property could not be found in the device description.
    #[error("The requested property does not exist in the device description.")]
    PropertyNotFound,

    /// The datatype of a property is invalid according to the Homie specification.
    #[error("Invalid homie datatype.")]
    InvalidHomieDataType,

    /// The data provided does not confirm to the homie specification for a homie id
    #[error("Invalid homie id: {0}")]
    InvalidHomieID(#[from] InvalidHomieIDError),

    /// The device state is invalid. Valid states are: "init", "ready", "disconnected", "sleeping", and "lost".
    ///
    /// The provided invalid state is included in the error message.
    #[error("Invalid device state: [{0}]! Only: init, ready, disconnected, sleeping and lost are allowed.")]
    InvalidDeviceState(String),

    /// The data provided does not confirm to the homie specification for a homie-domain
    #[error("Invalid homie domain: {0}")]
    InvalidHomieDomain(#[from] InvalidHomieDomainError),

    /// The data provided could not be parsed into a HomieValue
    #[error("Invalid homie value: {0}")]
    InvalidHomieValue(#[from] Homie5ValueConversionError),

    /// Invalid Device log level
    #[error("Invalid device log level: {0}")]
    InvalidDeviceLogLevel(String),
}
