use bytes::Bytes;

use homie5::{DEFAULT_HOMIE_DOMAIN, DEVICE_ATTRIBUTE_STATE, HOMIE_VERSION};

use homie5::*;

#[test]
fn test_device_alert_msg() {
    let p = rumqttc::Publish {
        dup: false,
        qos: rumqttc::QoS::ExactlyOnce,
        payload: "Battery is low!".into(),
        pkid: 0,
        topic: format!(
            "{}/{}/{}/$alert/{}",
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1", "battery"
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
        assert_eq!(device.homie_domain, DEFAULT_HOMIE_DOMAIN.to_owned());
        assert_eq!(device.id.as_str(), "test-device-1");
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
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1", DEVICE_ATTRIBUTE_STATE
        ),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::DeviceRemoval { device }) = event {
        assert_eq!(device.homie_domain, DEFAULT_HOMIE_DOMAIN.to_owned());
        assert_eq!(device.id.as_str(), "test-device-1");
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
        payload: Bytes::from(HomieDeviceStatus::Ready.as_str()),
        pkid: 0,
        topic: format!(
            "{}/{}/{}/{}",
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1", DEVICE_ATTRIBUTE_STATE
        ),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::DeviceState { device, state }) = event {
        assert_eq!(device.homie_domain, DEFAULT_HOMIE_DOMAIN.to_owned());
        assert_eq!(device.id.as_str(), "test-device-1");
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
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1"
        ),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::PropertyValue { property, value }) = event {
        assert_eq!(property.node.device.id.as_str(), "test-device-1");
        assert_eq!(property.node.id, "some-node".try_into().unwrap());
        assert_eq!(property.id, "some-prop".try_into().unwrap());
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
        topic: format!("{}/{}/$broadcast/system", DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::Broadcast {
        homie_domain,
        subtopic,
        data,
    }) = event
    {
        assert_eq!(homie_domain, DEFAULT_HOMIE_DOMAIN.to_owned());
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
        topic: format!("{}/invalid", DEFAULT_HOMIE_DOMAIN),
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
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1", DEVICE_ATTRIBUTE_STATE
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
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1"
        ),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::DeviceDescription { device, description }) = event {
        assert_eq!(device.homie_domain, DEFAULT_HOMIE_DOMAIN.to_owned());
        assert_eq!(device.id.as_str(), "test-device-1");
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
        topic: format!("{}/{}/{}/$log", DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1"),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::DeviceLog { device, log_msg }) = event {
        assert_eq!(device.homie_domain, DEFAULT_HOMIE_DOMAIN.to_owned());
        assert_eq!(device.id.as_str(), "test-device-1");
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
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1"
        ),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::PropertyTarget { property, target }) = event {
        assert_eq!(property.node.device.id.as_str(), "test-device-1");
        assert_eq!(property.node.id, "some-node".try_into().unwrap());
        assert_eq!(property.id, "some-prop".try_into().unwrap());
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
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1"
        ),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::PropertySet { property, set_value }) = event {
        assert_eq!(property.node.device.id.as_str(), "test-device-1");
        assert_eq!(property.node.id, "some-node".try_into().unwrap());
        assert_eq!(property.id, "some-prop".try_into().unwrap());
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
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1", DEVICE_ATTRIBUTE_STATE
        ),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::DeviceRemoval { device }) = event {
        assert_eq!(device.homie_domain, DEFAULT_HOMIE_DOMAIN.to_owned());
        assert_eq!(device.id.as_str(), "test-device-1");
    } else {
        panic!(
            "Expected OK result with Homie5Message::DeviceRemoval. Instead received: {:#?}",
            event
        );
    }
}
