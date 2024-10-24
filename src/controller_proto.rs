//! Provides MQTT subscription and publish message generators for a Homie 5 controller.
//!
//! This module implements the `Homie5ControllerProtocol` which generates MQTT messages necessary for discovering and controlling Homie 5 devices.
//! The protocol manages the lifecycle of device discovery, subscribing to device attributes, and handling property changes.
//!
//! # Device Discovery Flow
//! 1. Start with the Subscriptions returned by [`Homie5ControllerProtocol::discover_devices`]
//!    This will subscribe to the $state attribute of all devices.
//! 2. When receiving a [`Homie5Message::DeviceState`] message, check if the device is already
//!    known, if not subscribe to the device using [`Homie5ControllerProtocol::subscribe_device`].
//!    This will subscibe to all the other device attributes like $log/$description/$alert
//! 3. When receiving a [`Homie5Message::DeviceDescription`] message, store the description for the
//!    device and subscibe to all the property values using
//!    [`Homie5ControllerProtocol::subscribe_props`]
//! 4. after this you will start receiving [`Homie5Message::PropertyValue`] and
//!    ['Homie5Message::PropertyTarget`] messages for the properties of the device
//!

use std::iter;

use crate::{
    client::{Publish, QoS, Subscription, Unsubscribe},
    device_description::{HomieDeviceDescription, HomiePropertyIterator},
    DeviceRef, HomieDomain, HomieID, HomieValue, PropertyRef, ToTopic, DEVICE_ATTRIBUTES, DEVICE_ATTRIBUTE_ALERT,
    DEVICE_ATTRIBUTE_STATE, HOMIE_TOPIC_BROADCAST, HOMIE_VERSION, PROPERTY_ATTRIBUTE_TARGET, PROPERTY_SET_TOPIC,
};

/// The `Homie5ControllerProtocol` struct provides the core functionality for generating MQTT subscription and publish commands required for interacting with Homie 5 devices.
///
/// This struct simplifies the process of discovering devices, subscribing to device attributes, handling device property changes, and sending commands or broadcasts to devices in the Homie MQTT protocol. It supports managing device discovery, subscriptions to device states and properties, and sending set commands and broadcast messages.
///
/// The `Homie5ControllerProtocol` is designed for use in Homie-based IoT controllers where efficient communication with multiple devices is required. It offers a straightforward interface for subscribing to device attributes and managing device properties.
///
/// # Usage
///
/// To create an instance of `Homie5ControllerProtocol`, use the `new` method. This struct provides methods for discovering devices, subscribing to device attributes and properties, and sending commands or broadcasts via MQTT topics.
///
/// # Example
///
/// ```rust
/// use homie5::{Homie5ControllerProtocol, HomieDomain};
///
/// let protocol = Homie5ControllerProtocol::new();
/// let subscriptions = protocol.discover_devices(&HomieDomain::Default);
/// for subscription in subscriptions {
///     // Handle each subscription
/// }
/// ```
///
/// The struct is intended for use in controller applications interacting with multiple Homie devices, enabling efficient subscription management and MQTT communication.
#[derive(Default)]
pub struct Homie5ControllerProtocol {}

impl Homie5ControllerProtocol {
    /// Creates a new `Homie5ControllerProtocol` instance.
    ///
    /// # Returns
    /// A new `Homie5ControllerProtocol` object initialized with default values.
    pub fn new() -> Self {
        Default::default()
    }

