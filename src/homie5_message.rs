use crate::{
    device_description::HomieDeviceDescription, error::Homie5ProtocolError, DeviceIdentifier, HomieDeviceStatus,
    PropertyIdentifier,
};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Homie5Message {
    DeviceState {
        device: DeviceIdentifier,
        state: HomieDeviceStatus,
    },
    DeviceDescription {
        device: DeviceIdentifier,
        description: HomieDeviceDescription,
    },
    DeviceLog {
        device: DeviceIdentifier,
        log_msg: String,
    },
    DeviceAlert {
        device: DeviceIdentifier,
        alert_id: String,
        alert_msg: String,
    },

    PropertyValue {
        property: PropertyIdentifier,
        value: String,
    },
    PropertyTarget {
        property: PropertyIdentifier,
        target: String,
    },
    PropertySet {
        property: PropertyIdentifier,
        set_value: String,
    },

    Broadcast {
        topic_root: String,
        subtopic: String,
        data: String,
    },

    DeviceRemoval {
        device: DeviceIdentifier,
    },
}

pub fn parse_mqtt_message(topic: &str, payload: &[u8]) -> Result<Homie5Message, Homie5ProtocolError> {
    let tokens: Vec<&str> = topic.split('/').collect();

    if tokens.len() <= 3 {
        return Err(Homie5ProtocolError::InvalidTopic);
    }

    let topic_root = tokens[0].to_owned();
    let device_id = tokens[2].to_owned();

    let payload = String::from_utf8(payload.to_vec())?;

    if device_id == "$broadcast" {
        return Ok(Homie5Message::Broadcast {
            topic_root,
            subtopic: tokens[3..].join("/"),
            data: payload,
        });
    }

    // len: 0    1  2     3        4       5       6
    // topic: homie/5/device_id/node_id/prop_id/$target
    // topic: homie/5/device_id/$state
    // index:    0  1     2        3       4       5
    match tokens.len() {
        4 => {
            // Device attribute (e.g. topic_root/5/device-id/$state)
            let attr = tokens[3];
            match attr {
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
                        Ok(Homie5Message::DeviceRemoval {
                            device: DeviceIdentifier {
                                topic_root,
                                id: device_id,
                            },
                        })
                    }
                }
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
                // Alert message (e.g. device_id/$alert/alert-id = alert-message)
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
                // PropertyValue (e.g. device-id/node-id/prop-id = true)
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
            // property attribute (e.g. device-id/node-id/prop-id/$target )
            let node_id = tokens[3].to_owned();
            let prop_id = tokens[4].to_owned();
            let attr = tokens[5];
            match attr {
                "set" => Ok(Homie5Message::PropertySet {
                    property: PropertyIdentifier::new(topic_root, device_id, node_id.to_owned(), prop_id.to_owned()),
                    set_value: payload,
                }),
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

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::{DEFAULT_ROOT_TOPIC, DEVICE_ATTRIBUTE_STATE, HOMIE_VERSION};

    use super::*;

    #[test]
    fn test_device_alert_msg() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: "Battery is low!".into(),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/$alert/{}",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1", "battery"
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::DeviceAlert {
            device,
            alert_id,
            alert_msg,
        }) = event
        {
            assert_eq!(device.topic_root, DEFAULT_ROOT_TOPIC.to_owned());
            assert_eq!(device.id, "test-device-1".to_owned());
            assert_eq!(alert_id, "battery".to_owned());
            assert_eq!(alert_msg, "Battery is low!".to_owned());
        } else {
            panic!(
                "Expected OK result with Homie5Message::DeviceAlert. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_empty_state_aka_device_removal() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: Bytes::new(),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/{}",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1", DEVICE_ATTRIBUTE_STATE
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::DeviceRemoval { device }) = event {
            assert_eq!(device.topic_root, DEFAULT_ROOT_TOPIC.to_owned());
            assert_eq!(device.id, "test-device-1".to_owned());
        } else {
            panic!(
                "Expected OK result with Homie5Message::DeviceRemoval. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_valid_state_event() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: Bytes::from(HomieDeviceStatus::Ready.to_str()),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/{}",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1", DEVICE_ATTRIBUTE_STATE
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::DeviceState { device, state }) = event {
            assert_eq!(device.topic_root, DEFAULT_ROOT_TOPIC.to_owned());
            assert_eq!(device.id, "test-device-1".to_owned());
            assert_eq!(state, HomieDeviceStatus::Ready);
        } else {
            panic!(
                "Expected OK result with Homie5Message::DeviceState. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_property_value() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: "true".into(),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/some-node/some-prop",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1"
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::PropertyValue { property, value }) = event {
            assert_eq!(property.node.device.id, "test-device-1".to_owned());
            assert_eq!(property.node.id, "some-node".to_owned());
            assert_eq!(property.id, "some-prop".to_owned());
            assert_eq!(value, "true".to_owned());
        } else {
            panic!(
                "Expected OK result with Homie5Message::PropertyValue. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_broadcast_message() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: "global broadcast data".into(),
            pkid: 0,
            topic: format!("{}/{}/$broadcast/system", DEFAULT_ROOT_TOPIC, HOMIE_VERSION),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::Broadcast {
            topic_root,
            subtopic,
            data,
        }) = event
        {
            assert_eq!(topic_root, DEFAULT_ROOT_TOPIC.to_owned());
            assert_eq!(subtopic, "system".to_owned());
            assert_eq!(data, "global broadcast data".to_owned());
        } else {
            panic!(
                "Expected OK result with Homie5Message::Broadcast. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_invalid_topic() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: "invalid".into(),
            pkid: 0,
            topic: format!("{}/invalid", DEFAULT_ROOT_TOPIC),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_err());
        assert_eq!(event.unwrap_err(), Homie5ProtocolError::InvalidTopic);
    }

    #[test]
    fn test_invalid_payload() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: "non existing state".into(),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/{}",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1", DEVICE_ATTRIBUTE_STATE
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_err());
        assert_eq!(event.unwrap_err(), Homie5ProtocolError::InvalidPayload);
    }
    #[test]
    fn test_device_description_msg() {
        let description_json = r#"{
        "name": "Test Device",
        "version": 1234,
        "homie": "5.0",
        "nodes":{} 
    }"#;

        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: Bytes::from(description_json),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/$description",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1"
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::DeviceDescription { device, description }) = event {
            assert_eq!(device.topic_root, DEFAULT_ROOT_TOPIC.to_owned());
            assert_eq!(device.id, "test-device-1".to_owned());
            assert_eq!(description.name.unwrap(), "Test Device");
        } else {
            panic!(
                "Expected OK result with Homie5Message::DeviceDescription. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_device_log_msg() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: Bytes::from("Device restarted"),
            pkid: 0,
            topic: format!("{}/{}/{}/$log", DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1"),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::DeviceLog { device, log_msg }) = event {
            assert_eq!(device.topic_root, DEFAULT_ROOT_TOPIC.to_owned());
            assert_eq!(device.id, "test-device-1".to_owned());
            assert_eq!(log_msg, "Device restarted".to_owned());
        } else {
            panic!(
                "Expected OK result with Homie5Message::DeviceLog. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_property_target_msg() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: Bytes::from("75"),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/some-node/some-prop/$target",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1"
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::PropertyTarget { property, target }) = event {
            assert_eq!(property.node.device.id, "test-device-1".to_owned());
            assert_eq!(property.node.id, "some-node".to_owned());
            assert_eq!(property.id, "some-prop".to_owned());
            assert_eq!(target, "75".to_owned());
        } else {
            panic!(
                "Expected OK result with Homie5Message::PropertyTarget. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_property_set_msg() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: Bytes::from("100"),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/some-node/some-prop/set",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1"
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::PropertySet { property, set_value }) = event {
            assert_eq!(property.node.device.id, "test-device-1".to_owned());
            assert_eq!(property.node.id, "some-node".to_owned());
            assert_eq!(property.id, "some-prop".to_owned());
            assert_eq!(set_value, "100".to_owned());
        } else {
            panic!(
                "Expected OK result with Homie5Message::PropertySet. Instead received: {:#?}",
                event
            );
        }
    }

    #[test]
    fn test_device_removal_msg() {
        let p = rumqttc::Publish {
            dup: false,
            qos: rumqttc::QoS::ExactlyOnce,
            payload: Bytes::new(),
            pkid: 0,
            topic: format!(
                "{}/{}/{}/{}",
                DEFAULT_ROOT_TOPIC, HOMIE_VERSION, "test-device-1", DEVICE_ATTRIBUTE_STATE
            ),
            retain: false,
        };

        let event = parse_mqtt_message(&p.topic, &p.payload);
        assert!(event.is_ok());
        if let Ok(Homie5Message::DeviceRemoval { device }) = event {
            assert_eq!(device.topic_root, DEFAULT_ROOT_TOPIC.to_owned());
            assert_eq!(device.id, "test-device-1".to_owned());
        } else {
            panic!(
                "Expected OK result with Homie5Message::DeviceRemoval. Instead received: {:#?}",
                event
            );
        }
    }
}
