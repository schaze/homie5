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
use super::property_format::{BooleanFormat, HomiePropertyFormat};
use super::{
    ColorFormat, FloatRange, HomieDeviceDescription, HomieNodeDescription, HomiePropertyDescription, IntegerRange,
    PropertyDescriptionValidationError, RETAINED_DEFAULT, SETTABLE_DEFAULT,
};
use crate::{HomieID, HOMIE_VERSION_FULL};
use std::collections::{btree_map, BTreeMap};
use std::marker::PhantomData;

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
                r#type: None,
                children: Vec::new(),
                extensions: Vec::new(),
                nodes: BTreeMap::new(),
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

    pub fn r#type<S: Into<String>>(mut self, r#type: impl Into<Option<S>>) -> Self {
        self.description.r#type = r#type.into().map(Into::into);
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
            btree_map::Entry::Occupied(mut oe) => {
                let oe_mut = oe.get_mut();
                let new_desc = f(Some(oe_mut));
                *oe_mut = new_desc;
            }
            btree_map::Entry::Vacant(ve) => {
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
///     .name("TemperatureNode")
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
                properties: BTreeMap::new(),
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

    pub fn name<S: Into<String>>(mut self, name: impl Into<Option<S>>) -> Self {
        self.description.name = name.into().map(Into::into);
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
            btree_map::Entry::Occupied(mut oe) => {
                let oe_mut = oe.get_mut();
                let new_desc = f(Some(oe_mut));
                *oe_mut = new_desc;
            }
            btree_map::Entry::Vacant(ve) => {
                let new_desc = f(None);
                ve.insert(new_desc);
            }
        }
        self
    }
}

pub struct PropertyBuilderInit;
pub struct IntegerProperty;
pub struct FloatProperty;
pub struct BooleanProperty;
pub struct StringProperty;
pub struct EnumProperty;
pub struct ColorProperty;
pub struct DatetimeProperty;
pub struct DurationProperty;
pub struct JsonProperty;

/// Builder for constructing `HomiePropertyDescription` objects.
///
/// This builder uses typestate markers to ensure only datatype-compatible format methods are available.
///
/// ### Example Usage
/// ```rust
/// use homie5::device_description::*;
/// use homie5::*;
/// let property_description = PropertyDescriptionBuilder::float()
///     .name("Temperature")
///     .settable(false)
///     .retained(true)
///     .unit(HOMIE_UNIT_DEGREE_CELSIUS)
///     .build();
/// ```
pub struct PropertyDescriptionBuilder<State = PropertyBuilderInit> {
    description: HomiePropertyDescription,
    _state: PhantomData<State>,
}

impl PropertyDescriptionBuilder<PropertyBuilderInit> {
    fn new_with(datatype: crate::HomieDataType) -> PropertyDescriptionBuilder<PropertyBuilderInit> {
        PropertyDescriptionBuilder {
            description: HomiePropertyDescription {
                name: None,
                datatype,
                format: HomiePropertyFormat::Empty,
                settable: SETTABLE_DEFAULT,
                retained: RETAINED_DEFAULT,
                unit: None,
            },
            _state: PhantomData,
        }
    }

    pub fn integer() -> PropertyDescriptionBuilder<IntegerProperty> {
        PropertyDescriptionBuilder {
            description: Self::new_with(crate::HomieDataType::Integer).description,
            _state: PhantomData,
        }
    }

    pub fn float() -> PropertyDescriptionBuilder<FloatProperty> {
        PropertyDescriptionBuilder {
            description: Self::new_with(crate::HomieDataType::Float).description,
            _state: PhantomData,
        }
    }

    pub fn boolean() -> PropertyDescriptionBuilder<BooleanProperty> {
        PropertyDescriptionBuilder {
            description: Self::new_with(crate::HomieDataType::Boolean).description,
            _state: PhantomData,
        }
    }

    pub fn string() -> PropertyDescriptionBuilder<StringProperty> {
        PropertyDescriptionBuilder {
            description: Self::new_with(crate::HomieDataType::String).description,
            _state: PhantomData,
        }
    }

    pub fn datetime() -> PropertyDescriptionBuilder<DatetimeProperty> {
        PropertyDescriptionBuilder {
            description: Self::new_with(crate::HomieDataType::Datetime).description,
            _state: PhantomData,
        }
    }

    pub fn duration() -> PropertyDescriptionBuilder<DurationProperty> {
        PropertyDescriptionBuilder {
            description: Self::new_with(crate::HomieDataType::Duration).description,
            _state: PhantomData,
        }
    }

    pub fn json() -> PropertyDescriptionBuilder<JsonProperty> {
        PropertyDescriptionBuilder {
            description: Self::new_with(crate::HomieDataType::JSON).description,
            _state: PhantomData,
        }
    }

