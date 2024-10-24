//! This module provides all types and tools to create (builders) and manage homie device, node and property
//! descriptions.
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::iter::Iterator;
use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{HomieDataType, HomieID, NodeRef, PropertyRef};

mod builder;
mod number_ranges;
mod property_format;

pub use builder::*;
pub use number_ranges::*;
pub use property_format::*;

pub const SETTABLE_DEFAULT: bool = false;
pub const RETAINTED_DEFAULT: bool = true;

/// PropertyDescription
///
/// The Property object has the following fields:
///
/// |Property   | Type         | Required | Default  | Nullable | Description |
/// |-----------|--------------|----------|----------|----|---------|
/// | name      | string       | no      |          | no | Friendly name of the Property. |
/// | datatype  | string       | yes      |          | no | The data type. See [Payloads](#payload). Any of the following values: `"integer", "float", "boolean", "string", "enum", "color", "datetime", "duration", "json"`. |
/// | format    | string       | see [formats](#formats)    | see [formats](#formats) | no | Specifies restrictions or options for the given data type. |
/// | settable  | boolean      | no       | `false`  | no | Whether the Property is settable. |
/// | retained  | boolean      | no       | `true`   | no | Whether the Property is retained. |
/// | unit      | string       | no       |          | no | Unit of this property. See [units](#units). |
///
///
/// For example, our `temperature` property would look like this in the device/node description document:
///
/// ```json
///       ...
///       "temperature": {
///         "name": "Engine temperature",
///         "unit": "°C",
///         "datatype": "float",
///         "format": "-20:120"
///       }
///       ...
/// ```
#[derive(Debug, Serialize, Clone, Hash, PartialEq)]
pub struct HomiePropertyDescription {
    pub name: Option<String>,
    pub datatype: HomieDataType,
    #[serde(
        skip_serializing_if = "HomiePropertyFormat::format_is_empty",
        serialize_with = "serialize_format"
    )]
    pub format: HomiePropertyFormat,
    #[serde(default = "serde_default_settable")]
    pub settable: bool,
    #[serde(default = "serde_default_retained")]
    pub retained: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

fn serde_default_settable() -> bool {
    SETTABLE_DEFAULT
}

fn serde_default_retained() -> bool {
    RETAINTED_DEFAULT
}

fn serialize_format<S>(item: &HomiePropertyFormat, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let f_str = &item.to_string();
    if !f_str.is_empty() {
        serializer.serialize_str(f_str)
    } else {
        serializer.serialize_none()
    }
}

// Implement custom deserialization for the PropertyDescription.
// This is required as the `format` field can only be properly parsed when the datatype field is
// known
impl<'de> Deserialize<'de> for HomiePropertyDescription {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize into a temporary struct -- Keep this the same as HomiePropertyDescription
        // except for the format field!
        #[derive(Debug, Serialize, Deserialize, Clone, Hash)]
        pub struct TempDescription {
            pub name: Option<String>,
            pub datatype: HomieDataType,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub format: Option<String>,
            #[serde(default = "serde_default_settable")]
            pub settable: bool,
            #[serde(default = "serde_default_retained")]
            pub retained: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub unit: Option<String>,
        }

        let temp = TempDescription::deserialize(deserializer)?;

        let format = if let Some(f) = temp.format {
            match HomiePropertyFormat::parse(&f, &temp.datatype) {
                Ok(format) => format,
                Err(err) => {
                    return Err(serde::de::Error::custom(err));
                }
            }
        } else {
            HomiePropertyFormat::Empty
        };

