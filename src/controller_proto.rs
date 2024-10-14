use std::iter;

use crate::{
    client::{Publish, QoS, Subscription, Unsubscribe},
    device_description::{HomieDeviceDescription, HomiePropertyIterator},
    DeviceRef, HomieID, HomieValue, PropertyRef, ToTopic, DEFAULT_ROOT_TOPIC, DEVICE_ATTRIBUTES,
    DEVICE_ATTRIBUTE_ALERT, DEVICE_ATTRIBUTE_STATE, HOMIE_TOPIC_BROADCAST, HOMIE_VERSION, PROPERTY_ATTRIBUTE_TARGET,
    PROPERTY_SET_TOPIC,
};

/// Provides generators for all mqtt subscribe and publish packages needed for a homie5 controller
/// ### The general order for discovering devices is as follows:
/// * Start with the Subscriptions returned by [`Homie5ControllerProtocol::discover_devices`]
///   This will subscribe to the $state attribute of all devices.
/// * When receiving a [`Homie5Message::DeviceState`] message, check if the device is already
///   known, if not subscribe to the device using [`Homie5ControllerProtocol::subscribe_device`].
///   This will subscibe to all the other device attributes like $log/$description/$alert
/// * When receiving a [`Homie5Message::DeviceDescription`] message, store the description for the
///   device and subscibe to all the property values using
///   [`Homie5ControllerProtocol::subscribe_props`]
/// * after this you will start receiving [`Homie5Message::PropertyValue`] and
///   ['Homie5Message::PropertyTarget`] messages for the properties of the device
#[derive(Default)]
pub struct Homie5ControllerProtocol {}

impl Homie5ControllerProtocol {
    /// Create a new Homie5ControllerProtocol object
    pub fn new() -> Self {
        Default::default()
    }

    pub fn discover_devices<'a>(&'a self, topic_root: Option<&'a str>) -> impl Iterator<Item = Subscription> + 'a {
        let topic_root = topic_root.unwrap_or("+");
        iter::once(Subscription {
            topic: format!("{}/{}/+/{}", topic_root, HOMIE_VERSION, DEVICE_ATTRIBUTE_STATE),
            qos: QoS::ExactlyOnce,
        })
    }

    pub fn subscribe_device<'a>(&'a self, device: &'a DeviceRef) -> impl Iterator<Item = Subscription> + 'a {
        DEVICE_ATTRIBUTES
            .iter()
            .skip(1) // Skip the $state attribute (which must be first in the array)
            .map(move |attribute| {
                if *attribute == DEVICE_ATTRIBUTE_ALERT {
                    Subscription {
                        topic: format!("{}/{}/+", device.to_topic(), *attribute),
                        qos: QoS::ExactlyOnce,
                    }
                } else {
                    Subscription {
                        topic: format!("{}/{}", device.to_topic(), *attribute),
                        qos: QoS::ExactlyOnce,
                    }
                }
            })
    }

    pub fn unsubscribe_device<'a>(&'a self, device: &'a DeviceRef) -> impl Iterator<Item = Unsubscribe> + 'a {
        DEVICE_ATTRIBUTES.iter().skip(1).map(move |attribute| {
            if *attribute == DEVICE_ATTRIBUTE_ALERT {
                Unsubscribe {
                    topic: format!("{}/{}/+", device.to_topic(), *attribute),
                }
            } else {
                Unsubscribe {
                    topic: format!("{}/{}", device.to_topic(), *attribute),
                }
            }
        })
    }

    pub fn subscribe_props<'a>(
        &'a self,
        device: &'a DeviceRef,
        description: &'a HomieDeviceDescription,
    ) -> impl Iterator<Item = Subscription> + 'a {
        let prop_iter = HomiePropertyIterator::new(description);

        prop_iter.flat_map(move |(node_id, _, prop_id, _)| {
            [
                Subscription {
                    topic: format!("{}/{}/{}", device.to_topic(), node_id, prop_id),
                    qos: QoS::ExactlyOnce,
                },
                Subscription {
                    topic: format!(
                        "{}/{}/{}/{}",
                        device.to_topic(),
                        node_id,
                        prop_id,
                        PROPERTY_ATTRIBUTE_TARGET
                    ),
                    qos: QoS::ExactlyOnce,
                },
            ]
        })
    }

    pub fn unsubscribe_props<'a>(
        &'a self,
        device: &'a DeviceRef,
        description: &'a HomieDeviceDescription,
    ) -> impl Iterator<Item = Unsubscribe> + 'a {
        let prop_iter = HomiePropertyIterator::new(description);
        prop_iter.map(move |(node_id, _, prop_id, _)| Unsubscribe {
            topic: format!("{}/{}/{}", device.to_topic(), node_id, prop_id),
        })
    }

    pub fn set_command_ids(
        &self,
        topic_root: Option<&str>,
        device_id: &HomieID,
        node_id: &HomieID,
        prop_id: &HomieID,
        value: &HomieValue,
    ) -> Publish {
        let topic_root = if let Some(topic_root) = topic_root {
            topic_root
        } else {
            DEFAULT_ROOT_TOPIC
        };
        Publish {
            topic: format!(
                "{}/{}/{}/{}/{}/{}",
                topic_root, HOMIE_VERSION, device_id, node_id, prop_id, PROPERTY_SET_TOPIC
            ),
            qos: QoS::ExactlyOnce,
            retain: false,
            payload: value.to_string().into(),
        }
    }

    //pub fn set_command(&self, prop: &PropertyIdentifier, value: impl Into<String>) -> Publish {
    pub fn set_command(&self, prop: &PropertyRef, value: &HomieValue) -> Publish {
        self.set_command_ids(
            Some(&prop.node.device.topic_root),
            &prop.node.device.id,
            &prop.node.id,
            &prop.id,
            value,
        )
    }

    pub fn send_broadcast(
        &self,
        topic_root: Option<&str>,
        broadcast_topic: &str,
        broadcast_message: impl Into<String>,
    ) -> Publish {
        let topic_root = if let Some(topic_root) = topic_root {
            topic_root
        } else {
            DEFAULT_ROOT_TOPIC
        };
        Publish {
            topic: format!(
                "{}/{}/{}/{}",
                topic_root, HOMIE_VERSION, HOMIE_TOPIC_BROADCAST, broadcast_topic
            ),
            qos: QoS::ExactlyOnce,
            retain: false,
            payload: broadcast_message.into().into(),
        }
    }

    pub fn subscribe_broadcast<'a>(&'a self, topic_root: Option<&'a str>) -> impl Iterator<Item = Subscription> + 'a {
        let topic_root = topic_root.unwrap_or("+");
        iter::once(Subscription {
            topic: format!("{}/{}/$broadcast/#", topic_root, HOMIE_VERSION),
            qos: QoS::ExactlyOnce,
        })
    }
    pub fn unsubscribe_broadcast<'a>(&'a self, topic_root: Option<&'a str>) -> impl Iterator<Item = Unsubscribe> + 'a {
        let topic_root = topic_root.unwrap_or("+");
        iter::once(Unsubscribe {
            topic: format!("{}/{}/$broadcast/#", topic_root, HOMIE_VERSION),
        })
    }
}
