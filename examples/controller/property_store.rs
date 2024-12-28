use std::collections::HashMap;

use homie5::{HomieValue, PropertyPointer, PropertyRef};

#[derive(Debug, Clone, Default)]
pub struct PropertyState {
    value: Option<HomieValue>,  // Optional value
    target: Option<HomieValue>, // Optional target value
}

#[derive(Debug, Clone, Default)]
pub struct PropertyValueStore {
    values: HashMap<PropertyPointer, PropertyState>, // Nodes in the device
}

#[allow(dead_code)]
impl PropertyValueStore {
    pub fn new() -> Self {
        PropertyValueStore { values: HashMap::new() }
    }

    // Store both property value and target, or update if already exists
    pub fn store_property_value(
        &mut self,
        property: PropertyRef,
        value: Option<HomieValue>,
        target: Option<HomieValue>,
    ) {
        let property_state = self.values.entry(property.prop_pointer().clone()).or_default();

        if let Some(v) = value {
            property_state.value = Some(v);
        }
        if let Some(t) = target {
            property_state.target = Some(t);
        }
    }

    // Get the current state of the property (returns value and target)
    pub fn get_property_state(&self, property: &PropertyRef) -> Option<&PropertyState> {
        self.values.get(property.prop_pointer())
    }

    // Get the current value of a property
    pub fn get_property_value(&self, property: &PropertyRef) -> Option<&HomieValue> {
        self.get_property_state(property).and_then(|state| state.value.as_ref())
    }

    // Get the target value of a property
    pub fn get_property_target(&self, property: &PropertyRef) -> Option<&HomieValue> {
        self.get_property_state(property)
            .and_then(|state| state.target.as_ref())
    }

    // Check if a property exists in the store
    pub fn property_exists(&self, property: &PropertyRef) -> bool {
        self.values.contains_key(property.prop_pointer())
    }

    // Remove a property from the store
    pub fn remove_property(&mut self, property: &PropertyRef) {
        self.values.remove(property.prop_pointer());
    }
}
