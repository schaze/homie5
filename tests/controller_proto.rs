use homie5::client::QoS;
use homie5::device_description::*;
use homie5::*;

fn prop() -> PropertyRef {
    PropertyRef::new(
        HomieDomain::Default,
        HomieID::new_const("dev"),
        HomieID::new_const("node"),
        HomieID::new_const("prop"),
    )
}

#[test]
fn set_command_uses_set_topic_without_retain() {
    let p = Homie5ControllerProtocol::new().set_command(&prop(), &HomieValue::Integer(5));
    assert_eq!(p.topic, "homie/5/dev/node/prop/set");
    assert_eq!(p.payload, b"5");
    assert!(!p.retain);
    assert_eq!(p.qos, QoS::ExactlyOnce);
}

#[test]
fn set_command_for_property_uses_qos_0_for_non_retained() {
    let controller = Homie5ControllerProtocol::new();
    let value = HomieValue::Bool(true);

    let event = PropertyDescriptionBuilder::boolean()
        .settable(true)
        .retained(false)
        .build();
    let p = controller.set_command_for_property(&prop(), &event, &value);
    assert_eq!((p.qos, p.retain), (QoS::AtMostOnce, false));
    assert_eq!(p.topic, "homie/5/dev/node/prop/set");
    assert_eq!(p.payload, b"true");

    let state = PropertyDescriptionBuilder::boolean().settable(true).build();
    let p = controller.set_command_for_property(&prop(), &state, &value);
    assert_eq!((p.qos, p.retain), (QoS::ExactlyOnce, false));
}

#[test]
fn set_command_encodes_empty_string_as_nul_byte() {
    let p = Homie5ControllerProtocol::new().set_command(&prop(), &HomieValue::String(String::new()));
    assert_eq!(p.payload, vec![0u8]);
}

#[test]
fn device_subscriptions_cover_device_attributes() {
    let device = DeviceRef::new(HomieDomain::Default, HomieID::new_const("dev"));
    let topics: Vec<String> = Homie5ControllerProtocol::new()
        .subscribe_device(&device)
        .map(|s| s.topic)
        .collect();
    assert!(topics.contains(&"homie/5/dev/$description".to_string()));
    assert!(topics.contains(&"homie/5/dev/$alert/+".to_string()));
    assert!(!topics.contains(&"homie/5/dev/$state".to_string()));
}
