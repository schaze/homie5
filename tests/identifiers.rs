use homie5::*;

// Helper function to create a device identifier
fn create_device_identifier() -> DeviceIdentifier {
    DeviceIdentifier::new("homie".to_string(), "device1".to_string())
}

// Helper function to create a node identifier
fn create_node_identifier() -> NodeIdentifier {
    NodeIdentifier::new("homie".to_string(), "device1".to_string(), "node1".to_string())
}

// Helper function to create a property identifier
fn create_property_identifier() -> PropertyIdentifier {
    PropertyIdentifier::new(
        "homie".to_string(),
        "device1".to_string(),
        "node1".to_string(),
        "prop1".to_string(),
    )
}

// Test direct comparisons for PartialEq implementations

#[test]
fn test_device_identifier_partial_eq_with_property_identifier() {
    let device_id = create_device_identifier();
    let property_id = create_property_identifier();

    // Direct comparison without needing to manually access property.node.device
    assert_eq!(device_id, property_id);
}

#[test]
fn test_device_identifier_partial_eq_with_node_identifier() {
    let device_id = create_device_identifier();
    let node_id = create_node_identifier();

    // Direct comparison without manually accessing node.device
    assert_eq!(device_id, node_id);
}

#[test]
fn test_node_identifier_partial_eq_with_property_identifier() {
    let node_id = create_node_identifier();
    let property_id = create_property_identifier();

    // Direct comparison between node and property
    assert_eq!(node_id, property_id);
}

#[test]
fn test_property_identifier_partial_eq_with_device_identifier() {
    let property_id = create_property_identifier();
    let device_id = create_device_identifier();

    // Direct comparison from property to device
    assert_eq!(property_id, device_id);
}

#[test]
fn test_property_identifier_partial_eq_with_node_identifier() {
    let property_id = create_property_identifier();
    let node_id = create_node_identifier();

    // Direct comparison from property to node
    assert_eq!(property_id, node_id);
}

#[test]
fn test_device_identifier_partial_eq_ref_with_property_identifier() {
    let device_id = create_device_identifier();
    let property_id = create_property_identifier();

    // Direct comparison between references
    assert_eq!(&device_id, &property_id);
}

#[test]
fn test_device_identifier_partial_eq_ref_with_node_identifier() {
    let device_id = create_device_identifier();
    let node_id = create_node_identifier();

    // Direct comparison between references
    assert_eq!(&device_id, &node_id);
}

// Test mismatches to ensure inequality works correctly
#[test]
fn test_device_identifier_not_equal_to_different_property_identifier() {
    let device_id = create_device_identifier();
    let different_property_id = PropertyIdentifier::new(
        "homie".to_string(),
        "device2".to_string(),
        "node1".to_string(),
        "prop1".to_string(),
    );

    // Ensure that mismatching device_id results in inequality
    assert_ne!(device_id, different_property_id);
}

#[test]
fn test_node_identifier_not_equal_to_different_property_identifier() {
    let node_id = create_node_identifier();
    let different_property_id = PropertyIdentifier::new(
        "homie".to_string(),
        "device1".to_string(),
        "node2".to_string(),
        "prop1".to_string(),
    );

    // Ensure that mismatching node_id results in inequality
    assert_ne!(node_id, different_property_id);
}

#[test]
fn test_property_identifier_not_equal_to_different_device_identifier() {
    let property_id = create_property_identifier();
    let different_device_id = DeviceIdentifier::new("homie".to_string(), "device2".to_string());

    // Ensure that mismatching property device_id results in inequality
    assert_ne!(property_id, different_device_id);
}

// Test same device with different node
#[test]
fn test_same_device_different_node() {
    let node1 = NodeIdentifier::new("homie".to_string(), "device1".to_string(), "node1".to_string());
    let node2 = NodeIdentifier::new("homie".to_string(), "device1".to_string(), "node2".to_string());

    // Same device but different nodes should not be equal
    assert_ne!(node1, node2);
}

// Test same node with different property
#[test]
fn test_same_node_different_property() {
    let property1 = PropertyIdentifier::new(
        "homie".to_string(),
        "device1".to_string(),
        "node1".to_string(),
        "prop1".to_string(),
    );
    let property2 = PropertyIdentifier::new(
        "homie".to_string(),
        "device1".to_string(),
        "node1".to_string(),
        "prop2".to_string(),
    );

    // Same node but different properties should not be equal
    assert_ne!(property1, property2);
}