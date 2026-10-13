use homie5::*;

fn dev() -> DeviceRef {
    DeviceRef::new(HomieDomain::Default, HomieID::new_const("dev"))
}

fn prop() -> PropertyRef {
    PropertyRef::new(
        HomieDomain::Default,
        HomieID::new_const("dev"),
        HomieID::new_const("node"),
        HomieID::new_const("prop"),
    )
}

#[test]
fn empty_state_is_device_removal() {
    assert_eq!(
        parse_mqtt_message("homie/5/dev/$state", b"").unwrap(),
        Homie5Message::DeviceRemoval { device: dev() }
    );
}

#[test]
fn empty_description_is_description_cleared() {
    assert_eq!(
        parse_mqtt_message("homie/5/dev/$description", b"").unwrap(),
        Homie5Message::DeviceDescriptionCleared { device: dev() }
    );
}

#[test]
fn alert_deletion_and_empty_alert_are_distinct() {
    let alert_id = HomieID::new_const("battery");
    assert_eq!(
        parse_mqtt_message("homie/5/dev/$alert/battery", b"").unwrap(),
        Homie5Message::DeviceAlertCleared {
            device: dev(),
            alert_id: alert_id.clone()
        }
    );
    assert_eq!(
        parse_mqtt_message("homie/5/dev/$alert/battery", &[0]).unwrap(),
        Homie5Message::DeviceAlert {
            device: dev(),
            alert_id,
            alert_msg: String::new()
        }
    );
}

#[test]
fn value_deletion_and_empty_string_are_distinct() {
    assert_eq!(
        parse_mqtt_message("homie/5/dev/node/prop", b"").unwrap(),
        Homie5Message::PropertyValueCleared { property: prop() }
    );
    assert_eq!(
        parse_mqtt_message("homie/5/dev/node/prop", &[0]).unwrap(),
        Homie5Message::PropertyValue {
            property: prop(),
            value: String::new()
        }
    );
    assert_eq!(
        parse_mqtt_message("homie/5/dev/node/prop", b"21.5").unwrap(),
        Homie5Message::PropertyValue {
            property: prop(),
            value: "21.5".to_owned()
        }
    );
}

#[test]
fn target_deletion_and_empty_string_are_distinct() {
    assert_eq!(
        parse_mqtt_message("homie/5/dev/node/prop/$target", b"").unwrap(),
        Homie5Message::PropertyTargetCleared { property: prop() }
    );
    assert_eq!(
        parse_mqtt_message("homie/5/dev/node/prop/$target", &[0]).unwrap(),
        Homie5Message::PropertyTarget {
            property: prop(),
            target: String::new()
        }
    );
}

#[test]
fn device_removal_round_trips_through_parser() {
    use homie5::device_description::*;

    let (proto, _) = Homie5DeviceProtocol::new(HomieID::new_const("dev"), HomieDomain::Default);
    let desc = DeviceDescriptionBuilder::new()
        .add_node(
            HomieID::new_const("node"),
            NodeDescriptionBuilder::new()
                .add_property(
                    HomieID::new_const("prop"),
                    PropertyDescriptionBuilder::integer().build(),
                )
                .build(),
        )
        .build();
    let messages: Vec<Homie5Message> = proto
        .remove_device(&desc)
        .unwrap()
        .map(|p| parse_mqtt_message(&p.topic, &p.payload).unwrap())
        .collect();
    assert_eq!(
        messages,
        [
            Homie5Message::DeviceRemoval { device: dev() },
            Homie5Message::DeviceDescriptionCleared { device: dev() },
            Homie5Message::PropertyValueCleared { property: prop() },
            Homie5Message::PropertyTargetCleared { property: prop() },
        ]
    );
}

#[test]
fn published_empty_alert_is_not_a_deletion() {
    let (proto, _) = Homie5DeviceProtocol::new(HomieID::new_const("dev"), HomieDomain::Default);
    let p = proto.publish_alert(&HomieID::new_const("battery"), "");
    assert_eq!(p.payload, vec![0u8]);
    assert!(matches!(
        parse_mqtt_message(&p.topic, &p.payload).unwrap(),
        Homie5Message::DeviceAlert { .. }
    ));
}
