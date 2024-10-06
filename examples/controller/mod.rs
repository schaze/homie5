pub mod property_store;
use anyhow::anyhow;
use homie5::{
    device_description::HomieDeviceDescription, DeviceIdentifier, Homie5ProtocolError, HomieDeviceStatus, HomieValue,
    PropertyIdentifier, ToTopic,
};
pub use property_store::*;

/// Represents a discovered device.
/// Note, that we do not store property values so far
#[allow(dead_code)]
pub struct Device {
    pub ident: DeviceIdentifier,
    pub state: HomieDeviceStatus,
    pub description: Option<HomieDeviceDescription>,
    pub properties: PropertyValueStore,
}

impl Device {
    pub fn store_value(&mut self, property: PropertyIdentifier, value: String) -> anyhow::Result<()> {
        let Some(desc) = self.description.as_ref() else {
            return Ok(());
        };

        if !self.is_retained(&property, desc) {
            return Ok(());
        }
        let value = self.parse_value(&property, value)?;

        self.properties.store_property_value(property, Some(value), None);
        Ok(())
    }

    pub fn store_target(&mut self, property: PropertyIdentifier, value: String) -> anyhow::Result<()> {
        let Some(desc) = self.description.as_ref() else {
            return Ok(());
        };

        if !self.is_retained(&property, desc) {
            return Ok(());
        }
        let value = self.parse_value(&property, value)?;

        self.properties.store_property_value(property, None, Some(value));
        Ok(())
    }
    fn is_retained(&self, property: &PropertyIdentifier, desc: &HomieDeviceDescription) -> bool {
        // get the retained setting for the property
        let Ok(retained) = desc.with_property(property, |prop| prop.retained).ok_or_else(|| {
            log::debug!("Cannot set value for: {}", property.to_topic());
            Homie5ProtocolError::PropertyNotFound
        }) else {
            return false;
        };

        retained
    }

    fn parse_value(&mut self, property: &PropertyIdentifier, value: String) -> anyhow::Result<HomieValue> {
        let Some(desc) = self.description.as_ref() else {
            return Err(anyhow!("Cannot parse value for device without description!"));
        };

        let value = desc
            .with_property(property, |prop| HomieValue::parse(&value, prop))
            .ok_or_else(|| {
                log::debug!("Cannot set value for: {}", property.to_topic());
                Homie5ProtocolError::PropertyNotFound
            })?
            .map_err(|err| {
                log::debug!(
                    "Invalid value provided for property: {} -- {:?}",
                    property.to_topic(),
                    err
                );
                Homie5ProtocolError::InvalidTopic
            })?;
        Ok(value)
    }
}
