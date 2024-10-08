use crate::{
    client::{LastWill, Publish, QoS, Subscription, Unsubscribe},
    device_description::{HomieDeviceDescription, HomiePropertyIterator},
    error::Homie5ProtocolError,
    statemachine::{HomieStateMachine, Transition},
    HomieDeviceStatus, PropertyIdentifier, DEFAULT_ROOT_TOPIC, DEVICE_ATTRIBUTES, DEVICE_ATTRIBUTE_ALERT,
    DEVICE_ATTRIBUTE_DESCRIPTION, DEVICE_ATTRIBUTE_LOG, DEVICE_ATTRIBUTE_STATE, HOMIE_VERSION,
    PROPERTY_ATTRIBUTE_TARGET, PROPERTY_SET_TOPIC,
};

#[derive(Default, Copy, Clone)]
/// Contains all the steps required to publish a device
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

/// provides an iterator that will yield all steps required for publishing a device in the correct
/// order
pub fn homie_device_publish_steps() -> impl Iterator<Item = DevicePublishStep> {
    HomieStateMachine::new(Default::default())
}

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

/// provides an iterator that will yield all steps required to reconfigure a device in the correct
/// order
pub fn homie_device_reconfigure_steps() -> impl Iterator<Item = DeviceReconfigureStep> {
    HomieStateMachine::new(Default::default())
}
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

/// provides an iterator that will yield all steps required to disconnect a device in the correct
/// order
pub fn homie_device_disconnect_steps() -> impl Iterator<Item = DeviceDisconnectStep> {
    HomieStateMachine::new(Default::default())
}

#[derive(Clone, Debug)]
pub struct Homie5DeviceProtocol {
    id: String,
    topic_root: String,
    is_child: bool,
}

