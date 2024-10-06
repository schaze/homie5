use thiserror::Error;

#[derive(Debug, Error)]
pub enum Homie5ProtocolError {
    #[error("Error Subscribing to topic")]
    SubscribeError,
    #[error("Unknown MqttClient Error.")]
    MqttClientError,
    #[error("Error Unsubscribing to topic.")]
    UnsubscribeError,
    #[error("Error sending set command")]
    SetCommandError,
    #[error("Message for invalid homie MQTT topic received.")]
    InvalidTopic,
    #[error(transparent)]
    PayloadConversionError(#[from] std::string::FromUtf8Error),
    #[error("Cannot parse DeviceDescription. Invalid format.")]
    InvalidDeviceDescription,
    #[error("Invalid message payload received.")]
    InvalidPayload,
    #[error("Publish request for wrong homie root topic.")]
    RootMismatch,
    #[error("Root device cannot refer to another root device. root attribute must be empty.")]
    NonEmptyRootForRootDevice,
    #[error("The requested property does not exist in the device description.")]
    PropertyNotFound,
}
