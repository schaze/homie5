//! This module defines the `Homie5Message` enum, which represents all possible MQTT message types
//! according to the Homie 5 protocol. The module also includes the `parse_mqtt_message` function
//! that processes incoming MQTT messages and converts them into the appropriate `Homie5Message` variant.
//!
//! # Overview
//!
//! The Homie 5 protocol is an MQTT-based standard for IoT devices, facilitating structured communication
//! between devices and the broker. Devices publish their attributes, state changes, logs, and property
//! values to specific topics under the Homie topic hierarchy. The `Homie5Message` enum encapsulates
//! the various types of MQTT messages a device may publish or receive in this protocol.
//!
//! This module includes:
//!
//! - `Homie5Message`: An enum representing different types of Homie 5 messages such as `DeviceState`,
//!   `PropertyValue`, and `DeviceLog`.
//! - `parse_mqtt_message`: A function to parse raw MQTT messages into `Homie5Message` based on the
//!   message's topic and payload structure.
//!
//! # Homie5Message Enum
//!
//! The `Homie5Message` enum provides variants for different message types. Each variant corresponds to
//! a specific interaction, such as receiving a device's state, description, or property value, or handling
//! a command like setting a property's value or processing a broadcast message.
//!
//! - `DeviceState`: Indicates a device's current state.
//! - `DeviceRemoval`: Represents the removal of a device from the network (empty `$state`).
//! - `DeviceDescription`: Provides metadata about the device.
//! - `DeviceDescriptionCleared`: The device description was deleted (empty `$description`).
//! - `DeviceLog`: Contains log messages for debugging purposes.
//! - `DeviceAlert`: Represents critical alerts from the device.
//! - `DeviceAlertCleared`: An alert was cleared (empty `$alert/<alert-id>`).
//! - `PropertyValue`: Contains the current value of a property.
//! - `PropertyValueCleared`: A retained property value was deleted.
//! - `PropertyTarget`: Represents the desired state of a property.
//! - `PropertyTargetCleared`: A retained property target was deleted.
//! - `PropertySet`: A command to set a property to a specific value.
//! - `Broadcast`: Represents general-purpose communication broadcast messages.
//!
//! # Empty payloads
//!
//! MQTT distinguishes a zero-length payload, which deletes a retained topic, from the single
//! `0x00` byte Homie uses for an empty string. Deletions are reported through the dedicated
//! `*Cleared` / `DeviceRemoval` variants; a `""` value always means a real empty string.
//!
//! # Parsing MQTT Messages
//!
//! The `parse_mqtt_message` function is responsible for parsing MQTT messages received from a broker
//! and converting them into appropriate `Homie5Message` enum variants. This function analyzes the topic
//! structure, which follows the Homie 5 protocol conventions, and decodes the payload accordingly.
//!
//! - The topic is split into components to determine the type of message.
//! - Based on the topic structure (e.g., `homie/5/<device-id>/$state`, `homie/5/<device-id>/<node-id>/<property-id>`),
//!   the function attempts to parse the payload and return the corresponding `Homie5Message`.
//!
//! # Errors
//!
//! The `parse_mqtt_message` function may return a `Homie5ProtocolError` if the message does not conform
//! to the expected topic structure or if the payload is invalid. Possible errors include:
//!
//! - `InvalidTopic`: The topic format is not valid according to the Homie 5 protocol.
//! - `InvalidPayload`: The payload is malformed or cannot be parsed as expected.
//!
//! # Example
//!
//! ```rust
//! use homie5::*;
//! let topic = "homie/5/device1/$state";
//! let payload = b"ready";
//! let message = parse_mqtt_message(topic, payload).unwrap();
//! match message {
//!     Homie5Message::DeviceState { device, state } => {
//!         println!("Device {} is in state: {:?}", device.device_id(), state);
//!     }
//!     _ => panic!("Unexpected message type"),
//! }
//! ```
//!
//! In this example, the topic and payload represent a device state message where `device1` is in the "ready" state.
//! The `parse_mqtt_message` function successfully parses the message, and the resulting `Homie5Message` enum variant
//! is used to handle the message.

use crate::prelude::*;
use crate::{
    client::mqtt_payload_to_string,
    device_description::{DescriptionIssue, HomieDeviceDescription},
    error::Homie5ProtocolError,
    DeviceLogLevel, DeviceRef, HomieDeviceStatus, HomieDomain, HomieID, NodeRef, PropertyRef, HOMIE_VERSION,
};

