//! This module implements the necessary components for managing a Homie v5 device,
//! including publishing device states, handling subscriptions, and managing device
//! disconnection and reconfiguration.
//!
//! The primary struct in this module is [`Homie5DeviceProtocol`], which provides
//! functionality to generate MQTT publish and subscribe messages according to the Homie
//! v5 protocol. Additionally, the fixed step sequences for device publishing, reconfiguration,
//! and disconnection are provided as constant arrays.

use crate::{
    client::{LastWill, Publish, QoS, Subscription, Unsubscribe},
    device_description::HomieDeviceDescription,
    error::Homie5ProtocolError,
    homie_str_to_vecu8, DeviceLogLevel, DeviceRef, HomieDeviceStatus, HomieDomain, HomieID, TopicBuilder,
    DEVICE_ATTRIBUTE_ALERT, DEVICE_ATTRIBUTE_DESCRIPTION, DEVICE_ATTRIBUTE_LOG, DEVICE_ATTRIBUTE_STATE,
    PROPERTY_ATTRIBUTE_TARGET, PROPERTY_SET_TOPIC,
};

/// Represents the steps required to publish a Homie device.
///
/// This enum enumerates the sequential steps needed to bring a device online
/// following the Homie protocol, from setting the device's initial state to
/// publishing the device's retained properties and subscribing to relevant topics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePublishStep {
    /// Set the state of the device to "init" and publish the state
    DeviceStateInit,
    /// Publish the device description
    DeviceDescription,
    /// Publish the property values for all the retained properties
    PropertyValues,
    /// Subscribe to all settable property /set topics
    SubscribeProperties,
    /// Set the state of the device to "ready" and publish the state
    DeviceStateReady,
}

/// All steps for publishing a device, in order.
pub const DEVICE_PUBLISH_STEPS: [DevicePublishStep; 5] = [
    DevicePublishStep::DeviceStateInit,
    DevicePublishStep::DeviceDescription,
    DevicePublishStep::PropertyValues,
    DevicePublishStep::SubscribeProperties,
    DevicePublishStep::DeviceStateReady,
];

/// Provides an iterator that yields all the necessary steps for publishing a device in order.
///
/// See [`DEVICE_PUBLISH_STEPS`].
pub fn homie_device_publish_steps() -> impl Iterator<Item = DevicePublishStep> {
    DEVICE_PUBLISH_STEPS.into_iter()
}

/// Represents the steps required to reconfigure a Homie device.
///
/// This enum defines the steps needed to update a device's configuration, such as
/// modifying nodes or properties, followed by republishing the updated state and properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceReconfigureStep {
    /// Set the state of the device to "init" and publish the state
    DeviceStateInit,
    /// Unsubscribe from all device properties
    UnsubscribeProperties,
    /// Perform the device reconfiguration (change of nodes/properties/name...)
    Reconfigure,
    /// Publish the device description
    DeviceDescription,
    /// Publish the property values for all the retained properties
    PropertyValues,
    /// Subscribe to all settable property /set topics
    SubscribeProperties,
    /// Set the state of the device to "ready" and publish the state
    DeviceStateReady,
}

/// All steps for reconfiguring a device, in order.
pub const DEVICE_RECONFIGURE_STEPS: [DeviceReconfigureStep; 7] = [
    DeviceReconfigureStep::DeviceStateInit,
    DeviceReconfigureStep::UnsubscribeProperties,
    DeviceReconfigureStep::Reconfigure,
    DeviceReconfigureStep::DeviceDescription,
    DeviceReconfigureStep::PropertyValues,
    DeviceReconfigureStep::SubscribeProperties,
    DeviceReconfigureStep::DeviceStateReady,
];

/// Provides an iterator that yields all the necessary steps for reconfiguring a device in order.
///
/// See [`DEVICE_RECONFIGURE_STEPS`].
pub fn homie_device_reconfigure_steps() -> impl Iterator<Item = DeviceReconfigureStep> {
    DEVICE_RECONFIGURE_STEPS.into_iter()
}

/// Represents the steps required to disconnect a Homie device.
///
/// This enum enumerates the steps needed to gracefully disconnect a device
/// following the Homie protocol, ensuring that all device properties are properly unsubscribed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceDisconnectStep {
    /// Set the state of the device to "disconnect" and publish the state
    DeviceStateDisconnect,
    /// Unsubscribe from all device properties
    UnsubscribeProperties,
}

/// All steps for disconnecting a device, in order.
pub const DEVICE_DISCONNECT_STEPS: [DeviceDisconnectStep; 2] = [
    DeviceDisconnectStep::DeviceStateDisconnect,
    DeviceDisconnectStep::UnsubscribeProperties,
];

/// Provides an iterator that yields all the necessary steps for disconnecting a device in order.
///
/// See [`DEVICE_DISCONNECT_STEPS`].
pub fn homie_device_disconnect_steps() -> impl Iterator<Item = DeviceDisconnectStep> {
    DEVICE_DISCONNECT_STEPS.into_iter()
}

