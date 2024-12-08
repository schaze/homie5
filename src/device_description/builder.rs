//! This module provides builders for constructing descriptions of Homie devices, nodes, and properties.
//! The builders allow for flexible and incremental building of `HomieDeviceDescription`,
//! `HomieNodeDescription`, and `HomiePropertyDescription` objects, commonly used in the Homie protocol.
//!
//! # Conditional Building
//!
//! All builders in this module offer the `do_if` method, which allows for executing a closure
//! only if a certain condition is met. This is useful for adding optional elements or applying logic
//! based on runtime conditions.
//!
//! ```
use crate::{HomieDataType, HomieID, HOMIE_VERSION_FULL};

use super::{
    property_format::HomiePropertyFormat, HomieDeviceDescription, HomieNodeDescription, HomiePropertyDescription,
    RETAINTED_DEFAULT, SETTABLE_DEFAULT,
};
use std::collections::{hash_map, HashMap};

/// Builder for constructing `HomieDeviceDescription` objects.
///
/// The `DeviceDescriptionBuilder` helps construct a complete `HomieDeviceDescription` by setting attributes
/// like children, nodes, extensions, and device metadata. It provides flexibility in handling device structure
/// and content, allowing for customization at each step.
///
/// ### Example Usage
/// ```rust
/// use homie5::device_description::*;
/// let device_description = DeviceDescriptionBuilder::new()
///     .name("MyDevice")
///     .add_child("node1".try_into().unwrap())
///     .add_extension("com.example.extension".to_string())
///     .build();
/// ```
pub struct DeviceDescriptionBuilder {
    description: HomieDeviceDescription,
}

impl Default for DeviceDescriptionBuilder {
    fn default() -> Self {
        DeviceDescriptionBuilder {
            description: HomieDeviceDescription {
                name: None,
                version: 0,
                homie: HOMIE_VERSION_FULL.to_owned(),
                children: Vec::new(),
                extensions: Vec::new(),
                nodes: HashMap::new(),
                parent: None,
                root: None,
            },
        }
    }
}

impl DeviceDescriptionBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_description(description: &HomieDeviceDescription) -> Self {
        DeviceDescriptionBuilder {
            description: description.clone(),
        }
    }

    pub fn build(mut self) -> HomieDeviceDescription {
        self.description.update_version();
        self.description
    }

    pub fn add_child(mut self, child_id: HomieID) -> Self {
        self.description.children.push(child_id);
        self
    }

    pub fn remove_child(mut self, child_id: &HomieID) -> Self {
        if let Some(pos) = self.description.children.iter().position(|x| x == child_id) {
            self.description.children.remove(pos);
        }
        self
    }

    pub fn replace_children(mut self, children: Vec<HomieID>) -> Self {
        self.description.children = children;
        self
    }

    pub fn add_extension(mut self, extension: impl Into<String>) -> Self {
        self.description.extensions.push(extension.into());
        self
    }

    pub fn parent(mut self, parent: impl Into<Option<HomieID>>) -> Self {
        self.description.parent = parent.into();
        self
    }

    pub fn root(mut self, parent: impl Into<Option<HomieID>>) -> Self {
        self.description.root = parent.into();
        self
    }

    pub fn name<S: Into<String>>(mut self, name: impl Into<Option<S>>) -> Self {
        self.description.name = name.into().map(Into::into);
        self
    }

    pub fn add_node(mut self, node_id: HomieID, node_desc: HomieNodeDescription) -> Self {
        self.description.nodes.insert(node_id, node_desc);
        self
    }

    pub fn do_if(self, condition: bool, cb: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            cb(self)
        } else {
            self
        }
    }

    pub fn remove_node(mut self, node_id: &HomieID) -> Self {
        self.description.nodes.remove(node_id);
        self
    }

    pub fn replace_or_insert_node(
        mut self,
        node_id: HomieID,
        f: impl FnOnce(Option<&HomieNodeDescription>) -> HomieNodeDescription,
    ) -> Self {
        let entry = self.description.nodes.entry(node_id);
        match entry {
            hash_map::Entry::Occupied(mut oe) => {
                let oe_mut = oe.get_mut();
                let new_desc = f(Some(oe_mut));
                *oe_mut = new_desc;
            }
            hash_map::Entry::Vacant(ve) => {
                let new_desc = f(None);
                ve.insert(new_desc);
            }
        }
        self
    }
}

