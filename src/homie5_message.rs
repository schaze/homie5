use crate::{
    device_description::HomieDeviceDescription, error::Homie5ProtocolError, DeviceIdentifier, HomieDeviceStatus,
    PropertyIdentifier,
};
use std::str::FromStr;
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
        device: DeviceIdentifier,
        /// The received state, which can be "init", "ready", "disconnected", "sleeping", or "lost".
        state: HomieDeviceStatus,
    },

    /// A message describing a device has been received.
    ///
    /// Device descriptions are sent when a device publishes its metadata (such as its name,
    /// version, and available nodes) under `homie/5/<device-id>/$description`.
    DeviceDescription {
        /// The device identifier for which the description was received.
        device: DeviceIdentifier,
        /// The full device description, including metadata like nodes and properties.
        description: HomieDeviceDescription,
    },

    /// A log message from the device.
    ///
    /// Devices can publish logs for debugging or informational purposes to the topic
    /// `homie/5/<device-id>/$log`. These logs can help monitor device activity or identify issues.
    DeviceLog {
        /// The device identifier from which the log was received.
        device: DeviceIdentifier,
        /// The log message from the device.
        log_msg: String,
    },

    /// An alert message from the device.
    ///
    /// Alerts are published under `homie/5/<device-id>/$alert` and represent critical
    /// notifications about the device’s state (e.g., sensor failures, errors that need human intervention).
    DeviceAlert {
        /// The device identifier from which the alert was received.
        device: DeviceIdentifier,
        /// A unique identifier for the alert.
        alert_id: String,
        /// The alert message providing details about the issue.
        alert_msg: String,
    },

    /// A property value message has been received.
    ///
    /// Property values are typically sensor readings or other dynamic values
    /// sent under `homie/5/<device-id>/<node-id>/<property-id>`.
    PropertyValue {
        /// The property identifier indicating which device/node/property the value belongs to.
        property: PropertyIdentifier,
        /// The actual value of the property (e.g., "21.5" for a temperature sensor).
        value: String,
    },

    /// A property target message has been received.
    ///
    /// The `$target` attribute describes an intended state change for a property and is published
    /// under `homie/5/<device-id>/<node-id>/<property-id>/$target`. It represents the desired value.
    PropertyTarget {
        /// The property identifier indicating which device/node/property the target is set for.
        property: PropertyIdentifier,
        /// The intended target value for the property.
        target: String,
    },

    /// A property set message has been received.
    ///
    /// These messages represent commands to set a property to a specific value and are sent under
    /// `homie/5/<device-id>/<node-id>/<property-id>/set`.
    PropertySet {
        /// The property identifier for which the value is being set.
        property: PropertyIdentifier,
        /// The value to which the property is being set.
        set_value: String,
    },

    /// A broadcast message has been received.
    ///
    /// Broadcasts are messages that do not belong to a specific device but are sent under
    /// `homie/5/$broadcast/<subtopic>` to all listeners. These messages are used for general-purpose communication.
    Broadcast {
        /// The root topic of the broadcast.
        topic_root: String,
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
        device: DeviceIdentifier,
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

    let topic_root = tokens[0].to_owned();
    let device_id = tokens[2].to_owned();

    // Attempt to parse the payload as a UTF-8 string
    let payload = String::from_utf8(payload.to_vec())?;

    // Handle broadcast messages (e.g. "homie/5/$broadcast")
    if device_id == "$broadcast" {
        return Ok(Homie5Message::Broadcast {
            topic_root,
            subtopic: tokens[3..].join("/"),
            data: payload,
        });
    }

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
                        if let Ok(state) = HomieDeviceStatus::from_str(&payload) {
                            Ok(Homie5Message::DeviceState {
                                device: DeviceIdentifier {
                                    topic_root,
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
                            device: DeviceIdentifier {
                                topic_root,
                                id: device_id,
                            },
                        })
                    }
                }
                // Handle the "$description" attribute, parsing as JSON
                "$description" => match serde_json::from_str::<HomieDeviceDescription>(&payload) {
                    Ok(description) => Ok(Homie5Message::DeviceDescription {
                        device: DeviceIdentifier {
                            topic_root,
                            id: device_id,
                        },
                        description,
                    }),
                    Err(err) => {
                        log::error!("{:#?}", err);
                        Err(Homie5ProtocolError::InvalidPayload)
                    }
                },
                // Handle the "$log" attribute
                "$log" => Ok(Homie5Message::DeviceLog {
                    device: DeviceIdentifier {
                        topic_root,
                        id: device_id,
                    },
                    log_msg: payload,
                }),
                _ => Err(Homie5ProtocolError::InvalidTopic),
            }
        }
        5 => {
            match tokens[3] {
                // Handle alert messages (e.g. "device-id/$alert/alert-id")
                "$alert" => {
                    let alert_id = tokens[4].to_owned();
                    Ok(Homie5Message::DeviceAlert {
                        device: DeviceIdentifier {
                            topic_root,
                            id: device_id,
                        },
                        alert_id,
                        alert_msg: payload,
                    })
                }
                // Handle property values (e.g. "device-id/node-id/prop-id")
                _ => {
                    let node_id = tokens[3].to_owned();
                    let prop_id = tokens[4].to_owned();
                    Ok(Homie5Message::PropertyValue {
                        property: PropertyIdentifier::new(topic_root, device_id, node_id, prop_id),
                        value: payload,
                    })
                }
            }
        }
        6 => {
            // Handle property attributes (e.g. "device-id/node-id/prop-id/$target")
            let node_id = tokens[3].to_owned();
            let prop_id = tokens[4].to_owned();
            let attr = tokens[5];
            match attr {
                // Handle the "set" action
                "set" => Ok(Homie5Message::PropertySet {
                    property: PropertyIdentifier::new(topic_root, device_id, node_id.to_owned(), prop_id.to_owned()),
                    set_value: payload,
                }),
                // Handle the "$target" attribute
                "$target" => Ok(Homie5Message::PropertyTarget {
                    property: PropertyIdentifier::new(topic_root, device_id, node_id, prop_id),
                    target: payload,
                }),
                _ => Err(Homie5ProtocolError::InvalidTopic),
            }
        }
        _ => Err(Homie5ProtocolError::InvalidTopic),
    }
}
