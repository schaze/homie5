//! This module provides MQTT primitives for library-agnostic message handling.
//!
//! These types represent the fundamental building blocks of MQTT communication, such as publishing,
//! subscribing, and managing QoS levels. They serve as a common interface that can be adapted or
//! converted to the specific types used by any MQTT client library.
//!
//! # Purpose
//!
//! The types in this module are designed to be the lowest common denominator of MQTT functionality,
//! ensuring compatibility across various MQTT client libraries. By abstracting these core MQTT concepts,
//! this module allows for greater flexibility and modularity in MQTT-based applications.
//!
//! Users of the homie5 library are expected to convert these types into the corresponding types of their chosen
//! MQTT client library when performing actual MQTT operations.
//!
//! # Primitives
//!
//! The module includes:
//!
//! - `Publish`: Represents a publish message, including the topic, payload, QoS level, and retain flag.
//! - `Subscription`: Represents a subscription to a specific MQTT topic with a defined QoS level.
//! - `Unsubscribe`: Represents a request to unsubscribe from a topic.
//! - `LastWill`: Represents the "Last Will" message used to notify others in case of unexpected disconnection.
//! - `QoS`: Represents the three levels of Quality of Service in MQTT (AtMostOnce, AtLeastOnce, ExactlyOnce).
//!
//! These primitives form the backbone of MQTT communication and can be converted to their equivalents in
//! various MQTT libraries, making this module a flexible foundation for MQTT client implementations.

use std::string::FromUtf8Error;

use serde::{Deserialize, Serialize};

/// Represents the Last Will (LW) contract for an MQTT client.
///
/// The Last Will message is a feature in MQTT that ensures a device can notify others of an unexpected disconnection.
/// When a device loses connection to the MQTT broker unexpectedly, the broker will publish this "last will" message
/// on behalf of the disconnected client, typically indicating the device is "lost" or "offline".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastWill {
    /// The MQTT topic where the Last Will message will be published.
    ///
    /// Example: `"homie/5/device-1/$state"`
    ///
    /// This topic typically represents the state attribute of the device, and the Last Will message
    /// will notify subscribers that the device has gone offline or is "lost".
    pub topic: String,

    /// The payload of the Last Will message.
    ///
    /// Example: `"lost"`
    ///
    /// This payload represents the status or state of the device when an unexpected disconnection occurs.
    /// It is transmitted as a binary payload (`Vec<u8>`), which can contain text or other types of data.
    pub message: Vec<u8>,

    /// The Quality of Service (QoS) level for the Last Will message.
    ///
    /// Example: `QoS::ExactlyOnce`
    ///
    /// This QoS level dictates the level of delivery assurance for the Last Will message when published by the broker.
    pub qos: QoS,

    /// A flag indicating whether the Last Will message should be retained by the broker.
    ///
    /// Example: `true`
    ///
    /// If `retain` is set to `true`, the broker will store the Last Will message and deliver it to any future subscribers
    /// of the topic, ensuring that new subscribers are immediately informed that the device is "lost."
    pub retain: bool,
}

/// Represents the 3 MQTT QoS (Quality of Service) strategies for publishing messages.
///
/// The QoS level determines how the MQTT protocol ensures message delivery between the publisher and the broker.
/// Higher QoS levels offer greater message delivery guarantees but may also involve more overhead.
#[derive(Default, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum QoS {
    /// At most once delivery (QoS 0). The message is delivered at most once, without confirmation.
    ///
    /// - This is the fastest and least reliable QoS level.
    /// - The message is delivered with a "fire and forget" approach, and no acknowledgment is expected.
    /// - It may be lost if the connection is unstable.
    AtMostOnce,

    /// At least once delivery (QoS 1). The message is guaranteed to be delivered at least once.
    ///
    /// - The publisher will resend the message until it receives an acknowledgment from the broker.
    /// - This could result in duplicate messages being delivered to the broker if acknowledgments are delayed.
    /// - Ensures delivery but not necessarily just once.
    AtLeastOnce,

    /// Exactly once delivery (QoS 2). The message is guaranteed to be delivered exactly once.
    ///
    /// - This is the highest level of message delivery guarantee in MQTT.
    /// - The publisher and broker engage in a handshake process to ensure the message is delivered once and only once.
    /// - This QoS level introduces the most overhead and can increase latency due to the multiple confirmation steps.
    #[default]
    ExactlyOnce,
}