/// Represents all possible MQTT message types according to the Homie 5 protocol.
/// These messages define the interactions between devices, their attributes, and the broker.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Homie5Message {
    /// A device state message has been received.
    ///
    /// These messages are published under `homie/5/<device-id>/$state` and inform
    /// about the current state of the device (e.g., "init", "ready", "disconnected").
    DeviceState {
        /// The device identifier for which the state was received.
        device: DeviceRef,
        /// The received state, which can be "init", "ready", "disconnected", "sleeping", or "lost".
        state: HomieDeviceStatus,
    },

    /// A device removal message has been received, indicating the device has been permanently removed from the network.
    ///
    /// This message represents the process of clearing all retained messages for a device from the MQTT broker,
    /// starting with a zero-length payload published to the `$state` topic. Afterward, other retained attributes
    /// and property values must also be cleared. This effectively removes the device from the MQTT ecosystem.
    DeviceRemoval {
        /// The device identifier for the device that was removed.
        device: DeviceRef,
    },

    /// A message describing a device has been received.
    ///
    /// Device descriptions are sent when a device publishes its metadata (such as its name,
    /// version, and available nodes) under `homie/5/<device-id>/$description`.
    ///
    /// Invalid nodes and properties are dropped from the description and reported in `issues`.
    DeviceDescription {
        /// The device identifier for which the description was received.
        device: DeviceRef,
        /// The device description, including metadata like nodes and properties.
        description: HomieDeviceDescription,
        /// Nodes and properties that were dropped because they are invalid.
        issues: Vec<DescriptionIssue>,
    },

    /// The device description was deleted (zero-length payload on `$description`).
    ///
    /// This is sent as part of a device removal, after the `$state` topic has been cleared.
    DeviceDescriptionCleared {
        /// The device identifier whose description was deleted.
        device: DeviceRef,
    },

    /// A log message from the device.
    ///
    /// Devices can publish logs for debugging or informational purposes to the topic
    /// `homie/5/<device-id>/$log/<level>`. These logs can help monitor device activity or identify issues.
    DeviceLog {
        /// The device identifier from which the log was received.
        device: DeviceRef,
        /// The log level under which the message is published
        level: DeviceLogLevel,
        /// The log message from the device.
        log_msg: String,
    },

    /// An alert message from the device.
    ///
    /// Alerts are published under `homie/5/<device-id>/$alert/<alert-id>` and represent critical
    /// notifications about the device’s state (e.g., sensor failures, errors that need human intervention).
    DeviceAlert {
        /// The device identifier from which the alert was received.
        device: DeviceRef,
        /// A unique identifier for the alert.
        alert_id: HomieID,
        /// The alert message providing details about the issue.
        alert_msg: String,
    },

    /// An alert was cleared (zero-length payload on `$alert/<alert-id>`).
    DeviceAlertCleared {
        /// The device identifier from which the alert was cleared.
        device: DeviceRef,
        /// The identifier of the cleared alert.
        alert_id: HomieID,
    },

    /// A property value message has been received.
    ///
    /// Property values are typically sensor readings or other dynamic values
    /// sent under `homie/5/<device-id>/<node-id>/<property-id>`.
    PropertyValue {
        /// The property identifier indicating which device/node/property the value belongs to.
        property: PropertyRef,
        /// The actual value of the property (e.g., "21.5" for a temperature sensor).
        value: String,
    },

    /// A retained property value was deleted (zero-length payload).
    PropertyValueCleared {
        /// The property whose value was deleted.
        property: PropertyRef,
    },

    /// A property target message has been received.
    ///
    /// The `$target` attribute describes an intended state change for a property and is published
    /// under `homie/5/<device-id>/<node-id>/<property-id>/$target`. It represents the desired value.
    PropertyTarget {
        /// The property identifier indicating which device/node/property the target is set for.
        property: PropertyRef,
        /// The intended target value for the property.
        target: String,
    },

    /// A retained property target was deleted (zero-length payload on `$target`).
    PropertyTargetCleared {
        /// The property whose target was deleted.
        property: PropertyRef,
    },

    /// A property set message has been received.
    ///
    /// These messages represent commands to set a property to a specific value and are sent under
    /// `homie/5/<device-id>/<node-id>/<property-id>/set`.
    PropertySet {
        /// The property identifier for which the value is being set.
        property: PropertyRef,
        /// The value to which the property is being set.
        set_value: String,
    },

    /// A broadcast message has been received.
    ///
    /// Broadcasts are messages that do not belong to a specific device but are sent under
    /// `homie/5/$broadcast/<subtopic>` to all listeners. These messages are used for general-purpose communication.
    Broadcast {
        /// The root topic of the broadcast.
        homie_domain: HomieDomain,
        /// The subtopic of the broadcast.
        subtopic: String,
        /// The broadcasted data.
        data: String,
    },
}

