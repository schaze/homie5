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
//! - `DeviceDescription`: Provides metadata about the device.
//! - `DeviceLog`: Contains log messages for debugging purposes.
//! - `DeviceAlert`: Represents critical alerts from the device.
//! - `PropertyValue`: Contains the current value of a property.
//! - `PropertyTarget`: Represents the desired state of a property.
//! - `PropertySet`: A command to set a property to a specific value.
//! - `Broadcast`: Represents general-purpose communication broadcast messages.
//! - `DeviceRemoval`: Represents the removal of a device from the network.
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

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec::Vec,
};

use crate::{
    client::mqtt_payload_to_string, device_description::HomieDeviceDescription, error::Homie5ProtocolError,
    DeviceLogLevel, DeviceRef, HomieDeviceStatus, HomieDomain, HomieID, PropertyRef, HOMIE_VERSION,
};
/// Represents all possible MQTT message types according to the Homie 5 protocol.
/// These messages define the interactions between devices, their attributes, and the broker.
#[derive(Debug, Clone)]
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

    /// A message describing a device has been received.
    ///
    /// Device descriptions are sent when a device publishes its metadata (such as its name,
    /// version, and available nodes) under `homie/5/<device-id>/$description`.
    DeviceDescription {
        /// The device identifier for which the description was received.
        device: DeviceRef,
        /// The full device description, including metadata like nodes and properties.
        description: HomieDeviceDescription,
    },

    /// A log message from the device.
    ///
    /// Devices can publish logs for debugging or informational purposes to the topic
    /// `homie/5/<device-id>/$log`. These logs can help monitor device activity or identify issues.
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
    /// Alerts are published under `homie/5/<device-id>/$alert` and represent critical
    /// notifications about the device’s state (e.g., sensor failures, errors that need human intervention).
    DeviceAlert {
        /// The device identifier from which the alert was received.
        device: DeviceRef,
        /// A unique identifier for the alert.
        alert_id: HomieID,
        /// The alert message providing details about the issue.
        alert_msg: String,
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

    /// A device removal message has been received, indicating the device has been permanently removed from the network.
    ///
    /// This message represents the process of clearing all retained messages for a device from the MQTT broker,
    /// starting with a zero-length payload published to the `$state` topic. Afterward, other retained attributes
    /// and property values must also be cleared. This effectively removes the device from the MQTT ecosystem.
    DeviceRemoval {
        /// The device identifier for the device that was removed.
        device: DeviceRef,
    },
}

