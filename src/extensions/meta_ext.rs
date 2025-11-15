use core::iter;
use std::collections::HashMap;

use thiserror::Error;

use crate::{
    client::{mqtt_payload_to_string, Publish, QoS, Subscription},
    DeviceRef, Homie5DeviceProtocol, HomieDomain, HomieID, InvalidHomieDomainError, InvalidHomieIDError, NodeRef,
    PropertyRef, TopicBuilder, HOMIE_VERSION,
};

pub const EXT_META_ATTRIBUTE: &str = "$meta";
pub const EXT_TAGS_ATTRIBUTE: &str = "$tags";

#[derive(Debug, Error)]
pub enum MetaExtError {
    #[error("Error parsing MetaData")]
    InvalidMetaData(#[from] serde_json::Error),

    /// An MQTT message was received for a topic that does not conform to the Homie convention.
    #[error("Message for invalid homie MQTT topic received.")]
    InvalidTopic,

    /// Error occurred while converting a payload from bytes to UTF-8.
    ///
    /// This typically happens when the payload contains invalid UTF-8 bytes.
    #[error(transparent)]
    PayloadConversionError(#[from] std::string::FromUtf8Error),

    /// An invalid message payload was received.
    #[error("Invalid message payload received.")]
    InvalidPayload,

    /// The data provided does not confirm to the homie specification for a homie-domain
    #[error("Invalid homie domain: {0}")]
    InvalidHomieDomain(#[from] InvalidHomieDomainError),

    /// The data provided does not confirm to the homie specification for a homie id
    #[error("Invalid homie id: {0}")]
    InvalidHomieID(#[from] InvalidHomieIDError),
}

/// Represents the protocol implementation for the meta extension for a device, providing methods for
/// publishing and handling meta information
///
#[derive(Clone, Debug)]
pub struct MetaDeviceProtocol {
    id: HomieID,
    homie_domain: HomieDomain,
}

impl MetaDeviceProtocol {
    pub fn new(device_id: HomieID, homie_domain: HomieDomain) -> Self {
        Self {
            id: device_id,
            homie_domain,
        }
    }

    /// Returns the device's ID.
    pub fn id(&self) -> &HomieID {
        &self.id
    }

    /// Returns the domain in which the device is operating.
    pub fn homie_domain(&self) -> &HomieDomain {
        &self.homie_domain
    }

    /// Publishes the state for the given `device_id`.
    pub fn publish_meta_device(
        &self,
        device_id: &HomieID,
        meta: &HashMap<String, String>,
    ) -> Result<Publish, MetaExtError> {
        Ok(Publish {
            topic: TopicBuilder::new_for_device(&self.homie_domain, device_id)
                .add_attr(EXT_META_ATTRIBUTE)
                .build(),
            retain: true,
            payload: serde_json::to_string(meta)?.into(),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Publishes the state for the given `device_id` and `node_id`.
    pub fn publish_meta_node(
        &self,
        device_id: &HomieID,
        node_id: &HomieID,
        meta: &HashMap<String, String>,
    ) -> Result<Publish, MetaExtError> {
        Ok(Publish {
            topic: TopicBuilder::new_for_node(&self.homie_domain, device_id, node_id)
                .add_attr(EXT_META_ATTRIBUTE)
                .build(),
            retain: true,
            payload: serde_json::to_string(meta)?.into(),
            qos: QoS::ExactlyOnce,
        })
    }
    /// Publishes the state for the given `device_id` and `node_id`.
    pub fn publish_meta_property(
        &self,
        device_id: &HomieID,
        node_id: &HomieID,
        property_id: &HomieID,
        meta: &HashMap<String, String>,
    ) -> Result<Publish, MetaExtError> {
        Ok(Publish {
            topic: TopicBuilder::new_for_property(&self.homie_domain, device_id, node_id, property_id)
                .add_attr(EXT_META_ATTRIBUTE)
                .build(),
            retain: true,
            payload: serde_json::to_string(meta)?.into(),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Publishes the state for the given `device_id`.
    pub fn publish_tags_device(&self, device_id: &HomieID, tags: &Vec<String>) -> Result<Publish, MetaExtError> {
        Ok(Publish {
            topic: TopicBuilder::new_for_device(&self.homie_domain, device_id)
                .add_attr(EXT_TAGS_ATTRIBUTE)
                .build(),
            retain: true,
            payload: serde_json::to_string(tags)?.into(),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Publishes the state for the given `device_id` and `node_id`.
    pub fn publish_tags_node(
        &self,
        device_id: &HomieID,
        node_id: &HomieID,
        tags: &Vec<String>,
    ) -> Result<Publish, MetaExtError> {
        Ok(Publish {
            topic: TopicBuilder::new_for_node(&self.homie_domain, device_id, node_id)
                .add_attr(EXT_TAGS_ATTRIBUTE)
                .build(),
            retain: true,
            payload: serde_json::to_string(tags)?.into(),
            qos: QoS::ExactlyOnce,
        })
    }
    /// Publishes the state for the given `device_id` and `node_id`.
    pub fn publish_tags_property(
        &self,
        device_id: &HomieID,
        node_id: &HomieID,
        property_id: &HomieID,
        tags: &Vec<String>,
    ) -> Result<Publish, MetaExtError> {
        Ok(Publish {
            topic: TopicBuilder::new_for_property(&self.homie_domain, device_id, node_id, property_id)
                .add_attr(EXT_TAGS_ATTRIBUTE)
                .build(),
            retain: true,
            payload: serde_json::to_string(tags)?.into(),
            qos: QoS::ExactlyOnce,
        })
    }
}

impl From<&Homie5DeviceProtocol> for MetaDeviceProtocol {
    fn from(value: &Homie5DeviceProtocol) -> Self {
        Self {
            id: value.id().clone(),
            homie_domain: value.homie_domain().clone(),
        }
    }
}

/// ...
#[derive(Default)]
pub struct MetaControllerProtocol {}

impl MetaControllerProtocol {
    pub fn subscribe_for_device<'a>(&'a self, device: &'a DeviceRef) -> impl Iterator<Item = Subscription> + 'a {
        iter::once(Subscription {
            topic: format!(
                "{}/{}/{}/{}",
                device.homie_domain, HOMIE_VERSION, device.id, EXT_META_ATTRIBUTE
            ),
            qos: QoS::ExactlyOnce,
        })
    }
}

pub enum MetaExtMessage {
    DeviceMeta {
        device: DeviceRef,
        meta: HashMap<String, String>,
    },
    NodeMeta {
        node: NodeRef,
        meta: HashMap<String, String>,
    },
    PropertyMeta {
        property: PropertyRef,
        meta: HashMap<String, String>,
    },
    DeviceTags {
        device: DeviceRef,
        tags: Vec<String>,
    },
    NodeTags {
        node: NodeRef,
        tags: Vec<String>,
    },
    PropertyTags {
        property: PropertyRef,
        tags: Vec<String>,
    },
}

impl MetaExtMessage {
    pub fn from_mqtt_message(topic: &str, payload: &[u8]) -> Result<Self, MetaExtError> {
        // Split the topic into components based on '/' delimiter
        let tokens: Vec<&str> = topic.split('/').collect();

        // Ensure the topic contains at least 4 tokens and the last one is named $meta (e.g. "homie/5/device-id/$meta")
        if tokens.last() != Some(&EXT_META_ATTRIBUTE) || tokens.last() != Some(&EXT_TAGS_ATTRIBUTE) {
            return Err(MetaExtError::InvalidTopic);
        }

        let homie_domain: HomieDomain = tokens[0].to_owned().try_into()?;

        // check the homie id provided
        let device_id = tokens[2].to_string().try_into()?;

        match (tokens.len(), tokens.last()) {
            // Device meta
            // ===================
            (4, Some(&EXT_META_ATTRIBUTE)) => Ok(serde_json::from_str::<HashMap<String, String>>(
                &mqtt_payload_to_string(payload)?,
            )
            .map(|meta| Self::DeviceMeta {
                device: DeviceRef {
                    homie_domain,
                    id: device_id,
                },
                meta,
            })?),
            // Device tags
            // ===================
            (4, Some(&EXT_TAGS_ATTRIBUTE)) => Ok(serde_json::from_str::<Vec<String>>(&mqtt_payload_to_string(
                payload,
            )?)
            .map(|tags| Self::DeviceTags {
                device: DeviceRef {
                    homie_domain,
                    id: device_id,
                },
                tags,
            })?),
            // Node meta
            // ===================
            (5, Some(&EXT_META_ATTRIBUTE)) => {
                let node_id = tokens[3].to_string().try_into()?;

                Ok(
                    serde_json::from_str::<HashMap<String, String>>(&mqtt_payload_to_string(payload)?).map(|meta| {
                        Self::NodeMeta {
                            node: NodeRef::new(homie_domain, device_id, node_id),
                            meta,
                        }
                    })?,
                )
            }
            // Node tags
            // ===================
            (5, Some(&EXT_TAGS_ATTRIBUTE)) => {
                let node_id = tokens[3].to_string().try_into()?;

                Ok(
                    serde_json::from_str::<Vec<String>>(&mqtt_payload_to_string(payload)?).map(|tags| {
                        Self::NodeTags {
                            node: NodeRef::new(homie_domain, device_id, node_id),
                            tags,
                        }
                    })?,
                )
            }
            // Property meta
            // ===================
            (6, Some(&EXT_META_ATTRIBUTE)) => {
                let node_id = tokens[3].to_string().try_into()?;
                let property_id = tokens[4].to_string().try_into()?;

                Ok(
                    serde_json::from_str::<HashMap<String, String>>(&mqtt_payload_to_string(payload)?).map(|meta| {
                        Self::PropertyMeta {
                            property: PropertyRef::new(homie_domain, device_id, node_id, property_id),
                            meta,
                        }
                    })?,
                )
            }
            // Property tags
            // ===================
            (6, Some(&EXT_TAGS_ATTRIBUTE)) => {
                let node_id = tokens[3].to_string().try_into()?;
                let property_id = tokens[4].to_string().try_into()?;

                Ok(
                    serde_json::from_str::<Vec<String>>(&mqtt_payload_to_string(payload)?).map(|tags| {
                        Self::PropertyTags {
                            property: PropertyRef::new(homie_domain, device_id, node_id, property_id),
                            tags,
                        }
                    })?,
                )
            }
            _ => Err(MetaExtError::InvalidTopic),
        }
    }
}
