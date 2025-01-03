//! This module implements the necessary components for managing a Homie v5 device,
//! including publishing device states, handling subscriptions, and managing device
//! disconnection and reconfiguration.
//!
//! The primary struct in this module is [`Homie5DeviceProtocol`], which provides
//! functionality to generate MQTT publish and subscribe messages according to the Homie
//! v5 protocol. Additionally, state machines for device publishing, reconfiguration,
//! and disconnection are provided through the use of enumerated steps and transitions.

use crate::{
    client::{LastWill, Publish, QoS, Subscription, Unsubscribe},
    device_description::{HomieDeviceDescription, HomiePropertyIterator},
    error::Homie5ProtocolError,
    homie_str_to_vecu8,
    statemachine::{HomieStateMachine, Transition},
    DeviceLogLevel, DeviceRef, HomieDeviceStatus, HomieDomain, HomieID, TopicBuilder, DEVICE_ATTRIBUTES,
    DEVICE_ATTRIBUTE_ALERT, DEVICE_ATTRIBUTE_DESCRIPTION, DEVICE_ATTRIBUTE_LOG, DEVICE_ATTRIBUTE_STATE,
    PROPERTY_ATTRIBUTE_TARGET, PROPERTY_SET_TOPIC,
};

#[derive(Default, Copy, Clone)]
/// Represents the steps required to publish a Homie device.
///
/// This enum enumerates the sequential steps needed to bring a device online
/// following the Homie protocol, from setting the device's initial state to
/// publishing the device's retained properties and subscribing to relevant topics.

pub enum DevicePublishStep {
    #[default]
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

impl Transition<DevicePublishStep> for DevicePublishStep {
    fn transition(&self) -> Option<DevicePublishStep> {
        match self {
            DevicePublishStep::DeviceStateInit => Some(DevicePublishStep::DeviceDescription),
            DevicePublishStep::DeviceDescription => Some(DevicePublishStep::PropertyValues),
            DevicePublishStep::PropertyValues => Some(DevicePublishStep::SubscribeProperties),
            DevicePublishStep::SubscribeProperties => Some(DevicePublishStep::DeviceStateReady),
            DevicePublishStep::DeviceStateReady => None,
        }
    }
}

/// Provides an iterator that yields all the necessary steps for publishing a device in order.
///
/// This iterator follows the sequence defined in [`DevicePublishStep`], transitioning
/// through the device's initialization, description publishing, property publishing,
/// and subscription setup.
pub fn homie_device_publish_steps() -> impl Iterator<Item = DevicePublishStep> {
    HomieStateMachine::new(Default::default())
}

/// Represents the steps required to reconfigure a Homie device.
///
/// This enum defines the steps needed to update a device's configuration, such as
/// modifying nodes or properties, followed by republishing the updated state and properties.
#[derive(Default, Copy, Clone)]
pub enum DeviceReconfigureStep {
    #[default]
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

impl Transition<DeviceReconfigureStep> for DeviceReconfigureStep {
    fn transition(&self) -> Option<DeviceReconfigureStep> {
        match self {
            DeviceReconfigureStep::DeviceStateInit => Some(DeviceReconfigureStep::UnsubscribeProperties),
            DeviceReconfigureStep::UnsubscribeProperties => Some(DeviceReconfigureStep::Reconfigure),
            DeviceReconfigureStep::Reconfigure => Some(DeviceReconfigureStep::DeviceDescription),
            DeviceReconfigureStep::DeviceDescription => Some(DeviceReconfigureStep::PropertyValues),
            DeviceReconfigureStep::PropertyValues => Some(DeviceReconfigureStep::SubscribeProperties),
            DeviceReconfigureStep::SubscribeProperties => Some(DeviceReconfigureStep::DeviceStateReady),
            DeviceReconfigureStep::DeviceStateReady => None,
        }
    }
}

/// Provides an iterator that yields all the necessary steps for reconfiguring a device in order.
///
/// This iterator follows the sequence defined in [`DeviceReconfigureStep`], ensuring the device
/// is properly reconfigured, including unsubscribing from old properties, reconfiguring, and
/// republishing the updated state.
pub fn homie_device_reconfigure_steps() -> impl Iterator<Item = DeviceReconfigureStep> {
    HomieStateMachine::new(Default::default())
}

/// Represents the steps required to disconnect a Homie device.
///
/// This enum enumerates the steps needed to gracefully disconnect a device
/// following the Homie protocol, ensuring that all device properties are properly unsubscribed.
#[derive(Default, Copy, Clone)]
pub enum DeviceDisconnectStep {
    #[default]
    /// Set the state of the device to "disconnect" and publish the state
    DeviceStateDisconnect,
    /// Unsubscribe from all device properties
    UnsubscribeProperties,
}

impl Transition<DeviceDisconnectStep> for DeviceDisconnectStep {
    fn transition(&self) -> Option<DeviceDisconnectStep> {
        match self {
            DeviceDisconnectStep::DeviceStateDisconnect => Some(DeviceDisconnectStep::UnsubscribeProperties),
            DeviceDisconnectStep::UnsubscribeProperties => None,
        }
    }
}

/// Provides an iterator that yields all the necessary steps for disconnecting a device in order.
///
/// This iterator follows the sequence defined in [`DeviceDisconnectStep`], ensuring the device
/// is properly disconnected and its properties unsubscribed.
pub fn homie_device_disconnect_steps() -> impl Iterator<Item = DeviceDisconnectStep> {
    HomieStateMachine::new(Default::default())
}

/// Represents the Homie v5 protocol implementation for a device, providing methods for
/// publishing state, logging, and handling properties.
///
/// [`Homie5DeviceProtocol`] defines the MQTT topics and messages needed for a device to
/// communicate its state, properties, and logs, as well as managing subscriptions to
/// settable properties.
#[derive(Clone, Debug)]
pub struct Homie5DeviceProtocol {
    device_ref: DeviceRef,
    is_child: bool,
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
            qos: crate::client::QoS::AtLeastOnce,
            retain: true,
        };

