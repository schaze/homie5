use homie5::*;
use std::str::FromStr;

// Helper function to create a device identifier
fn create_device_identifier() -> DeviceRef {
    DeviceRef::new(Default::default(), "device1".try_into().unwrap())
}

// Helper function to create a node identifier
fn create_node_identifier() -> NodeRef {
    NodeRef::new(
        HomieDomain::Default,
        "device1".try_into().unwrap(),
        "node1".try_into().unwrap(),
    )
}

// Helper function to create a property identifier
fn create_property_identifier() -> PropertyRef {
    PropertyRef::new(
        HomieDomain::Default,
        "device1".try_into().unwrap(),
        "node1".try_into().unwrap(),
        "prop1".try_into().unwrap(),
    )
}

// ---- belongs_to tests ----

#[test]
fn test_node_belongs_to_device() {
    let device = create_device_identifier();
    let node = create_node_identifier();
    assert!(node.belongs_to(&device));
}

#[test]
fn test_node_not_belongs_to_different_device() {
    let different_device = DeviceRef::new(Default::default(), "device2".try_into().unwrap());
    let node = create_node_identifier();
    assert!(!node.belongs_to(&different_device));
}

#[test]
fn test_property_belongs_to_device() {
    let device = create_device_identifier();
    let property = create_property_identifier();
    assert!(property.belongs_to_device(&device));
}

#[test]
fn test_property_not_belongs_to_different_device() {
    let different_device = DeviceRef::new(Default::default(), "device2".try_into().unwrap());
    let property = create_property_identifier();
    assert!(!property.belongs_to_device(&different_device));
}

#[test]
fn test_property_belongs_to_node() {
    let node = create_node_identifier();
    let property = create_property_identifier();
    assert!(property.belongs_to_node(&node));
}

#[test]
fn test_property_not_belongs_to_different_node() {
    let different_node = NodeRef::new(
        Default::default(),
        "device1".try_into().unwrap(),
        "node2".try_into().unwrap(),
    );
    let property = create_property_identifier();
    assert!(!property.belongs_to_node(&different_node));
}

// ---- Same-type equality tests ----

#[test]
fn test_same_device_different_node() {
    let node1 = NodeRef::new(
        Default::default(),
        "device1".try_into().unwrap(),
        "node1".try_into().unwrap(),
    );
    let node2 = NodeRef::new(
        Default::default(),
        "device1".try_into().unwrap(),
        "node2".try_into().unwrap(),
    );
    assert_ne!(node1, node2);
}

#[test]
fn test_same_node_different_property() {
    let property1 = PropertyRef::new(
        Default::default(),
        "device1".try_into().unwrap(),
        "node1".try_into().unwrap(),
        "prop1".try_into().unwrap(),
    );
    let property2 = PropertyRef::new(
        Default::default(),
        "device1".try_into().unwrap(),
        "node1".try_into().unwrap(),
        "prop2".try_into().unwrap(),
    );
    assert_ne!(property1, property2);
}

// ---- Display / FromStr round-trip tests ----

#[test]
fn test_device_ref_display_fromstr() {
    let device = create_device_identifier();
    let display = device.to_string();
    assert_eq!(display, "homie/device1");

    let parsed: DeviceRef = display.parse().unwrap();
    assert_eq!(parsed, device);
}

#[test]
fn test_device_ref_fromstr_default_domain() {
    let parsed: DeviceRef = "device1".parse().unwrap();
    assert_eq!(parsed.homie_domain(), &HomieDomain::Default);
    assert_eq!(parsed.device_id().as_str(), "device1");
}

#[test]
fn test_node_ref_display_fromstr() {
    let node = create_node_identifier();
    let display = node.to_string();
    assert_eq!(display, "homie/device1/node1");

    let parsed: NodeRef = display.parse().unwrap();
    assert_eq!(parsed, node);
}

#[test]
fn test_node_ref_fromstr_default_domain() {
    let parsed: NodeRef = "device1/node1".parse().unwrap();
    assert_eq!(parsed.homie_domain(), &HomieDomain::Default);
    assert_eq!(parsed.device_id().as_str(), "device1");
    assert_eq!(parsed.node_id().as_str(), "node1");
}