/// Builder for constructing `HomieNodeDescription` objects.
///
/// The `NodeDescriptionBuilder` simplifies the creation of `HomieNodeDescription` instances.
/// Nodes are the intermediate layer between devices and properties, and this builder facilitates
/// adding properties, setting the node name and type, and optionally removing or replacing properties.
///
/// ### Example Usage
/// ```rust
/// use homie5::device_description::*;
/// let node_description = NodeDescriptionBuilder::new()
///     .name(Some("TemperatureNode".to_string()))
///     .r#type("sensor")
///     .build();
/// ```
pub struct NodeDescriptionBuilder {
    description: HomieNodeDescription,
}

impl Default for NodeDescriptionBuilder {
    fn default() -> Self {
        NodeDescriptionBuilder {
            description: HomieNodeDescription {
                name: None,
                r#type: None,
                properties: HashMap::new(),
            },
        }
    }
}

impl NodeDescriptionBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_description(description: &HomieNodeDescription) -> Self {
        NodeDescriptionBuilder {
            description: description.clone(),
        }
    }

    pub fn build(self) -> HomieNodeDescription {
        self.description
    }

    pub fn name(mut self, name: Option<String>) -> Self {
        self.description.name = name;
        self
    }

    pub fn r#type(mut self, v: impl Into<String>) -> Self {
        let s: String = v.into();
        self.description.r#type = if s.is_empty() { None } else { Some(s) };
        self
    }

    pub fn add_property(mut self, prop_id: HomieID, property_desc: HomiePropertyDescription) -> Self {
        self.description.properties.insert(prop_id, property_desc);
        self
    }

    pub fn do_if(self, condition: bool, cb: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            cb(self)
        } else {
            self
        }
    }

    pub fn add_property_cond(
        mut self,
        prop_id: HomieID,
        condition: bool,
        property_desc: impl FnOnce() -> HomiePropertyDescription,
    ) -> Self {
        if condition {
            self.description.properties.insert(prop_id, property_desc());
        }
        self
    }

    pub fn remove_property(mut self, prop_id: &HomieID) -> Self {
        self.description.properties.remove(prop_id);
        self
    }

    pub fn replace_or_insert_property(
        mut self,
        prop_id: HomieID,
        f: impl FnOnce(Option<&HomiePropertyDescription>) -> HomiePropertyDescription,
    ) -> Self {
        let entry = self.description.properties.entry(prop_id);
        match entry {
            hash_map::Entry::Occupied(mut oe) => {
                let oe_mut = oe.get_mut();
                let new_desc = f(Some(oe_mut));
                *oe_mut = new_desc;
            }
            hash_map::Entry::Vacant(ve) => {
                let new_desc = f(None);
                ve.insert(new_desc);
            }
        }
        self
    }
}

/// Builder for constructing `HomiePropertyDescription` objects.
///
/// The `PropertyDescriptionBuilder` is designed for constructing `HomiePropertyDescription`
/// objects, which represent individual properties of a node, such as sensor readings or settings.
/// Properties have attributes like datatype, format, settable, and retained, which can be set using
/// this builder.
///
/// ### Example Usage
/// ```rust
/// use homie5::device_description::*;
/// use homie5::*;
/// let property_description = PropertyDescriptionBuilder::new(HomieDataType::Float)
///     .name(Some("Temperature".to_string()))
///     .settable(false)
///     .retained(true)
///     .unit(Some("°C".to_string()))
///     .build();
/// ```
pub struct PropertyDescriptionBuilder {
    description: HomiePropertyDescription,
}

impl PropertyDescriptionBuilder {
    pub fn new(datatype: HomieDataType) -> Self {
        PropertyDescriptionBuilder {
            description: HomiePropertyDescription {
                name: None,
                datatype,
                format: HomiePropertyFormat::Empty,
                settable: SETTABLE_DEFAULT,
                retained: RETAINTED_DEFAULT,
                unit: None,
            },
        }
    }

    pub fn from_description(description: &HomiePropertyDescription) -> Self {
        PropertyDescriptionBuilder {
            description: description.clone(),
        }
    }

    pub fn do_if(self, condition: bool, cb: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            cb(self)
        } else {
            self
        }
    }
    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }

    pub fn format(mut self, format: HomiePropertyFormat) -> Self {
        self.description.format = format;
        self
    }

    pub fn name(mut self, name: Option<String>) -> Self {
        self.description.name = name;
        self
    }

    pub fn settable(mut self, settable: bool) -> Self {
        self.description.settable = settable;
        self
    }

    pub fn retained(mut self, retained: bool) -> Self {
        self.description.retained = retained;
        self
    }

    pub fn unit(mut self, unit: Option<String>) -> Self {
        self.description.unit = unit;
        self
    }

    pub fn datatype(mut self, datatype: HomieDataType) -> Self {
        self.description.datatype = datatype;
        self
    }
}
