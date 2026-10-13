use homie5::device_description::*;
use homie5::*;

const DESCRIPTION: &str = r#"{
    "homie": "5.0",
    "version": 3,
    "name": "Relay",
    "nodes": {
        "relay": {
            "name": "Relay",
            "properties": {
                "state": { "datatype": "boolean", "format": "off,on", "settable": true },
                "mode":  { "datatype": "boolean", "format": "a,b,c" },
                "Bad_Id": { "datatype": "integer" },
                "power": { "datatype": "float", "unit": "W" }
            }
        },
        "broken": "not an object",
        "Bad-Node": { "properties": {} }
    }
}"#;

#[test]
fn lenient_parsing_drops_only_invalid_objects() {
    let (desc, issues) = HomieDeviceDescription::from_json_lenient(DESCRIPTION).unwrap();

    let node = desc.get_node(&HomieID::new_const("relay")).unwrap();
    let props: Vec<&str> = node.properties.keys().map(|id| id.as_str()).collect();
    assert_eq!(props, ["power", "state"]);
    assert_eq!(desc.nodes.len(), 1);
    assert_eq!(desc.name.as_deref(), Some("Relay"));

    let mut dropped: Vec<(String, Option<String>)> =
        issues.iter().map(|i| (i.node_id.clone(), i.prop_id.clone())).collect();
    dropped.sort();
    assert_eq!(
        dropped,
        [
            ("Bad-Node".to_owned(), None),
            ("broken".to_owned(), None),
            ("relay".to_owned(), Some("Bad_Id".to_owned())),
            ("relay".to_owned(), Some("mode".to_owned())),
        ]
    );
    assert!(issues.iter().all(|i| !i.reason.is_empty()));
    assert!(issues
        .iter()
        .any(|i| i.to_string().starts_with("dropped property relay/mode")));
}

#[test]
fn strict_parsing_rejects_any_invalid_object() {
    assert!(matches!(
        HomieDeviceDescription::from_json_strict(DESCRIPTION),
        Err(Homie5ProtocolError::InvalidDeviceDescription)
    ));
}

#[test]
fn lenient_and_strict_agree_on_valid_descriptions() {
    let json =
        r#"{"homie":"5.0","version":1,"nodes":{"n":{"properties":{"p":{"datatype":"integer","format":"0:10"}}}}}"#;
    let strict = HomieDeviceDescription::from_json_strict(json).unwrap();
    let (lenient, issues) = HomieDeviceDescription::from_json_lenient(json).unwrap();
    assert!(issues.is_empty());
    assert_eq!(strict, lenient);
}

#[test]
fn invalid_device_frame_is_rejected() {
    for json in [
        r#"{"version":1}"#,
        r#"{"homie":"5.0"}"#,
        r#"{"homie":"5.0","version":1,"root":"Bad_Root"}"#,
        r#"{"homie":"5.0","version":1,"children":["Bad_Child"]}"#,
        "not json",
    ] {
        assert!(HomieDeviceDescription::from_json_lenient(json).is_err(), "{json}");
    }
}

#[test]
fn parser_reports_issues_with_description() {
    let message = parse_mqtt_message("homie/5/dev/$description", DESCRIPTION.as_bytes()).unwrap();
    let Homie5Message::DeviceDescription {
        description, issues, ..
    } = message
    else {
        panic!("expected a description, got {message:?}");
    };
    assert_eq!(issues.len(), 4);
    assert_eq!(description.nodes.len(), 1);
}

fn sample() -> HomieDeviceDescription {
    DeviceDescriptionBuilder::new()
        .name("Sample")
        .add_node(
            HomieID::new_const("node"),
            NodeDescriptionBuilder::new()
                .add_property(
                    HomieID::new_const("prop"),
                    PropertyDescriptionBuilder::integer().build(),
                )
                .build(),
        )
        .build()
}

#[test]
fn version_is_stable_and_content_based() {
    let mut a = sample();
    a.update_version();
    // Pinned value (FNV-1a over the canonical JSON, cross-checked outside Rust): changing it
    // means every deployed device publishes a new version.
    assert_eq!(a.version, 370_847_234_462_436_014);
    assert!(a.version >= 0);

    let mut b = sample();
    b.version = 42;
    b.update_version();
    assert_eq!(a.version, b.version, "version must not depend on the previous version");

    let mut c = sample();
    c.name = Some("Other".to_owned());
    c.update_version();
    assert_ne!(a.version, c.version);
}