        let homie5_proto = Self {
            device_ref: DeviceRef {
                homie_domain,
                id: device_id,
            },
            is_child: false,
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
        self.is_child
    }

    /// Clones the protocol for a child device using the given `device_id`.
    pub fn clone_for_child(&self, device_id: HomieID) -> Self {
        Self {
            device_ref: DeviceRef {
                homie_domain: self.device_ref.homie_domain.clone(),
                id: device_id,
            },
            is_child: true,
        }
    }

    /// Clones the protocol for a child device using the provided root protocol and device ID.
    pub fn for_child(device_id: HomieID, root: Homie5DeviceProtocol) -> Self {
        Self {
            device_ref: DeviceRef {
                homie_domain: root.homie_domain().clone(),
                id: device_id,
            },
            is_child: true,
        }
    }

    /// Publishes the device's state.
    pub fn publish_state(&self, state: HomieDeviceStatus) -> Publish {
        self.publish_state_for_id(self.id(), state)
    }

    /// Publishes the state for the given `device_id`.
    pub fn publish_state_for_id(&self, device_id: &HomieID, state: HomieDeviceStatus) -> Publish {
        Publish {
            topic: TopicBuilder::new_for_device(self.homie_domain(), device_id)
                .add_attr(DEVICE_ATTRIBUTE_STATE)
                .build(),
            retain: true,
            payload: state.as_str().into(),
            qos: QoS::ExactlyOnce,
        }
    }

    /// Publishes a log message for the device.
    pub fn publish_log(&self, level: DeviceLogLevel, log_msg: &str) -> Publish {
        self.publish_log_for_id(self.id(), level, log_msg)
    }

    /// Publishes a log message for the given `device_id`.
    pub fn publish_log_for_id(&self, device_id: &HomieID, level: DeviceLogLevel, log_msg: &str) -> Publish {
        Publish {
            topic: TopicBuilder::new_for_device(self.homie_domain(), device_id)
                .add_attr(DEVICE_ATTRIBUTE_LOG)
                .add_attr(level.as_str())
                .build(),
            qos: QoS::AtLeastOnce,
            retain: true,
            payload: log_msg.into(),
        }
    }

    // Publishes an alert with a given `alert_id` and `alert_msg`.
    pub fn publish_alert(&self, alert_id: &str, alert_msg: &str) -> Publish {
        self.publish_alert_for_id(self.id(), alert_id, alert_msg)
    }