/// Parses an incoming MQTT message into a `Homie5Message`.
///
/// This function analyzes the topic structure and payload of an MQTT message according
/// to the Homie 5 protocol. It converts the raw MQTT message into the appropriate `Homie5Message`
/// enum variant, such as `DeviceState`, `DeviceDescription`, `DeviceLog`, or others.
///
/// Zero-length payloads on retained topics are reported as the corresponding deletion variant
/// (`DeviceRemoval`, `DeviceDescriptionCleared`, `DeviceAlertCleared`, `PropertyValueCleared`,
/// `PropertyTargetCleared`).
///
/// Devices only receive `/set` and `$broadcast` messages and can use [`parse_device_message`]
/// instead, which keeps the device description parser out of their firmware.
///
/// # Arguments
///
/// - `topic`: The MQTT topic string, which must follow the Homie topic conventions (e.g., "homie/5/device_id/node_id/property_id").
/// - `payload`: The message payload as a byte slice, which is parsed as UTF-8 where appropriate.
///
/// # Returns
///
/// - `Ok(Homie5Message)`: On successful parsing into one of the Homie 5 message types.
/// - `Err(Homie5ProtocolError)`: If the message topic or payload is invalid according to Homie 5 conventions.
///
/// # Errors
///
/// - Returns `Homie5ProtocolError::InvalidTopic` if the topic format is incorrect.
/// - Returns `Homie5ProtocolError::InvalidPayload` if the payload is malformed or unexpected for the given message type.
/// - Returns `Homie5ProtocolError::InvalidDeviceDescription` if the device-level attributes of a
///   `$description` are invalid.
///
/// # Example
/// ```rust
/// use homie5::parse_mqtt_message;
/// let topic = "homie/5/device1/$state";
/// let payload = b"ready";
/// let message = parse_mqtt_message(topic, payload).unwrap();
/// ```
pub fn parse_mqtt_message(topic: &str, payload: &[u8]) -> Result<Homie5Message, Homie5ProtocolError> {
    let (homie_domain, tokens) = split_topic(topic)?;

    // Broadcasts and property `/set` commands are parsed exactly as for devices
    if let Some(message) = parse_device_topic(&homie_domain, &tokens, payload)? {
        return Ok(message);
    }

    // check the homie id provided
    let device = DeviceRef::new(homie_domain, tokens[2].to_string().try_into()?);
    let deleted = payload.is_empty();

    // Match the topic length to identify the message type
    // len: 0    1  2     3        4       5       6
    // topic: homie/5/device_id/node_id/prop_id/$target
    // topic: homie/5/device_id/$state
    // index:    0  1     2        3       4       5
    match tokens.len() {
        4 => match tokens[3] {
            // Empty payload signifies device removal
            "$state" if deleted => Ok(Homie5Message::DeviceRemoval { device }),
            "$state" => {
                let state = mqtt_payload_to_string(payload)?
                    .try_into()
                    .map_err(|_| Homie5ProtocolError::InvalidPayload)?;
                Ok(Homie5Message::DeviceState { device, state })
            }
            "$description" if deleted => Ok(Homie5Message::DeviceDescriptionCleared { device }),
            "$description" => {
                let (description, issues) =
                    HomieDeviceDescription::from_json_lenient(&mqtt_payload_to_string(payload)?)?;
                Ok(Homie5Message::DeviceDescription {
                    device,
                    description,
                    issues,
                })
            }
            _ => Err(Homie5ProtocolError::InvalidTopic),
        },
        5 => match tokens[3] {
            // Handle alert messages (e.g. "device-id/$alert/alert-id")
            "$alert" => {
                let alert_id = HomieID::try_from(tokens[4].to_owned())?;
                if deleted {
                    Ok(Homie5Message::DeviceAlertCleared { device, alert_id })
                } else {
                    Ok(Homie5Message::DeviceAlert {
                        device,
                        alert_id,
                        alert_msg: mqtt_payload_to_string(payload)?,
                    })
                }
            }
            // Handle the "$log" attribute
            "$log" => Ok(Homie5Message::DeviceLog {
                device,
                level: DeviceLogLevel::try_from(tokens[4])?,
                log_msg: mqtt_payload_to_string(payload)?,
            }),
            // Handle property values (e.g. "device-id/node-id/prop-id")
            _ => {
                let property = PropertyRef::from_node(
                    NodeRef::from_device(device, HomieID::try_from(tokens[3].to_string())?),
                    HomieID::try_from(tokens[4].to_string())?,
                );
                if deleted {
                    Ok(Homie5Message::PropertyValueCleared { property })
                } else {
                    Ok(Homie5Message::PropertyValue {
                        property,
                        value: mqtt_payload_to_string(payload)?,
                    })
                }
            }
        },
        6 => {
            // Handle property attributes (e.g. "device-id/node-id/prop-id/$target")
            let property = PropertyRef::from_node(
                NodeRef::from_device(device, HomieID::try_from(tokens[3].to_string())?),
                HomieID::try_from(tokens[4].to_string())?,
            );
            match tokens[5] {
                // Handle the "$target" attribute
                "$target" if deleted => Ok(Homie5Message::PropertyTargetCleared { property }),
                "$target" => Ok(Homie5Message::PropertyTarget {
                    property,
                    target: mqtt_payload_to_string(payload)?,
                }),
                _ => Err(Homie5ProtocolError::InvalidTopic),
            }
        }
        _ => Err(Homie5ProtocolError::InvalidTopic),
    }
}

