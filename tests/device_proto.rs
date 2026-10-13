use homie5::client::{Publish, QoS};
use homie5::device_description::*;
use homie5::*;

fn id(s: &'static str) -> HomieID {
    HomieID::new_const(s)
}

fn root_proto() -> Homie5DeviceProtocol {
    Homie5DeviceProtocol::new(id("root"), HomieDomain::Default).0
}

fn description(root: Option<HomieID>) -> HomieDeviceDescription {
    DeviceDescriptionBuilder::new()
        .root(root)
        .add_node(
            id("node"),
            NodeDescriptionBuilder::new()
                .add_property(id("retained"), PropertyDescriptionBuilder::integer().build())
                .add_property(
                    id("event"),
                    PropertyDescriptionBuilder::integer().retained(false).build(),
                )
                .build(),
        )
        .build()
}

#[test]
fn remove_device_clears_state_first_and_only_retained_topics() {
    let proto = root_proto();
    let desc = description(None);
    let publishes: Vec<Publish> = proto.remove_device(&desc).unwrap().collect();
    let topics: Vec<&str> = publishes.iter().map(|p| p.topic.as_str()).collect();

    assert_eq!(
        topics,
        [
            "homie/5/root/$state",
            "homie/5/root/$description",
            "homie/5/root/node/retained",
            "homie/5/root/node/retained/$target",
        ]
    );
    assert!(publishes.iter().all(|p| p.retain && p.payload.is_empty()));
}

#[test]
fn clear_alert_targets_alert_topic() {
    let p = root_proto().publish_clear_alert(&id("battery"));
    assert_eq!(p.topic, "homie/5/root/$alert/battery");
    assert!(p.retain);
    assert!(p.payload.is_empty());
}

#[test]
fn non_retained_values_and_targets_use_qos_0() {
    let proto = root_proto();
    let value = proto.publish_value(&id("node"), &id("event"), "1", false);
    assert_eq!((value.qos, value.retain), (QoS::AtMostOnce, false));
    let target = proto.publish_target(&id("node"), &id("event"), "1", false);
    assert_eq!((target.qos, target.retain), (QoS::AtMostOnce, false));

    let value = proto.publish_value(&id("node"), &id("retained"), "1", true);
    assert_eq!((value.qos, value.retain), (QoS::ExactlyOnce, true));
    let target = proto.publish_target(&id("node"), &id("retained"), "1", true);
    assert_eq!((target.qos, target.retain), (QoS::ExactlyOnce, true));
}

#[test]
fn empty_string_value_is_published_as_nul_byte() {
    let p = root_proto().publish_value(&id("node"), &id("retained"), "", true);
    assert_eq!(p.payload, vec![0u8]);
}

#[test]
fn root_device_description_must_not_have_root() {
    let proto = root_proto();
    assert!(proto.publish_description(&description(None)).is_ok());
    assert!(matches!(
        proto.publish_description(&description(Some(id("other")))),
        Err(Homie5ProtocolError::NonEmptyRootForRootDevice)
    ));
}

#[test]
fn child_device_description_must_reference_root() {
    let child = root_proto().clone_for_child(id("child"));
    assert!(child.is_child());
    assert_eq!(child.root_id(), &id("root"));

    assert!(child.publish_description(&description(Some(id("root")))).is_ok());
    assert!(matches!(
        child.publish_description(&description(None)),
        Err(Homie5ProtocolError::RootMismatch)
    ));
    assert!(matches!(
        child.publish_description(&description(Some(id("other")))),
        Err(Homie5ProtocolError::RootMismatch)
    ));
    assert!(child.subscribe_props(&description(None)).is_err());
    assert!(child.remove_device(&description(None)).is_err());
}

#[test]
fn nested_child_keeps_original_root() {
    let grandchild = root_proto()
        .clone_for_child(id("child"))
        .clone_for_child(id("grandchild"));
    assert_eq!(grandchild.root_id(), &id("root"));
    assert!(grandchild.publish_description(&description(Some(id("root")))).is_ok());

    let via_for_child = Homie5DeviceProtocol::for_child(id("child"), root_proto());
    assert_eq!(via_for_child.root_id(), &id("root"));
}

#[test]
fn root_device_publishing_for_child_id_requires_root() {
    let proto = root_proto();
    assert!(proto
        .publish_description_for_id(&id("child"), &description(Some(id("root"))))
        .is_ok());
    assert!(matches!(
        proto.publish_description_for_id(&id("child"), &description(None)),
        Err(Homie5ProtocolError::RootMismatch)
    ));
}

#[test]
fn description_json_omits_unset_optional_fields() {
    let desc = DeviceDescriptionBuilder::new().name("x").build();
    let p = root_proto().publish_description(&desc).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&p.payload).unwrap();
    let obj = json.as_object().unwrap();
    assert!(!obj.contains_key("type"));
    assert!(!obj.contains_key("root"));
    assert!(!obj.contains_key("parent"));
    assert_eq!(obj["homie"], "5.0");

    let typed = DeviceDescriptionBuilder::new().r#type("sensor").build();
    let json = serde_json::to_value(&typed).unwrap();
    assert_eq!(json["type"], "sensor");
}

#[test]
fn subscribe_props_only_covers_settable_properties() {
    let desc = DeviceDescriptionBuilder::new()
        .add_node(
            id("node"),
            NodeDescriptionBuilder::new()
                .add_property(id("ro"), PropertyDescriptionBuilder::integer().build())
                .add_property(id("rw"), PropertyDescriptionBuilder::integer().settable(true).build())
                .build(),
        )
        .build();
    let proto = root_proto();
    let subs: Vec<String> = proto.subscribe_props(&desc).unwrap().map(|s| s.topic).collect();
    assert_eq!(subs, ["homie/5/root/node/rw/set"]);
    let unsubs: Vec<String> = proto.unsubscribe_props(&desc).unwrap().map(|s| s.topic).collect();
    assert_eq!(unsubs, subs);
}

#[test]
fn last_will_publishes_lost_state() {
    let (_, last_will) = Homie5DeviceProtocol::new(id("root"), HomieDomain::Default);
    assert_eq!(last_will.topic, "homie/5/root/$state");
    assert_eq!(last_will.message, b"lost");
    assert!(last_will.retain);
}