    pub fn enumeration(
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<PropertyDescriptionBuilder<EnumProperty>, PropertyDescriptionValidationError> {
        let variants = collect_enum_values(values)?;
        Ok(PropertyDescriptionBuilder {
            description: HomiePropertyDescription {
                name: None,
                datatype: crate::HomieDataType::Enum,
                format: HomiePropertyFormat::Enum(variants),
                settable: SETTABLE_DEFAULT,
                retained: RETAINED_DEFAULT,
                unit: None,
            },
            _state: PhantomData,
        })
    }

    pub fn color(
        formats: impl IntoIterator<Item = ColorFormat>,
    ) -> Result<PropertyDescriptionBuilder<ColorProperty>, PropertyDescriptionValidationError> {
        let formats = collect_color_formats(formats)?;
        Ok(PropertyDescriptionBuilder {
            description: HomiePropertyDescription {
                name: None,
                datatype: crate::HomieDataType::Color,
                format: HomiePropertyFormat::Color(formats),
                settable: SETTABLE_DEFAULT,
                retained: RETAINED_DEFAULT,
                unit: None,
            },
            _state: PhantomData,
        })
    }
}

impl<State> PropertyDescriptionBuilder<State> {
    pub fn do_if(self, condition: bool, cb: impl FnOnce(Self) -> Self) -> Self {
        if condition {
            cb(self)
        } else {
            self
        }
    }

    pub fn name<S: Into<String>>(mut self, name: impl Into<Option<S>>) -> Self {
        self.description.name = name.into().map(|s| s.into());
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

    pub fn unit<S: Into<String>>(mut self, unit: impl Into<Option<S>>) -> Self {
        self.description.unit = unit.into().map(Into::into);
        self
    }
}

impl PropertyDescriptionBuilder<IntegerProperty> {
    pub fn integer_range(mut self, range: IntegerRange) -> Self {
        self.description.format = HomiePropertyFormat::IntegerRange(range);
        self
    }

    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

impl PropertyDescriptionBuilder<FloatProperty> {
    pub fn float_range(mut self, range: FloatRange) -> Self {
        self.description.format = HomiePropertyFormat::FloatRange(range);
        self
    }

    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

impl PropertyDescriptionBuilder<BooleanProperty> {
    pub fn boolean_labels(mut self, false_val: impl Into<String>, true_val: impl Into<String>) -> Self {
        self.description.format = HomiePropertyFormat::Boolean(BooleanFormat {
            false_val: false_val.into(),
            true_val: true_val.into(),
        });
        self
    }

    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

impl PropertyDescriptionBuilder<StringProperty> {
    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

impl PropertyDescriptionBuilder<EnumProperty> {
    pub fn enum_values(
        mut self,
        values: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, PropertyDescriptionValidationError> {
        let variants = collect_enum_values(values)?;
        self.description.format = HomiePropertyFormat::Enum(variants);
        Ok(self)
    }

    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

impl PropertyDescriptionBuilder<ColorProperty> {
    pub fn color_formats(
        mut self,
        formats: impl IntoIterator<Item = ColorFormat>,
    ) -> Result<Self, PropertyDescriptionValidationError> {
        let formats = collect_color_formats(formats)?;
        self.description.format = HomiePropertyFormat::Color(formats);
        Ok(self)
    }

    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

impl PropertyDescriptionBuilder<DatetimeProperty> {
    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

impl PropertyDescriptionBuilder<DurationProperty> {
    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

impl PropertyDescriptionBuilder<JsonProperty> {
    pub fn json_schema(mut self, schema: impl Into<String>) -> Self {
        self.description.format = HomiePropertyFormat::Json(schema.into());
        self
    }

    pub fn build(self) -> HomiePropertyDescription {
        self.description
    }
}

fn collect_enum_values(
    values: impl IntoIterator<Item = impl Into<String>>,
) -> Result<Vec<String>, PropertyDescriptionValidationError> {
    let mut result = Vec::new();
    for value in values {
        let value: String = value.into();
        if value.is_empty() {
            return Err(PropertyDescriptionValidationError::EnumContainsEmptyValue);
        }
        if result.contains(&value) {
            return Err(PropertyDescriptionValidationError::DuplicateEnumValues);
        }
        result.push(value);
    }

    if result.is_empty() {
        return Err(PropertyDescriptionValidationError::MissingEnumFormat);
    }
    Ok(result)
}

fn collect_color_formats(
    formats: impl IntoIterator<Item = ColorFormat>,
) -> Result<Vec<ColorFormat>, PropertyDescriptionValidationError> {
    let mut result = Vec::new();
    for format in formats {
        if result.contains(&format) {
            return Err(PropertyDescriptionValidationError::DuplicateColorFormats);
        }
        result.push(format);
    }

    if result.is_empty() {
        return Err(PropertyDescriptionValidationError::MissingColorFormat);
    }
    Ok(result)
}