/// Parses an MQTT message received by a device: a property `/set` command or a `$broadcast`.
///
/// Devices only subscribe to these two kinds of topics. Unlike [`parse_mqtt_message`], this
/// function does not handle `$description` and the other topics a controller receives, so the
/// device description parser is not linked into device firmware. For `/set` and `$broadcast`
/// topics it returns exactly what [`parse_mqtt_message`] returns.
///
/// # Errors
///
/// - Returns `Homie5ProtocolError::InvalidTopic` for every other topic.
/// - Returns the same errors as [`parse_mqtt_message`] for invalid IDs or payloads.
///
/// # Example
/// ```rust
/// use homie5::{parse_device_message, Homie5Message};
/// let message = parse_device_message("homie/5/device1/light/state/set", b"true").unwrap();
/// assert!(matches!(message, Homie5Message::PropertySet { .. }));
/// ```
pub fn parse_device_message(topic: &str, payload: &[u8]) -> Result<Homie5Message, Homie5ProtocolError> {
    let (homie_domain, tokens) = split_topic(topic)?;
    parse_device_topic(&homie_domain, &tokens, payload)?.ok_or(Homie5ProtocolError::InvalidTopic)
}

/// Splits a topic into its segments and checks the `<domain>/5/` prefix.
fn split_topic(topic: &str) -> Result<(HomieDomain, Vec<&str>), Homie5ProtocolError> {
    // Split the topic into components based on '/' delimiter
    let tokens: Vec<&str> = topic.split('/').collect();

    // Ensure the topic contains at least 4 tokens (e.g. "homie/5/device-id/$state")
    if tokens.len() <= 3 {
        return Err(Homie5ProtocolError::InvalidTopic);
    }

    // Ensure homie version matches to supported version
    if tokens[1] != HOMIE_VERSION {
        return Err(Homie5ProtocolError::InvalidTopic);
    }

    let homie_domain: HomieDomain = tokens[0].to_owned().try_into()?;
    Ok((homie_domain, tokens))
}

/// Parses the topics a device receives (`$broadcast` and property `/set`). Returns `None` for
/// all other topics.
fn parse_device_topic(
    homie_domain: &HomieDomain,
    tokens: &[&str],
    payload: &[u8],
) -> Result<Option<Homie5Message>, Homie5ProtocolError> {
    // Handle broadcast messages (e.g. "homie/5/$broadcast")
    if tokens[2] == "$broadcast" {
        return Ok(Some(Homie5Message::Broadcast {
            homie_domain: homie_domain.clone(),
            subtopic: tokens[3..].join("/"),
            data: mqtt_payload_to_string(payload)?,
        }));
    }

    // Handle the "set" action (e.g. "device-id/node-id/prop-id/set")
    let [_, _, device_id, node_id, prop_id, "set"] = tokens else {
        return Ok(None);
    };
    let device = DeviceRef::new(homie_domain.clone(), device_id.to_string().try_into()?);
    let property = PropertyRef::from_node(
        NodeRef::from_device(device, HomieID::try_from(node_id.to_string())?),
        HomieID::try_from(prop_id.to_string())?,
    );
    Ok(Some(Homie5Message::PropertySet {
        property,
        set_value: mqtt_payload_to_string(payload)?,
    }))
}
