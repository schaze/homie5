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
        assert_eq!(device.homie_domain(), &HomieDomain::Default);
        assert_eq!(device.device_id().as_str(), "test-device-1");
        assert_eq!(alert_id, HomieID::new_const("battery"));
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
        assert_eq!(device.homie_domain(), &HomieDomain::Default);
        assert_eq!(device.device_id().as_str(), "test-device-1");
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
        assert_eq!(device.homie_domain(), &HomieDomain::Default);
        assert_eq!(device.device_id().as_str(), "test-device-1");
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
        assert_eq!(property.device_id().as_str(), "test-device-1");
        assert_eq!(property.node_id().as_str(), "some-node");
        assert_eq!(property.prop_id().as_str(), "some-prop");
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
        assert_eq!(homie_domain, HomieDomain::Default);
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
    assert!(matches!(event.unwrap_err(), Homie5ProtocolError::InvalidTopic));
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
    assert!(matches!(event.unwrap_err(), Homie5ProtocolError::InvalidPayload));
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
    if let Ok(Homie5Message::DeviceDescription {
        device,
        description,
        issues,
    }) = event
    {
        assert!(issues.is_empty());
        assert_eq!(device.homie_domain(), &HomieDomain::Default);
        assert_eq!(device.device_id().as_str(), "test-device-1");
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
        topic: format!(
            "{}/{}/{}/$log/warn",
            DEFAULT_HOMIE_DOMAIN, HOMIE_VERSION, "test-device-1"
        ),
        retain: false,
    };

    let event = parse_mqtt_message(&p.topic, &p.payload);
    assert!(event.is_ok());
    if let Ok(Homie5Message::DeviceLog { device, level, log_msg }) = event {
        assert_eq!(device.homie_domain(), &HomieDomain::Default);
        assert_eq!(device.device_id().as_str(), "test-device-1");
        assert!(matches!(level, DeviceLogLevel::Warn));
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
        assert_eq!(property.device_id().as_str(), "test-device-1");
        assert_eq!(property.node_id().as_str(), "some-node");
        assert_eq!(property.prop_id().as_str(), "some-prop");
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
        assert_eq!(property.device_id().as_str(), "test-device-1");
        assert_eq!(property.node_id().as_str(), "some-node");
        assert_eq!(property.prop_id().as_str(), "some-prop");
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
        assert_eq!(device.homie_domain(), &HomieDomain::Default);
        assert_eq!(device.device_id().as_str(), "test-device-1");
    } else {
        panic!(
            "Expected OK result with Homie5Message::DeviceRemoval. Instead received: {:#?}",
            event
        );
    }
}

#[test]
fn test_device_message_matches_full_parser() {
    // A device parses `/set` and `$broadcast` messages exactly like `parse_mqtt_message`
    for (topic, payload) in [
        ("homie/5/test-device-1/light/state/set", "true"),
        ("homie/5/test-device-1/light/state/set", ""),
        ("my-domain/5/test-device-1/light/brightness/set", "42"),
        ("homie/5/$broadcast/alert", "intruder"),
        ("homie/5/$broadcast/a/b/c", "data"),
    ] {
        let expected = parse_mqtt_message(topic, payload.as_bytes()).unwrap();
        assert_eq!(
            parse_device_message(topic, payload.as_bytes()).unwrap(),
            expected,
            "{topic}"
        );
    }
}

#[test]
fn test_device_message_rejects_other_topics() {
    for topic in [
        "homie/5/test-device-1/$state",
        "homie/5/test-device-1/$description",
        "homie/5/test-device-1/$log/info",
        "homie/5/test-device-1/$alert/battery",
        "homie/5/test-device-1/light/state",
        "homie/5/test-device-1/light/state/$target",
        "homie/4/test-device-1/light/state/set",
        "homie/5/test-device-1/light/state/set/extra",
    ] {
        assert!(
            matches!(
                parse_device_message(topic, b"ready"),
                Err(Homie5ProtocolError::InvalidTopic)
            ),
            "{topic}"
        );
    }
}

#[test]
fn test_device_message_rejects_invalid_ids_like_full_parser() {
    let topic = "homie/5/Test-Device/light/state/set";
    assert!(matches!(
        parse_device_message(topic, b"true"),
        Err(Homie5ProtocolError::InvalidHomieID(_))
    ));
    assert!(matches!(
        parse_mqtt_message(topic, b"true"),
        Err(Homie5ProtocolError::InvalidHomieID(_))
    ));
}