        Ok(HomiePropertyDescription {
            name: temp.name,
            datatype: temp.datatype,
            format,
            settable: temp.settable,
            retained: temp.retained,
            unit: temp.unit,
        })
    }
}
/// HomieNodeDescription
///
/// The Node object has the following fields:
///
/// |Property   | Type         | Required | Default | Nullable | Description |
/// |-----------|--------------|----------|---------|----------|-------------|
/// | name      |string        | no      |         | no       | Friendly name of the Node. |
/// | properties|object        | no       | `{}`    | no       | The [Properties](#properties) the Node exposes. An object containing the [Properties](#properties), indexed by their [ID](#topic-ids). Defaults to an empty object.|
///
/// For example, our `engine` node would look like this:
///
/// ```json
///       ...
///       "engine": {
///         "name": "Car engine",
///         "properties": {
///           "speed": { ... },
///           "direction": { ... },
///           "temperature": { ... }
///         }
///       }
///       ...
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HomieNodeDescription {
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(default, skip_serializing_if = "serde_skip_if_properties")]
    pub properties: HashMap<HomieID, HomiePropertyDescription>,
}
impl HomieNodeDescription {
    pub fn with_property<T>(
        &self,
        property: &PropertyRef,
        f: impl FnOnce(&HomiePropertyDescription) -> T,
    ) -> Option<T> {
        self.with_property_by_id(&property.id, f)
    }

    pub fn with_property_by_id<T>(
        &self,
        prop_id: &HomieID,
        f: impl FnOnce(&HomiePropertyDescription) -> T,
    ) -> Option<T> {
        if let Some(prop) = self.properties.get(prop_id) {
            return Some(f(prop));
        }
        None
    }
}
impl Hash for HomieNodeDescription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.r#type.hash(state);

        // Hashing HashMap contents in a deterministic order
        let mut keys: Vec<_> = self.properties.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(state);
            self.properties.get(key).unwrap().hash(state);
        }
    }
}

#[allow(dead_code)]
fn null_to_default<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let key = Option::<T>::deserialize(de)?;
    Ok(key.unwrap_or_default())
}

/// If the properties HashMap is empty, skip serializing the field
fn serde_skip_if_properties(properties: &HashMap<HomieID, HomiePropertyDescription>) -> bool {
    properties.is_empty()
}

pub type HomieNodes = HashMap<HomieID, HomieNodeDescription>;
/// HomieDeviceDescription
///
/// The JSON description document has the following format:
///
/// | Property  | Type         | Required | Default | Nullable | Description |
/// |-----------|--------------|----------|---------|----------|-------------|
/// | homie     |string        | yes      |         | no       | The implemented Homie convention version, without the "patch" level. So the format is `"5.x"`, where the `'x'` is the minor version. |
/// | version   | integer      | yes      |         | no       | The version of the description document. Whenever the document changes, a new higher version must be assigned. This does not need to be sequential, eg. a timestamp could be used. |
/// | nodes     |object        | no       | `{}`    | no       | The [Nodes](#nodes) the device exposes. An object containing the [Nodes](#nodes), indexed by their [ID](#topic-ids). Defaults to an empty object.|
/// | name      |string        | no      |         | no       | Friendly name of the device. |
/// | children  |array-strings | no       | `[]`    | no       | Array of [ID](#topic-ids)'s of child devices. Defaults to an empty array.|
/// | root      |string        | yes/no   |         | no       | [ID](#topic-ids) of the root parent device. **Required** if the device is NOT the root device, MUST be omitted otherwise. |
/// | parent    |string        | yes/no   | same as `root`| no | [ID](#topic-ids) of the parent device. **Required** if the parent is NOT the root device. Defaults to the value of the `root` property. |
/// | extensions|array-strings | no       | `[]`    | no       | Array of supported extensions. Defaults to an empty array.|
///
/// For example, a device with an ID of `super-car` that comprises of a `wheels`, `engine`, and a `lights` node would send:
/// ```java
/// homie/5/super-car/$state → "init"
/// homie/5/super-car/$description → following JSON document;
/// ```
/// ```json
///       {
///         "homie": "5.0",
///         "name": "Supercar",
///         "version": 7,
///         "nodes": {
///           "wheels": { ... },
///           "engine": { ... },
///           "lights": { ... }
///         }
///       }
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HomieDeviceDescription {
    pub name: Option<String>,
    pub version: i64,
    pub homie: String,
    #[serde(default = "serde_default_list", skip_serializing_if = "serde_skip_if_empty_list")]
    pub children: Vec<HomieID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<HomieID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<HomieID>,
    #[serde(default = "serde_default_list", skip_serializing_if = "serde_skip_if_empty_list")]
    pub extensions: Vec<String>,
    #[serde(default, skip_serializing_if = "serde_skip_if_nodes")]
    pub nodes: HomieNodes,
}