    /// Publishes an alert with a given `alert_id` and `alert_msg` for the provided `device_id`.
    pub fn publish_alert_for_id(&self, device_id: &HomieID, alert_id: &str, alert_msg: &str) -> Publish {
        Publish {
            topic: TopicBuilder::new_for_device(self.homie_domain(), device_id)
                .add_attr(DEVICE_ATTRIBUTE_ALERT)
                .add_attr(alert_id)
                .build(),
            qos: QoS::AtLeastOnce,
            retain: true,
            payload: alert_msg.into(),
        }
    }

    /// Publishes a Homie value for a given property and node.
    pub fn publish_value(
        &self,
        node_id: &HomieID,
        prop_id: &HomieID,
        value: impl Into<String>,
        retain: bool,
    ) -> Publish {
        self.publish_value_for_id(self.id(), node_id, prop_id, value, retain)
    }

    /// Publishes a value for a specific `device_id`.
    pub fn publish_value_for_id(
        &self,
        device_id: &HomieID,
        node_id: &HomieID,
        prop_id: &HomieID,
        value: impl Into<String>,
        retain: bool,
    ) -> Publish {
        Publish {
            topic: TopicBuilder::new_for_property(self.homie_domain(), device_id, node_id, prop_id).build(),
            qos: QoS::ExactlyOnce,
            retain,
            payload: homie_str_to_vecu8(value.into()),
        }
    }

    /// Publishes the target value for a given property and node.
    pub fn publish_target(
        &self,
        node_id: &HomieID,
        prop_id: &HomieID,
        value: impl Into<String>,
        retained: bool,
    ) -> Publish {
        self.publish_target_for_id(self.id(), node_id, prop_id, value, retained)
    }

    /// Publishes the target value for a given property using the provided `device_id`.
    pub fn publish_target_for_id(
        &self,
        device_id: &HomieID,
        node_id: &HomieID,
        prop_id: &HomieID,
        value: impl Into<String>,
        retain: bool,
    ) -> Publish {
        Publish {
            topic: TopicBuilder::new_for_property(self.homie_domain(), device_id, node_id, prop_id)
                .add_attr(PROPERTY_ATTRIBUTE_TARGET)
                .build(),
            qos: QoS::ExactlyOnce,
            retain,
            payload: homie_str_to_vecu8(value),
        }
    }

    /// Publishes the device description.
    ///
    /// # Errors
    /// Returns an error if the description is invalid for the device type.
    pub fn publish_description(&self, description: &HomieDeviceDescription) -> Result<Publish, Homie5ProtocolError> {
        self.publish_description_for_id(self.id(), description)
    }

    /// Publishes the device description for the provided `device_id`.
    ///
    /// # Errors
    /// Returns an error if the description is invalid for the device type.
    pub fn publish_description_for_id(
        &self,
        device_id: &HomieID,
        description: &HomieDeviceDescription,
    ) -> Result<Publish, Homie5ProtocolError> {
        if !self.is_child && self.id() == device_id && description.root.is_some() {
            return Err(Homie5ProtocolError::NonEmptyRootForRootDevice);
        } else if !self.is_child && self.id() != device_id && Some(self.id()) != description.root.as_ref() {
            return Err(Homie5ProtocolError::RootMismatch);
        }
        match serde_json::to_string(description) {
            Ok(json) => Ok(Publish {
                topic: TopicBuilder::new_for_device(self.homie_domain(), device_id)
                    .add_attr(DEVICE_ATTRIBUTE_DESCRIPTION)
                    .build(),
                qos: QoS::ExactlyOnce,
                retain: true,
                payload: json.into(),
            }),
            Err(_) => {
                // TODO: log actual error for debug purposes
                Err(Homie5ProtocolError::InvalidDeviceDescription)
            }
        }
    }

    /// Subscribes to all settable properties for the device.
    ///
    /// # Errors
    /// Returns an error if the description is invalid for the device type.
    pub fn subscribe_props<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Subscription> + 'a, Homie5ProtocolError> {
        self.subscribe_props_for_id(self.id(), description)
    }