/// Parses an incoming MQTT message into a `Homie5Message`.
///
/// This function analyzes the topic structure and payload of an MQTT message according
/// to the Homie 5 protocol. It converts the raw MQTT message into the appropriate `Homie5Message`
/// enum variant, such as `DeviceState`, `DeviceDescription`, `DeviceLog`, or others.
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
///
/// # Example
/// ```rust
/// use homie5::parse_mqtt_message;
/// let topic = "homie/5/device1/$state";
/// let payload = b"ready";
/// let message = parse_mqtt_message(topic, payload).unwrap();
/// ```
pub fn parse_mqtt_message(topic: &str, payload: &[u8]) -> Result<Homie5Message, Homie5ProtocolError> {
    // Split the topic into components based on '/' delimiter
    let tokens: Vec<&str> = topic.split('/').collect();

    // Ensure the topic contains at least 4 tokens (e.g. "homie/5/device-id/$state")
    if tokens.len() <= 3 {
        return Err(Homie5ProtocolError::InvalidTopic);
    }

    let homie_domain: HomieDomain = tokens[0].to_owned().try_into()?;

    // Ensure homie version matches to supported version
    if tokens[1] != HOMIE_VERSION {
        return Err(Homie5ProtocolError::InvalidTopic);
    }

    // Handle broadcast messages (e.g. "homie/5/$broadcast")
    if tokens[2] == "$broadcast" {
        return Ok(Homie5Message::Broadcast {
            homie_domain,
            subtopic: tokens[3..].join("/"),
            data: mqtt_payload_to_string(payload)?,
        });
    }

    // check the homie id provided
    let device_id = tokens[2].to_string().try_into()?;

    // Match the topic length to identify the message type
    // len: 0    1  2     3        4       5       6
    // topic: homie/5/device_id/node_id/prop_id/$target
    // topic: homie/5/device_id/$state
    // index:    0  1     2        3       4       5
    match tokens.len() {
        4 => {
            // Device attribute (e.g. "homie/5/device-id/$state")
            let attr = tokens[3];
            match attr {
                // Handle the "$state" attribute
                "$state" => {
                    if !payload.is_empty() {
                        if let Ok(state) = mqtt_payload_to_string(payload)?.try_into() {
                            Ok(Homie5Message::DeviceState {
                                device: DeviceRef {
                                    homie_domain,
                                    id: device_id,
                                },
                                state,
                            })
                        } else {
                            Err(Homie5ProtocolError::InvalidPayload)
                        }
                    } else {
                        // Empty payload signifies device removal
                        Ok(Homie5Message::DeviceRemoval {
                            device: DeviceRef {
                                homie_domain,
                                id: device_id,
                            },
                        })
                    }
                }
                // Handle the "$description" attribute, parsing as JSON
                "$description" => {
                    match serde_json::from_str::<HomieDeviceDescription>(&mqtt_payload_to_string(payload)?) {
                        Ok(description) => Ok(Homie5Message::DeviceDescription {
                            device: DeviceRef {
                                homie_domain,
                                id: device_id,
                            },
                            description,
                        }),
                        Err(_) => Err(Homie5ProtocolError::InvalidPayload),
                    }
                }
                _ => Err(Homie5ProtocolError::InvalidTopic),
            }
        }
        5 => {
            match tokens[3] {
                // Handle alert messages (e.g. "device-id/$alert/alert-id")
                "$alert" => {
                    let alert_id = HomieID::try_from(tokens[4].to_owned())?;
                    Ok(Homie5Message::DeviceAlert {
                        device: DeviceRef {
                            homie_domain,
                            id: device_id,
                        },
                        alert_id,
                        alert_msg: mqtt_payload_to_string(payload)?,
                    })
                }
                // Handle the "$log" attribute
                "$log" => {
                    let level = DeviceLogLevel::try_from(tokens[4])?;
                    Ok(Homie5Message::DeviceLog {
                        device: DeviceRef {
                            homie_domain,
                            id: device_id,
                        },
                        level,
                        log_msg: mqtt_payload_to_string(payload)?,
                    })
                }
                // Handle property values (e.g. "device-id/node-id/prop-id")
                _ => {
                    let node_id = HomieID::try_from(tokens[3].to_string())?;
                    let prop_id = HomieID::try_from(tokens[4].to_string())?;
                    Ok(Homie5Message::PropertyValue {
                        property: PropertyRef::new(homie_domain, device_id, node_id, prop_id),
                        value: mqtt_payload_to_string(payload)?,
                    })
                }
            }
        }
        6 => {
            // Handle property attributes (e.g. "device-id/node-id/prop-id/$target")
            let node_id = HomieID::try_from(tokens[3].to_string())?;
            let prop_id = HomieID::try_from(tokens[4].to_string())?;
            let attr = tokens[5];
            match attr {
                // Handle the "set" action
                "set" => Ok(Homie5Message::PropertySet {
                    property: PropertyRef::new(homie_domain, device_id, node_id.to_owned(), prop_id.to_owned()),
                    set_value: mqtt_payload_to_string(payload)?,
                }),
                // Handle the "$target" attribute
                "$target" => Ok(Homie5Message::PropertyTarget {
                    property: PropertyRef::new(homie_domain, device_id, node_id, prop_id),
                    target: mqtt_payload_to_string(payload)?,
                }),
                _ => Err(Homie5ProtocolError::InvalidTopic),
            }
        }
        _ => Err(Homie5ProtocolError::InvalidTopic),
    }
}