impl Default for HomieDeviceDescription {
    fn default() -> Self {
        HomieDeviceDescription {
            name: None,
            version: 0,
            homie: "5.0".to_owned(),
            children: Vec::new(),
            root: None,
            parent: None,
            extensions: Vec::new(),
            nodes: HashMap::new(),
        }
    }
}

impl HomieDeviceDescription {
    //pub fn with_node<T>(&self, node_id: impl Into<String>, f: impl FnOnce(Option<&HomieNodeDescription>) -> T) -> T {
    //    let node_id = node_id.into();
    //    let result = f(self.nodes.get(&node_id));
    //    result
    //}

    pub fn with_node_by_id<T>(&self, node_id: &HomieID, f: impl FnOnce(&HomieNodeDescription) -> T) -> Option<T> {
        if let Some(node) = self.nodes.get(node_id) {
            return Some(f(node));
        }
        None
    }

    pub fn with_node<T>(&self, node: &NodeRef, f: impl FnOnce(&HomieNodeDescription) -> T) -> Option<T> {
        self.with_node_by_id(&node.id, f)
    }

    pub fn get_node<T>(&self, node: &NodeRef) -> Option<&HomieNodeDescription> {
        self.nodes.get(&node.id)
    }

    pub fn with_property_by_id<T>(
        &self,
        node_id: &HomieID,
        prop_id: &HomieID,
        f: impl FnOnce(&HomiePropertyDescription) -> T,
    ) -> Option<T> {
        if let Some(node) = self.nodes.get(node_id) {
            if let Some(prop) = node.properties.get(prop_id) {
                return Some(f(prop));
            }
        }
        None
    }

    pub fn with_property<T>(
        &self,
        property: &PropertyRef,
        f: impl FnOnce(&HomiePropertyDescription) -> T,
    ) -> Option<T> {
        if let Some(prop) = self
            .nodes
            .get(&property.node.id)
            .and_then(|node| node.properties.get(&property.id))
        {
            return Some(f(prop));
        }
        None
    }
    pub fn get_property(&self, property: &PropertyRef) -> Option<&HomiePropertyDescription> {
        if let Some(prop) = self
            .nodes
            .get(&property.node.id)
            .and_then(|node| node.properties.get(&property.id))
        {
            return Some(prop);
        }
        None
    }

    pub fn update_version(&mut self) {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        self.version = i64::from_ne_bytes(hash.to_ne_bytes());
    }

    pub fn add_child(&mut self, child_id: HomieID) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    pub fn remove_child(&mut self, child_id: &HomieID) {
        if let Some(index) = self.children.iter().position(|x| x == child_id) {
            self.children.swap_remove(index);
        }
    }

    pub fn iter(&self) -> HomiePropertyIterator {
        HomiePropertyIterator::new(self)
    }
}

impl Hash for HomieDeviceDescription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.homie.hash(state);
        self.children.hash(state);
        self.root.hash(state);
        self.parent.hash(state);
        self.extensions.hash(state);

        // Hashing HashMap contents in a deterministic order
        let mut keys: Vec<_> = self.nodes.keys().collect();
        keys.sort();
        for key in keys {
            key.hash(state);
            self.nodes.get(key).unwrap().hash(state);
        }
    }
}

/// If the nodes HashMap is empty, skip serializing the field
fn serde_skip_if_nodes(nodes: &HashMap<HomieID, HomieNodeDescription>) -> bool {
    nodes.is_empty()
}

fn serde_default_list<T>() -> Vec<T> {
    vec![]
}

fn serde_skip_if_empty_list<T>(children: &[T]) -> bool {
    children.is_empty()
}