#[test]
fn test_property_ref_display_fromstr() {
    let prop = create_property_identifier();
    let display = prop.to_string();
    assert_eq!(display, "homie/device1/node1/prop1");

    let parsed: PropertyRef = display.parse().unwrap();
    assert_eq!(parsed, prop);
}

#[test]
fn test_property_ref_fromstr_default_domain() {
    let parsed: PropertyRef = "device1/node1/prop1".parse().unwrap();
    assert_eq!(parsed.homie_domain(), &HomieDomain::Default);
    assert_eq!(parsed.device_id().as_str(), "device1");
    assert_eq!(parsed.node_id().as_str(), "node1");
    assert_eq!(parsed.prop_id().as_str(), "prop1");
}

#[test]
fn test_property_pointer_display_fromstr() {
    let pp = PropertyPointer::new("node1".try_into().unwrap(), "prop1".try_into().unwrap());
    let display = pp.to_string();
    assert_eq!(display, "node1/prop1");

    let parsed: PropertyPointer = display.parse().unwrap();
    assert_eq!(parsed, pp);
}

// ---- to_node_ref test ----

#[test]
fn test_property_to_node_ref() {
    let prop = create_property_identifier();
    let node = prop.to_node_ref();
    assert_eq!(node, create_node_identifier());
}

// ---- Serde round-trip tests ----

#[test]
fn test_device_ref_serde_roundtrip() {
    let device = create_device_identifier();
    let json = serde_json::to_string(&device).unwrap();
    assert_eq!(json, "\"homie/device1\"");
    let parsed: DeviceRef = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, device);
}

#[test]
fn test_node_ref_serde_roundtrip() {
    let node = create_node_identifier();
    let json = serde_json::to_string(&node).unwrap();
    assert_eq!(json, "\"homie/device1/node1\"");
    let parsed: NodeRef = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, node);
}

#[test]
fn test_property_ref_serde_roundtrip() {
    let prop = create_property_identifier();
    let json = serde_json::to_string(&prop).unwrap();
    assert_eq!(json, "\"homie/device1/node1/prop1\"");
    let parsed: PropertyRef = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, prop);
}

#[test]
fn test_property_pointer_serde_roundtrip() {
    let pp = PropertyPointer::new("node1".try_into().unwrap(), "prop1".try_into().unwrap());
    let json = serde_json::to_string(&pp).unwrap();
    assert_eq!(json, "\"node1/prop1\"");
    let parsed: PropertyPointer = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, pp);
}

// ---- Custom domain round-trip ----

#[test]
fn test_property_ref_custom_domain_roundtrip() {
    let prop = PropertyRef::new(
        HomieDomain::try_from("custom-domain".to_string()).unwrap(),
        "device1".try_into().unwrap(),
        "node1".try_into().unwrap(),
        "prop1".try_into().unwrap(),
    );
    let display = prop.to_string();
    assert_eq!(display, "custom-domain/device1/node1/prop1");

    let parsed: PropertyRef = display.parse().unwrap();
    assert_eq!(parsed, prop);
}

// ---- match_with helpers ----

#[test]
fn test_match_with_node() {
    let node = create_node_identifier();
    let prop = create_property_identifier();
    let prop_id: HomieID = "prop1".try_into().unwrap();
    assert!(prop.match_with_node(&node, &prop_id));
}

#[test]
fn test_match_with_device() {
    let device = create_device_identifier();
    let prop = create_property_identifier();
    let node_id: HomieID = "node1".try_into().unwrap();
    let prop_id: HomieID = "prop1".try_into().unwrap();
    assert!(prop.match_with_device(&device, &node_id, &prop_id));
}

// ---- FromStr error cases ----

#[test]
fn test_device_ref_fromstr_too_many_segments() {
    assert!(DeviceRef::from_str("a/b/c").is_err());
}

#[test]
fn test_node_ref_fromstr_too_few_segments() {
    assert!(NodeRef::from_str("single").is_err());
}

#[test]
fn test_property_ref_fromstr_too_few_segments() {
    assert!(PropertyRef::from_str("a/b").is_err());
}

#[test]
fn test_property_pointer_fromstr_wrong_segments() {
    assert!(PropertyPointer::from_str("single").is_err());
    assert!(PropertyPointer::from_str("a/b/c").is_err());
}