/// QoS for property values, targets and set commands: the convention requires QoS 0 for
/// non-retained properties so that events are not delivered late or multiple times.
pub(crate) fn property_qos(retained: bool) -> QoS {
    if retained {
        QoS::ExactlyOnce
    } else {
        QoS::AtMostOnce
    }
}

/// Represents the Homie v5 protocol implementation for a single device, providing methods for
/// publishing state, logging, and handling properties.
///
/// [`Homie5DeviceProtocol`] defines the MQTT topics and messages needed for a device to
/// communicate its state, properties, and logs, as well as managing subscriptions to
/// settable properties. Each protocol instance publishes for exactly one device; use
/// [`Homie5DeviceProtocol::clone_for_child`] to obtain the protocol for a child device.
#[derive(Clone, Debug)]
pub struct Homie5DeviceProtocol {
    device_ref: DeviceRef,
    /// ID of the root device if this protocol belongs to a child device.
    root: Option<HomieID>,
}

impl Homie5DeviceProtocol {
    /// Creates a new [`Homie5DeviceProtocol`] and generates the corresponding last will message.
    ///
    /// # Parameters
    /// - `device_id`: The ID of the Homie device.
    /// - `homie_domain`: The domain under which the device operates.
    ///
    /// # Returns
    /// A tuple of the created [`Homie5DeviceProtocol`] and its [`LastWill`] message.
    pub fn new(device_id: HomieID, homie_domain: HomieDomain) -> (Self, LastWill) {
        let last_will = LastWill {
            topic: TopicBuilder::new_for_device(&homie_domain, &device_id)
                .add_attr(DEVICE_ATTRIBUTE_STATE)
                .build(),
            message: HomieDeviceStatus::Lost.as_str().bytes().collect(),
            qos: QoS::ExactlyOnce,
            retain: true,
        };

        let homie5_proto = Self {
            device_ref: DeviceRef {
                homie_domain,
                id: device_id,
            },
            root: None,
        };

        (homie5_proto, last_will)
    }

    /// Returns the device ref the protocol is instantiated for.
    pub fn device_ref(&self) -> &DeviceRef {
        &self.device_ref
    }

    /// Returns the device's ID.
    pub fn id(&self) -> &HomieID {
        &self.device_ref.id
    }

    /// Returns the domain in which the device is operating.
    pub fn homie_domain(&self) -> &HomieDomain {
        &self.device_ref.homie_domain
    }

    /// Checks if the device is a child device.
    pub fn is_child(&self) -> bool {
        self.root.is_some()
    }

    /// Returns the ID of the root device this protocol publishes under.
    pub fn root_id(&self) -> &HomieID {
        self.root.as_ref().unwrap_or(&self.device_ref.id)
    }

    /// Clones the protocol for a child device using the given `device_id`.
    ///
    /// Descriptions published through the child protocol must set `root` to the root device's ID.
    pub fn clone_for_child(&self, device_id: HomieID) -> Self {
        Self {
            device_ref: DeviceRef {
                homie_domain: self.device_ref.homie_domain.clone(),
                id: device_id,
            },
            root: Some(self.root_id().clone()),
        }
    }

    /// Checks that `description` carries the correct `root` attribute for this device.
    fn check_root(&self, description: &HomieDeviceDescription) -> Result<(), Homie5ProtocolError> {
        match (&self.root, &description.root) {
            (None, None) => Ok(()),
            (None, Some(_)) => Err(Homie5ProtocolError::NonEmptyRootForRootDevice),
            (Some(root), Some(desc_root)) if root == desc_root => Ok(()),
            _ => Err(Homie5ProtocolError::RootMismatch),
        }
    }

    fn device_topic(&self) -> TopicBuilder {
        TopicBuilder::new_for_device(self.homie_domain(), self.id())
    }

    fn property_topic(&self, node_id: &HomieID, prop_id: &HomieID) -> TopicBuilder {
        TopicBuilder::new_for_property(self.homie_domain(), self.id(), node_id, prop_id)
    }

    /// Publishes the device's state.
    pub fn publish_state(&self, state: HomieDeviceStatus) -> Publish {
        Publish {
            topic: self.device_topic().add_attr(DEVICE_ATTRIBUTE_STATE).build(),
            retain: true,
            payload: state.as_str().into(),
            qos: QoS::ExactlyOnce,
        }
    }

    /// Publishes a log message for the device.
    pub fn publish_log(&self, level: DeviceLogLevel, log_msg: &str) -> Publish {
        Publish {
            topic: self
                .device_topic()
                .add_attr(DEVICE_ATTRIBUTE_LOG)
                .add_attr(level.as_str())
                .build(),
            qos: QoS::AtLeastOnce,
            retain: false,
            payload: log_msg.into(),
        }
    }

    /// Publishes an alert with a given `alert_id` and `alert_msg`.
    ///
    /// An empty `alert_msg` is published as an empty string (`0x00`), not as an alert removal.
    /// Use [`Self::publish_clear_alert`] to clear an alert.
    pub fn publish_alert(&self, alert_id: &HomieID, alert_msg: &str) -> Publish {
        Publish {
            topic: self
                .device_topic()
                .add_attr(DEVICE_ATTRIBUTE_ALERT)
                .add_attr(alert_id.as_str())
                .build(),
            qos: QoS::ExactlyOnce,
            retain: true,
            payload: homie_str_to_vecu8(alert_msg),
        }
    }