pub struct HomiePropertyIterator<'a> {
    _device: &'a HomieDeviceDescription,
    node_iter: std::collections::hash_map::Iter<'a, HomieID, HomieNodeDescription>,
    current_node: Option<(&'a HomieID, &'a HomieNodeDescription)>,
    property_iter: Option<std::collections::hash_map::Iter<'a, HomieID, HomiePropertyDescription>>,
}

impl<'a> HomiePropertyIterator<'a> {
    pub fn new(_device: &'a HomieDeviceDescription) -> Self {
        let mut node_iter = _device.nodes.iter();

        let first_node = node_iter.next();

        let (current_node_id, property_iter) = match first_node {
            Some(node) => (Some(node), Some(node.1.properties.iter())),
            None => (None, None),
        };

        HomiePropertyIterator {
            _device,
            node_iter,
            current_node: current_node_id,
            property_iter,
        }
    }
}

impl<'a> Iterator for HomiePropertyIterator<'a> {
    type Item = (
        &'a HomieID,
        &'a HomieNodeDescription,
        &'a HomieID,
        &'a HomiePropertyDescription,
    );

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(iter) = self.property_iter.as_mut() {
                if let Some((property_id, property)) = iter.next() {
                    return Some((
                        self.current_node.unwrap().0,
                        self.current_node.unwrap().1,
                        property_id,
                        property,
                    ));
                }
            }

            match self.node_iter.next() {
                Some((node_id, node)) => {
                    self.current_node = Some((node_id, node));
                    self.property_iter = Some(node.properties.iter())
                }
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::device_description::number_ranges::FloatRange;

    use super::*;

    #[test]
    fn test_deserialization() {
        let data = serde_json::from_str::<HomiePropertyDescription>(
            r#"
            {
                "name": "Test",
                "datatype": "float",
                "format": "1.02:5.55:0.45"
            }
            "#,
        );
        assert_eq!(
            data.as_ref().unwrap(),
            &HomiePropertyDescription {
                name: Some("Test".to_string()),
                datatype: HomieDataType::Float,
                format: HomiePropertyFormat::FloatRange(FloatRange {
                    min: Some(1.02),
                    max: Some(5.55),
                    step: Some(0.45)
                }),
                settable: SETTABLE_DEFAULT,
                retained: RETAINTED_DEFAULT,
                unit: None,
            }
        );
        println!("DESERIALIZATION: {:#?}", data);
        if let Ok(data) = data {
            let s_f = serde_json::to_string(&data);
            println!("SERIALIZATION: {:#?}", s_f);
            //println!(
            //    "RE-Deserialize: {:#?}",
            //    serde_json::from_str::<HomiePropertyDescription>(&s_f)
            //);
        }
    }
    #[test]
    fn test_format_float() {
        assert_eq!(
            HomiePropertyFormat::parse("::5", &HomieDataType::Float),
            Ok(HomiePropertyFormat::FloatRange(FloatRange {
                min: None,
                max: None,
                step: Some(5.0)
            }))
        );
        assert_eq!(
            HomiePropertyFormat::parse("5:11:3", &HomieDataType::Float),
            Ok(HomiePropertyFormat::FloatRange(FloatRange {
                min: Some(5.0),
                max: Some(11.0),
                step: Some(3.0)
            }))
        );
        assert_eq!(
            HomiePropertyFormat::parse(":5", &HomieDataType::Float),
            Ok(HomiePropertyFormat::FloatRange(FloatRange {
                min: None,
                max: Some(5.0),
                step: None
            }))
        );
        assert_eq!(
            HomiePropertyFormat::parse("1::2", &HomieDataType::Float),
            Ok(HomiePropertyFormat::FloatRange(FloatRange {
                min: Some(1.0),
                max: None,
                step: Some(2.0)
            }))
        );
        assert_eq!(
            HomiePropertyFormat::parse("2", &HomieDataType::Float),
            Ok(HomiePropertyFormat::FloatRange(FloatRange {
                min: Some(2.0),
                max: None,
                step: None
            }))
        );
    }
}
