use homie5::device_description::HomieDeviceDescription;
use homie5::{
    homie_device_disconnect_steps, homie_device_publish_steps, Homie5DeviceProtocol, Homie5ProtocolError,
    HomieDeviceStatus, HomieID, HomieValue, PropertyRef, ToTopic,
};

use crate::common::HomieMQTTClient;

/// Basic homie device trait that can be implemented on any custom device struct/object.
/// This provide many convenience functions and further encapsulate the HomieMQTTClient trait
/// function calls which were extended to AsyncClient
/// Note that this is still independant of a specific mqtt client.
///
/// The reason a trait like this is not included is that even though this looks very generic it
/// will still not be usable for much more than rumqttc or a library VERY similar to it.
/// - some client implementations have mutable calls for publish and subscribe operations
/// - some client implementations are not async but sync
/// - this trait assumes you share the mqtt_client connection and only have a central event loop
///   and no seperate tasks running for each device
/// - if you would want to use this trait with and object that you access via dynamic dispatch
///   there would be issues and a crate like async_trait might be needed
///
/// Due to all these limitations I have not yet found a good way to include a truly useful and
/// generic HomieDevice abstraction for rust. If you know of one please open a issue and let me
/// know or better yet provide a pull request
pub trait HomieDevice<C>
where
    C: HomieMQTTClient + Send + Sync,
    Self::ResultError: From<C::ResultError> + From<Homie5ProtocolError> + Send + Sync,
    Self: Send + Sync,
{
    type ResultError;

    fn homie_id(&self) -> &HomieID;
    fn description(&self) -> &HomieDeviceDescription;
    fn client(&self) -> &C;
    fn protcol(&self) -> &Homie5DeviceProtocol;
    fn state(&self) -> HomieDeviceStatus;
    fn set_state(&mut self, state: HomieDeviceStatus);

    async fn publish_property_values(&mut self) -> Result<(), Self::ResultError>;
    async fn handle_set_command(&mut self, property: &PropertyRef, set_value: &str) -> Result<(), Self::ResultError>;

    async fn publish_description(&self) -> Result<(), Self::ResultError> {
        let p = self.protcol().publish_description(self.description())?;
        self.client().homie_publish(p).await?;
        Ok(())
    }

    async fn publish_state(&self) -> Result<(), Self::ResultError> {
        let p = self.protcol().publish_state(self.state());
        self.client().homie_publish(p).await?;
        Ok(())
    }

    async fn subscribe_props(&self) -> Result<(), Self::ResultError> {
        self.client()
            .homie_subscribe(self.protcol().subscribe_props(self.description())?)
            .await?;
        Ok(())
    }

    async fn unsubscribe_props(&self) -> Result<(), Self::ResultError> {
        self.client()
            .homie_unsubscribe(self.protcol().unsubscribe_props(self.description())?)
            .await?;
        Ok(())
    }

    async fn publish_value(
        &self,
        property: &PropertyRef,
        value: impl Into<String>,
    ) -> Result<HomieValue, Self::ResultError> {
        let (value, retained) = self.prepare_publish(property, &value.into())?;
        // publish the value to mqtt
        self.client()
            .homie_publish(self.protcol().publish_value_prop(property, &value, retained))
            .await?;
        Ok(value)
    }

    async fn publish_target(
        &self,
        property: &PropertyRef,
        value: impl Into<String>,
    ) -> Result<HomieValue, Self::ResultError> {
        let (value, retained) = self.prepare_publish(property, &value.into())?;
        // publish the value to mqtt
        self.client()
            .homie_publish(self.protcol().publish_target_prop(property, &value, retained))
            .await?;
        Ok(value)
    }
    fn prepare_publish(&self, property: &PropertyRef, value: &str) -> Result<(HomieValue, bool), Self::ResultError> {
        // parse the value to make sure that it conforms to the properties format requirements
        let value = self
            .description()
            .with_property(property, |prop| HomieValue::parse(value, prop))
            .ok_or(Homie5ProtocolError::PropertyNotFound)?
            .map_err(|_| Homie5ProtocolError::InvalidHomieValue)?;

        //log::debug!(
        //    "Invalid value provided for property: {} -- {:?}",
        //    property.to_topic(),
        //    err
        //);
        //log::debug!("Cannot set value for: {}", property.to_topic());
        // get the retained setting for the property
        let retained = self
            .description()
            .with_property(property, |prop| prop.retained)
            .ok_or_else(|| {
                log::debug!("Cannot set value for: {}", property.to_topic());
                Homie5ProtocolError::PropertyNotFound
            })?;

        Ok((value, retained))
    }

    async fn publish_device(&mut self) -> Result<(), Self::ResultError> {
        log::debug!("[{}] publishing", self.protcol().id());

        for step in homie_device_publish_steps() {
            match step {
                homie5::DevicePublishStep::DeviceStateInit => {
                    self.set_state(HomieDeviceStatus::Init);
                    self.publish_state().await?;
                }
                homie5::DevicePublishStep::DeviceDescription => {
                    self.publish_description().await?;
                }
                homie5::DevicePublishStep::PropertyValues => {
                    self.publish_property_values().await?;
                }
                homie5::DevicePublishStep::SubscribeProperties => {
                    self.subscribe_props().await?;
                }
                homie5::DevicePublishStep::DeviceStateReady => {
                    self.set_state(HomieDeviceStatus::Ready);
                    self.publish_state().await?;
                }
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    async fn unpublish_device(&self) -> Result<(), Self::ResultError> {
        let p = self.protcol().remove_device(self.description())?;

        for entry in p {
            self.client().homie_publish(entry).await?;
        }
        Ok(())
    }

    // Note that this will not disconnect the mqtt client
    // this is so that we can choose to share the mqtt client bewtween parent and child devices
    // which is supported in homie5.
    // In case you have only one device override this and include the disconnect of the mqtt client
    // here as well.
    async fn disconnect_device(&mut self) -> Result<(), Self::ResultError> {
        log::debug!("[{}] disconnect", self.protcol().id());
        for step in homie_device_disconnect_steps() {
            match step {
                homie5::DeviceDisconnectStep::DeviceStateDisconnect => {
                    self.set_state(HomieDeviceStatus::Disconnected);
                    self.publish_state().await?;
                }
                homie5::DeviceDisconnectStep::UnsubscribeProperties => {
                    self.unsubscribe_props().await?;
                }
            }
        }
        Ok(())
    }
}