    /// Generates a subscription to discover Homie devices by subscribing to the `$state` attribute of all devices.
    ///
    /// # Parameters
    /// - `homie_domain`: The Homie domain in which to discover devices.
    ///
    /// # Returns
    /// An iterator over a `Subscription` object that subscribes to the `$state` attribute of all devices in the specified domain.
    pub fn discover_devices<'a>(&'a self, homie_domain: &HomieDomain) -> impl Iterator<Item = Subscription> + 'a {
        iter::once(Subscription {
            topic: format!("{}/{}/+/{}", homie_domain, HOMIE_VERSION, DEVICE_ATTRIBUTE_STATE),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Generates subscriptions for all attributes of a specified device, excluding `$state`.
    ///
    /// # Parameters
    /// - `device`: A reference to the `DeviceRef` that identifies the device.
    ///
    /// # Returns
    /// An iterator over `Subscription` objects for the device's attributes (e.g., `$log`, `$description`, `$alert`).
    pub fn subscribe_device<'a>(&'a self, device: &'a DeviceRef) -> impl Iterator<Item = Subscription> + 'a {
        DEVICE_ATTRIBUTES
            .iter()
            .skip(1) // Skip the $state attribute
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

    /// Generates unsubscribe requests for all attributes of a specified device, excluding `$state`.
    ///
    /// # Parameters
    /// - `device`: A reference to the `DeviceRef` that identifies the device.
    ///
    /// # Returns
    /// An iterator over `Unsubscribe` objects for the device's attributes (e.g., `$log`, `$description`, `$alert`).
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

    /// Subscribes to all properties of a device as described in the provided `HomieDeviceDescription`.
    ///
    /// # Parameters
    /// - `device`: A reference to the `DeviceRef` that identifies the device.
    /// - `description`: A reference to the `HomieDeviceDescription` that describes the device's properties.
    ///
    /// # Returns
    /// An iterator over `Subscription` objects for the device's properties.
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

    /// Unsubscribes from all properties of a device based on its `HomieDeviceDescription`.
    ///
    /// # Parameters
    /// - `device`: A reference to the `DeviceRef` that identifies the device.
    /// - `description`: A reference to the `HomieDeviceDescription` that describes the device's properties.
    ///
    /// # Returns
    /// An iterator over `Unsubscribe` objects for the device's properties.
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

    /// Publishes a set command to change a property's value.
    ///
    /// # Parameters
    /// - `homie_domain`: The Homie domain in which the device is located.
    /// - `device_id`: The ID of the device.
    /// - `node_id`: The ID of the node the property belongs to.
    /// - `prop_id`: The ID of the property.
    /// - `value`: The new value to set for the property.
    ///
    /// # Returns
    /// A `Publish` object containing the set command to be sent to the MQTT broker.
    pub fn set_command_ids(
        &self,
        homie_domain: &HomieDomain,
        device_id: &HomieID,
        node_id: &HomieID,
        prop_id: &HomieID,
        value: &HomieValue,
    ) -> Publish {
        Publish {
            topic: format!(
                "{}/{}/{}/{}/{}/{}",
                homie_domain, HOMIE_VERSION, device_id, node_id, prop_id, PROPERTY_SET_TOPIC
            ),
            qos: QoS::ExactlyOnce,
            retain: false,
            payload: value.into(),
        }
    }

    /// Publishes a set command for a property using a `PropertyRef`.
    ///
    /// # Parameters
    /// - `prop`: A reference to the `PropertyRef` identifying the property.
    /// - `value`: The new value to set for the property.
    ///
    /// # Returns
    /// A `Publish` object containing the set command to be sent to the MQTT broker.
    pub fn set_command(&self, prop: &PropertyRef, value: &HomieValue) -> Publish {
        self.set_command_ids(
            &prop.node.device.homie_domain,
            &prop.node.device.id,
            &prop.node.id,
            &prop.id,
            value,
        )
    }

    /// Sends a broadcast message to all devices in the specified Homie domain.
    ///
    /// # Parameters
    /// - `homie_domain`: The Homie domain in which to send the broadcast.
    /// - `broadcast_topic`: The topic of the broadcast.
    /// - `broadcast_message`: The content of the broadcast message.
    ///
    /// # Returns
    /// A `Publish` object representing the broadcast message to be sent to the MQTT broker.
    pub fn send_broadcast(
        &self,
        homie_domain: &HomieDomain,
        broadcast_topic: &str,
        broadcast_message: impl Into<String>,
    ) -> Publish {
        Publish {
            topic: format!(
                "{}/{}/{}/{}",
                homie_domain, HOMIE_VERSION, HOMIE_TOPIC_BROADCAST, broadcast_topic
            ),
            qos: QoS::ExactlyOnce,
            retain: false,
            payload: broadcast_message.into().into(),
        }
    }

    /// Subscribes to broadcast messages in the specified Homie domain.
    ///
    /// # Parameters
    /// - `homie_domain`: The Homie domain in which to subscribe to broadcasts.
    ///
    /// # Returns
    /// An iterator over a `Subscription` object that subscribes to broadcast messages.
    pub fn subscribe_broadcast<'a>(&'a self, homie_domain: &HomieDomain) -> impl Iterator<Item = Subscription> + 'a {
        iter::once(Subscription {
            topic: format!("{}/{}/$broadcast/#", homie_domain, HOMIE_VERSION),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Unsubscribes from broadcast messages in the specified Homie domain.
    ///
    /// # Parameters
    /// - `homie_domain`: The Homie domain in which to unsubscribe from broadcasts.
    ///
    /// # Returns
    /// An iterator over an `Unsubscribe` object that unsubscribes from broadcast messages.
    pub fn unsubscribe_broadcast<'a>(&'a self, homie_domain: &HomieDomain) -> impl Iterator<Item = Unsubscribe> + 'a {
        iter::once(Unsubscribe {
            topic: format!("{}/{}/$broadcast/#", homie_domain, HOMIE_VERSION),
        })
    }
}