/// Represents an MQTT subscription to a specific topic.
///
/// Subscriptions allow clients to receive messages published to a topic.
/// Each subscription has a topic and a QoS level, which determines the Quality of Service
/// used for message delivery.
///
/// # Fields
///
/// - `topic`: The topic filter specifying which messages the client is interested in receiving.
/// - `qos`: The Quality of Service level that dictates how the broker delivers messages to the client.
#[derive(Clone, PartialEq, Eq)]
pub struct Subscription {
    /// The topic filter for the subscription.
    ///
    /// This can be a specific topic (e.g., "sensor/temperature") or a pattern that matches multiple topics (e.g., "sensor/#").
    pub topic: String,

    /// The Quality of Service (QoS) level for this subscription.
    ///
    /// Determines how reliably messages are delivered to the client.
    /// Higher QoS levels provide stronger guarantees at the cost of increased network overhead.
    pub qos: QoS,
}

/// Represents an MQTT publish message to a specific topic.
///
/// Publishing allows clients to send messages to the broker for distribution to other subscribers.
/// Each publish message contains a topic, a payload, and several additional options like retain flags and QoS levels.
///
/// # Fields
///
/// - `topic`: The topic to which the message is published.
/// - `retain`: A flag indicating whether the message should be retained by the broker.
/// - `payload`: The actual data being sent in the message, as a binary payload (vector of bytes).
/// - `qos`: The Quality of Service level, which determines the reliability of the message delivery.
#[derive(Clone, PartialEq, Eq)]
pub struct Publish {
    /// The topic for this publish message.
    ///
    /// This topic determines where the message is routed and which subscribers will receive it.
    pub topic: String,

    /// A flag indicating whether the broker should retain this message.
    ///
    /// - `true`: The broker will store this message and deliver it to new subscribers as soon as they subscribe.
    /// - `false`: The message will only be delivered to currently connected subscribers, and won't be retained for future subscribers.
    pub retain: bool,

    /// The payload of the message.
    ///
    /// This is the data being transmitted in the publish message. The payload can represent text, binary data, or other formats,
    /// depending on the application.
    pub payload: Vec<u8>,

    /// The Quality of Service (QoS) level for this publish message.
    ///
    /// This determines the reliability of message delivery between the client and the broker.
    /// Higher QoS levels offer stronger delivery guarantees but may introduce more overhead.
    pub qos: QoS,
}

/// Represents an MQTT unsubscribe request for a specific topic.
///
/// Unsubscribing from a topic stops the client from receiving messages that are published to that topic.
/// Once unsubscribed, the client will no longer receive updates from the broker for the specified topic.
#[derive(Clone, PartialEq, Eq)]
pub struct Unsubscribe {
    /// The MQTT topic the client wishes to unsubscribe from.
    ///
    /// Example: `"homie/5/device-1/sensor/temperature"`
    ///
    /// This topic specifies the exact path that the client will stop receiving messages from. After successfully
    /// unsubscribing, the client will no longer receive any further publications from the broker on this topic.
    pub topic: String,
}

/// Attempt to parse the payload as a UTF-8 string
/// special case:
/// accoring to the homie convention a string with a 0 value byte as first value constitues an
/// empty string. This is only true for homie string value types. However since all input data
/// for all other homie data types is a string we leave the handling of a valid input data to
/// the parsing function of HomieValue
pub fn mqtt_payload_to_string(payload: &[u8]) -> Result<String, FromUtf8Error> {
    if payload.first() == Some(&0) {
        Ok(String::new())
    } else {
        String::from_utf8(payload.to_vec())
    }
}