    /// Clears the alert with the given `alert_id`.
    pub fn publish_clear_alert(&self, alert_id: &HomieID) -> Publish {
        Publish {
            topic: self
                .device_topic()
                .add_attr(DEVICE_ATTRIBUTE_ALERT)
                .add_attr(alert_id.as_str())
                .build(),
            qos: QoS::ExactlyOnce,
            retain: true,
            payload: Vec::default(),
        }
    }

    /// Publishes a Homie value for a given property and node.
    ///
    /// Non-retained values are published with QoS 0 as required by the convention.
    pub fn publish_value(
        &self,
        node_id: &HomieID,
        prop_id: &HomieID,
        value: impl Into<String>,
        retain: bool,
    ) -> Publish {
        Publish {
            topic: self.property_topic(node_id, prop_id).build(),
            qos: property_qos(retain),
            retain,
            payload: homie_str_to_vecu8(value),
        }
    }

    /// Publishes the target value for a given property and node.
    pub fn publish_target(
        &self,
        node_id: &HomieID,
        prop_id: &HomieID,
        value: impl Into<String>,
        retain: bool,
    ) -> Publish {
        Publish {
            topic: self
                .property_topic(node_id, prop_id)
                .add_attr(PROPERTY_ATTRIBUTE_TARGET)
                .build(),
            qos: property_qos(retain),
            retain,
            payload: homie_str_to_vecu8(value),
        }
    }

    /// Publishes the device description.
    ///
    /// # Errors
    /// Returns an error if the description's `root` attribute does not match the device type.
    pub fn publish_description(&self, description: &HomieDeviceDescription) -> Result<Publish, Homie5ProtocolError> {
        self.check_root(description)?;
        let json = serde_json::to_string(description).map_err(|_| Homie5ProtocolError::InvalidDeviceDescription)?;
        Ok(Publish {
            topic: self.device_topic().add_attr(DEVICE_ATTRIBUTE_DESCRIPTION).build(),
            qos: QoS::ExactlyOnce,
            retain: true,
            payload: json.into(),
        })
    }

    /// Subscribes to all settable properties for the device.
    ///
    /// # Errors
    /// Returns an error if the description's `root` attribute does not match the device type.
    pub fn subscribe_props<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Subscription> + 'a, Homie5ProtocolError> {
        self.check_root(description)?;
        Ok(description
            .iter()
            .filter(|&(_, _, _, prop)| prop.settable)
            .map(|(node_id, _, prop_id, _)| Subscription {
                topic: self
                    .property_topic(node_id, prop_id)
                    .add_attr(PROPERTY_SET_TOPIC)
                    .build(),
                qos: QoS::ExactlyOnce,
            }))
    }

    /// Unsubscribes from all settable properties for the device.
    ///
    /// # Errors
    /// Returns an error if the description's `root` attribute does not match the device type.
    pub fn unsubscribe_props<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Unsubscribe> + 'a, Homie5ProtocolError> {
        self.check_root(description)?;
        Ok(description
            .iter()
            .filter(|&(_, _, _, prop)| prop.settable)
            .map(|(node_id, _, prop_id, _)| Unsubscribe {
                topic: self
                    .property_topic(node_id, prop_id)
                    .add_attr(PROPERTY_SET_TOPIC)
                    .build(),
            }))
    }

    /// Removes the device by clearing all retained property values.
    ///
    /// Clears `$state` first, then `$description` and all retained property values and targets.
    /// Active alerts live under `$alert/<alert-id>` and are not known to the protocol; clear them
    /// with [`Self::publish_clear_alert`] before removing the device.
    ///
    /// # Errors
    /// Returns an error if the description's `root` attribute does not match the device type.
    pub fn remove_device<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Publish> + 'a, Homie5ProtocolError> {
        self.check_root(description)?;

        let clear = |topic: String| Publish {
            topic,
            qos: QoS::ExactlyOnce,
            retain: true,
            payload: Vec::default(),
        };

        // clear retained device attributes (starts with `$state` as per convention)
        let attrs = [DEVICE_ATTRIBUTE_STATE, DEVICE_ATTRIBUTE_DESCRIPTION]
            .into_iter()
            .map(move |attribute| clear(self.device_topic().add_attr(attribute).build()));

        // clear all retained property values and targets
        let props =
            description
                .iter()
                .filter(|(_, _, _, prop)| prop.retained)
                .flat_map(move |(node_id, _, prop_id, _)| {
                    [
                        clear(self.property_topic(node_id, prop_id).build()),
                        clear(
                            self.property_topic(node_id, prop_id)
                                .add_attr(PROPERTY_ATTRIBUTE_TARGET)
                                .build(),
                        ),
                    ]
                });
        Ok(attrs.chain(props))
    }
}
