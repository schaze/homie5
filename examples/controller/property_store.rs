use std::collections::HashMap;

use homie5::{HomieValue, NodeIdentifier, PropertyIdentifier};

#[derive(Debug, Clone, Default)]
pub struct PropertyState {
    value: Option<HomieValue>,  // Optional value
    target: Option<HomieValue>, // Optional target value
}

#[derive(Debug, Clone, Default)]
pub struct PropertyStore {
    properties: HashMap<PropertyIdentifier, PropertyState>, // Properties for this node
}

#[derive(Debug, Clone, Default)]
pub struct PropertyValueStore {
    nodes: HashMap<NodeIdentifier, PropertyStore>, // Nodes in the device
}

#[allow(dead_code)]
impl PropertyValueStore {
    pub fn new() -> Self {
        PropertyValueStore { nodes: HashMap::new() }
    }

    // Store both property value and target, or update if already exists
    pub fn store_property_value(
        &mut self,
        property: PropertyIdentifier,
        value: Option<HomieValue>,
        target: Option<HomieValue>,
    ) {
        let node_store = self.nodes.entry(property.node.clone()).or_default();
        let property_state = node_store.properties.entry(property).or_default();

        if let Some(v) = value {
            property_state.value = Some(v);
        }
        if let Some(t) = target {
            property_state.target = Some(t);
        }
    }

    // Get the current state of the property (returns value and target)
    pub fn get_property_state(&self, property: &PropertyIdentifier) -> Option<&PropertyState> {
        self.nodes
            .get(&property.node)
            .and_then(|node_store| node_store.properties.get(property))
    }

    // Get the current value of a property
    pub fn get_property_value(&self, property: &PropertyIdentifier) -> Option<&HomieValue> {
        self.get_property_state(property).and_then(|state| state.value.as_ref())
    }

    // Get the target value of a property
    pub fn get_property_target(&self, property: &PropertyIdentifier) -> Option<&HomieValue> {
        self.get_property_state(property)
            .and_then(|state| state.target.as_ref())
    }

    // Check if a property exists in the store
    pub fn property_exists(&self, property: &PropertyIdentifier) -> bool {
        self.nodes
            .get(&property.node)
            .and_then(|node_store| node_store.properties.get(property))
            .is_some()
    }

    // Remove a property from the store
    pub fn remove_property(&mut self, property: &PropertyIdentifier) {
        if let Some(node_store) = self.nodes.get_mut(&property.node) {
            node_store.properties.remove(property);
            if node_store.properties.is_empty() {
                self.nodes.remove(&property.node);
            }
        }
    }
}