impl Homie5DeviceProtocol {
    pub fn new(device_id: impl Into<String>, topic_root: Option<impl Into<String>>) -> (Self, LastWill) {
        let device_id = device_id.into();
        let topic_root = if let Some(tr) = topic_root {
            tr.into()
        } else {
            DEFAULT_ROOT_TOPIC.to_owned()
        };
        let last_will = LastWill {
            topic: format!("{}/5/{}/$state", &topic_root, &device_id),
            message: HomieDeviceStatus::Lost.as_str().bytes().collect(),
            qos: crate::client::QoS::AtLeastOnce,
            retain: true,
        };

        let homie5_proto = Self {
            id: device_id,
            topic_root,
            is_child: false,
        };

        (homie5_proto, last_will)
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn topic_root(&self) -> &str {
        &self.topic_root
    }

    pub fn is_child(&self) -> bool {
        self.is_child
    }

    pub fn clone_for_child(&self, device_id: impl Into<String>) -> Self {
        Self {
            id: device_id.into(),
            topic_root: self.topic_root.clone(),
            is_child: true,
        }
    }

    pub fn for_child(device_id: impl Into<String>, root: Homie5DeviceProtocol) -> Self {
        Self {
            id: device_id.into(),
            topic_root: root.topic_root.clone(),
            is_child: true,
        }
    }

    pub fn publish_state(&self, state: HomieDeviceStatus) -> Publish {
        self.publish_state_for_id(&self.id, state)
    }

    pub fn publish_state_for_id(&self, device_id: &str, state: HomieDeviceStatus) -> Publish {
        Publish {
            topic: format!(
                "{}/{}/{}/{}",
                self.topic_root, HOMIE_VERSION, device_id, DEVICE_ATTRIBUTE_STATE
            ),
            retain: true,
            payload: state.as_str().into(),
            qos: QoS::ExactlyOnce,
        }
    }

    pub fn publish_log(&self, log_msg: &str) -> Publish {
        self.publish_log_for_id(&self.id, log_msg)
    }

    pub fn publish_log_for_id(&self, device_id: &str, log_msg: &str) -> Publish {
        Publish {
            topic: format!(
                "{}/{}/{}/{}",
                self.topic_root, HOMIE_VERSION, device_id, DEVICE_ATTRIBUTE_LOG
            ),
            qos: QoS::AtLeastOnce,
            retain: true,
            payload: log_msg.into(),
        }
    }

    pub fn publish_alert(&self, alert_id: &str, alert_msg: &str) -> Publish {
        self.publish_alert_for_id(&self.id, alert_id, alert_msg)
    }

    pub fn publish_alert_for_id(&self, device_id: &str, alert_id: &str, alert_msg: &str) -> Publish {
        Publish {
            topic: format!(
                "{}/{}/{}/{}/{}",
                self.topic_root, HOMIE_VERSION, device_id, DEVICE_ATTRIBUTE_ALERT, alert_id
            ),
            qos: QoS::AtLeastOnce,
            retain: true,
            payload: alert_msg.into(),
        }
    }

    pub fn publish_homie_value(&self, node_id: &str, prop_id: &str, value: impl Into<String>, retain: bool) -> Publish {
        self.publish_homie_value_for_id(&self.id, node_id, prop_id, value, retain)
    }

    pub fn publish_homie_value_for_id(
        &self,
        device_id: &str,
        node_id: &str,
        prop_id: &str,
        value: impl Into<String>,
        retain: bool,
    ) -> Publish {
        self.publish_value_for_id(device_id, node_id, prop_id, value, retain)
    }

    pub fn publish_value(&self, node_id: &str, prop_id: &str, value: impl Into<String>, retain: bool) -> Publish {
        self.publish_value_for_id(&self.id, node_id, prop_id, value, retain)
    }

    pub fn publish_value_prop(&self, prop: &PropertyIdentifier, value: impl Into<String>, retain: bool) -> Publish {
        self.publish_value_for_id(&self.id, &prop.node.id, &prop.id, value, retain)
    }
    pub fn publish_value_for_id(
        &self,
        device_id: &str,
        node_id: &str,
        prop_id: &str,
        value: impl Into<String>,
        retain: bool,
    ) -> Publish {
        Publish {
            topic: format!(
                "{}/{}/{}/{}/{}",
                self.topic_root, HOMIE_VERSION, device_id, node_id, prop_id
            ),
            qos: QoS::ExactlyOnce,
            retain,
            payload: value.into().into_bytes(),
        }
    }

    pub fn publish_target(&self, node_id: &str, prop_id: &str, value: impl Into<String>, retained: bool) -> Publish {
        self.publish_target_for_id(&self.id, node_id, prop_id, value, retained)
    }

    pub fn publish_target_prop(&self, prop: &PropertyIdentifier, value: impl Into<String>, retained: bool) -> Publish {
        self.publish_target_for_id(&self.id, &prop.node.id, &prop.id, value, retained)
    }
    pub fn publish_target_for_id(
        &self,
        device_id: &str,
        node_id: &str,
        prop_id: &str,
        value: impl Into<String>,
        retain: bool,
    ) -> Publish {
        Publish {
            topic: format!(
                "{}/{}/{}/{}/{}/{}",
                self.topic_root, HOMIE_VERSION, device_id, node_id, prop_id, PROPERTY_ATTRIBUTE_TARGET
            ),
            qos: QoS::ExactlyOnce,
            retain,
            payload: value.into().into_bytes(),
        }
    }

    pub fn publish_description(&self, description: &HomieDeviceDescription) -> Result<Publish, Homie5ProtocolError> {
        self.publish_description_for_id(&self.id, description)
    }

    pub fn publish_description_for_id(
        &self,
        device_id: &str,
        description: &HomieDeviceDescription,
    ) -> Result<Publish, Homie5ProtocolError> {
        if !self.is_child && self.id == device_id && description.root.is_some() {
            return Err(Homie5ProtocolError::NonEmptyRootForRootDevice);
        } else if !self.is_child && self.id != device_id && Some(&self.id) != description.root.as_ref() {
            return Err(Homie5ProtocolError::RootMismatch);
        }
        match serde_json::to_string(description) {
            Ok(json) => Ok(Publish {
                topic: format!(
                    "{}/{}/{}/{}",
                    self.topic_root, HOMIE_VERSION, device_id, DEVICE_ATTRIBUTE_DESCRIPTION
                ),
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

    pub fn base_topic_for_id(&self, device_id: &str) -> String {
        format!("{}/{}/{}", self.topic_root, HOMIE_VERSION, device_id)
    }

    pub fn subscribe_props<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Subscription> + 'a, Homie5ProtocolError> {
        self.subscribe_props_for_id(&self.id, description)
    }

    pub fn subscribe_props_for_id<'a>(
        &'a self,
        device_id: &'a str,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Subscription> + 'a, Homie5ProtocolError> {
        if !self.is_child && self.id == device_id && description.root.is_some() {
            return Err(Homie5ProtocolError::NonEmptyRootForRootDevice);
        } else if !self.is_child && self.id != device_id && Some(&self.id) != description.root.as_ref() {
            return Err(Homie5ProtocolError::RootMismatch);
        }
        let prop_iter = HomiePropertyIterator::new(description);
        let base_topic = format!("{}/{}/{}", self.topic_root, HOMIE_VERSION, device_id);

        Ok(prop_iter.map(move |(node_id, _, prop_id, _)| Subscription {
            topic: format!("{}/{}/{}/{}", base_topic, node_id, prop_id, PROPERTY_SET_TOPIC),
            qos: QoS::ExactlyOnce,
        }))
    }

    pub fn unsubscribe_props<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Unsubscribe> + 'a, Homie5ProtocolError> {
        self.unsubscribe_props_for_id(&self.id, description)
    }

    pub fn unsubscribe_props_for_id<'a>(
        &'a self,
        device_id: &'a str,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Unsubscribe> + 'a, Homie5ProtocolError> {
        if !self.is_child && self.id == device_id && description.root.is_some() {
            return Err(Homie5ProtocolError::NonEmptyRootForRootDevice);
        } else if !self.is_child && self.id != device_id && Some(&self.id) != description.root.as_ref() {
            return Err(Homie5ProtocolError::RootMismatch);
        }
        let prop_iter = HomiePropertyIterator::new(description);
        Ok(prop_iter.map(move |(node_id, _, prop_id, _)| Unsubscribe {
            topic: format!(
                "{}/{}/{}/{}/{}",
                self.topic_root, HOMIE_VERSION, device_id, node_id, prop_id
            ),
        }))
    }

    pub fn remove_device<'a>(
        &'a self,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Publish> + 'a, Homie5ProtocolError> {
        self.remove_device_for_id(&self.id, description)
    }

    pub fn remove_device_for_id<'a>(
        &'a self,
        device_id: &'a str,
        description: &'a HomieDeviceDescription,
    ) -> Result<impl Iterator<Item = Publish> + 'a, Homie5ProtocolError> {
        if !self.is_child && self.id == device_id && description.root.is_some() {
            return Err(Homie5ProtocolError::NonEmptyRootForRootDevice);
        } else if !self.is_child && self.id != device_id && Some(&self.id) != description.root.as_ref() {
            return Err(Homie5ProtocolError::RootMismatch);
        }

        // clear device attributes (startes with `$state` as per convention)
        let attrs = DEVICE_ATTRIBUTES.iter().map(move |attribute| Publish {
            topic: format!("{}/{}/{}/{}", self.topic_root, HOMIE_VERSION, device_id, attribute),
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
                        topic: format!(
                            "{}/{}/{}/{}/{}/{}",
                            self.topic_root, HOMIE_VERSION, device_id, node_id, prop_id, PROPERTY_SET_TOPIC
                        ),
                        qos: QoS::ExactlyOnce,
                        retain: true,
                        payload: Vec::default(),
                    },
                    Publish {
                        topic: format!(
                            "{}/{}/{}/{}/{}/{}",
                            self.topic_root, HOMIE_VERSION, device_id, node_id, prop_id, PROPERTY_ATTRIBUTE_TARGET
                        ),
                        qos: QoS::ExactlyOnce,
                        retain: true,
                        payload: Vec::default(),
                    },
                ]
            });
        Ok(attrs.chain(props))
    }
}