    /// Subscribes to all settable properties for the given `device_id`.
    ///
    /// # Errors
    /// Returns an error if the description is invalid for the device type.
    pub fn subscribe_props_for_id<'a>(
        &'a self,
        device_id: &'a HomieID,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Subscription> + 'a, Homie5ProtocolError> {
        if !self.is_child && self.id() == device_id && description.root.is_some() {
            return Err(Homie5ProtocolError::NonEmptyRootForRootDevice);
        } else if !self.is_child && self.id() != device_id && Some(self.id()) != description.root.as_ref() {
            return Err(Homie5ProtocolError::RootMismatch);
        }

        Ok(description.iter().map(move |(node_id, _, prop_id, _)| Subscription {
            topic: TopicBuilder::new_for_property(self.homie_domain(), device_id, node_id, prop_id)
                .add_attr(PROPERTY_SET_TOPIC)
                .build(),
            qos: QoS::ExactlyOnce,
        }))
    }

    /// Unsubscribes from all settable properties for the device.
    ///
    /// # Errors
    /// Returns an error if the description is invalid for the device type.
    pub fn unsubscribe_props<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Unsubscribe> + 'a, Homie5ProtocolError> {
        self.unsubscribe_props_for_id(self.id(), description)
    }

    /// Unsubscribes from all settable properties for the given `device_id`.
    ///
    /// # Errors
    /// Returns an error if the description is invalid for the device type.
    pub fn unsubscribe_props_for_id<'a>(
        &'a self,
        device_id: &'a HomieID,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Unsubscribe> + 'a, Homie5ProtocolError> {
        if !self.is_child && self.id() == device_id && description.root.is_some() {
            return Err(Homie5ProtocolError::NonEmptyRootForRootDevice);
        } else if !self.is_child && self.id() != device_id && Some(self.id()) != description.root.as_ref() {
            return Err(Homie5ProtocolError::RootMismatch);
        }
        let prop_iter = HomiePropertyIterator::new(description);
        Ok(prop_iter.map(move |(node_id, _, prop_id, _)| Unsubscribe {
            topic: TopicBuilder::new_for_property(self.homie_domain(), device_id, node_id, prop_id).build(),
        }))
    }

    /// Removes the device by clearing all retained property values.
    ///
    /// # Errors
    /// Returns an error if the description is invalid for the device type.
    pub fn remove_device<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Publish> + 'a, Homie5ProtocolError> {
        self.remove_device_for_id(self.id(), description)
    }

    /// Removes the device for the given `device_id` by clearing all retained property values.
    ///
    /// # Errors
    /// Returns an error if the description is invalid for the device type.
    pub fn remove_device_for_id<'a>(
        &'a self,
        device_id: &'a HomieID,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Publish> + 'a, Homie5ProtocolError> {
        if !self.is_child && self.id() == device_id && description.root.is_some() {
            return Err(Homie5ProtocolError::NonEmptyRootForRootDevice);
        } else if !self.is_child && self.id() != device_id && Some(self.id()) != description.root.as_ref() {
            return Err(Homie5ProtocolError::RootMismatch);
        }

        // clear device attributes (startes with `$state` as per convention)
        let attrs = DEVICE_ATTRIBUTES.iter().map(move |attribute| Publish {
            topic: TopicBuilder::new_for_device(self.homie_domain(), device_id)
                .add_attr(attribute)
                .build(),
            qos: QoS::ExactlyOnce,
            retain: true,
            payload: Vec::default(),
        });

        let prop_iter = HomiePropertyIterator::new(description);
        // clear all retained property values
        let props = prop_iter
            .filter(|(_, _, _, prop)| prop.retained)
            .flat_map(move |(node_id, _, prop_id, _)| {
                [
                    Publish {
                        topic: TopicBuilder::new_for_property(self.homie_domain(), device_id, node_id, prop_id)
                            .add_attr(PROPERTY_SET_TOPIC)
                            .build(),
                        qos: QoS::ExactlyOnce,
                        retain: true,
                        payload: Vec::default(),
                    },
                    Publish {
                        topic: TopicBuilder::new_for_property(self.homie_domain(), device_id, node_id, prop_id)
                            .add_attr(PROPERTY_ATTRIBUTE_TARGET)
                            .build(),
                        qos: QoS::ExactlyOnce,
                        retain: true,
                        payload: Vec::default(),
                    },
                ]
            });
        Ok(attrs.chain(props))
    }
}
