use std::time::Duration;

use rumqttc::AsyncClient;

use homie5::{
    device_description::{
        DeviceDescriptionBuilder, HomieDeviceDescription, HomiePropertyFormat, IntegerRange, NodeDescriptionBuilder,
        PropertyDescriptionBuilder,
    },
    Homie5DeviceProtocol, HomieDataType, HomieDeviceStatus, HomieID, HomieValue, NodeRef, PropertyRef,
    HOMIE_UNIT_PERCENT,
};

use super::HomieDevice;

pub(crate) struct LightDevice {
    id: HomieID,
    state: HomieDeviceStatus,
    desc: HomieDeviceDescription,
    light_state: bool,
    brightness: i64,
    prop_light_state: PropertyRef,
    prop_light_brightness: PropertyRef,
    mqtt_client: AsyncClient,
    protocol: Homie5DeviceProtocol,
}

impl LightDevice {
    pub fn new(id: HomieID, mqtt_client: AsyncClient, protocol: Homie5DeviceProtocol) -> Self {
        let (desc, _, prop_light_state, prop_light_brightness) =
            Self::make_device_description(protocol.topic_root(), &id);

        Self {
            id,
            state: HomieDeviceStatus::Init,
            desc,
            light_state: false,
            brightness: 0,
            prop_light_state,
            prop_light_brightness,
            mqtt_client,
            protocol,
        }
    }
    fn make_device_description(
        topic_root: &str,
        device_id: &HomieID,
    ) -> (HomieDeviceDescription, NodeRef, PropertyRef, PropertyRef) {
        let light_node = NodeRef::new(
            topic_root.to_string(),
            device_id.clone(),
            HomieID::new("light").unwrap(),
        );
        let prop_light_state = PropertyRef::from_node(light_node.clone(), HomieID::new("state").unwrap());
        let prop_light_brightness = PropertyRef::from_node(light_node.clone(), HomieID::new("brightness").unwrap());

        // Build the device description
        let desc = DeviceDescriptionBuilder::new()
            .name(Some("homie5client test-device-1".to_owned()))
            .add_node(
                light_node.id.clone(),
                NodeDescriptionBuilder::new()
                    .name(Some("Light node".to_owned()))
                    .add_property(
                        prop_light_state.id.clone(),
                        PropertyDescriptionBuilder::new(HomieDataType::Boolean)
                            .name(Some("Light state".to_owned()))
                            .format(HomiePropertyFormat::Boolean {
                                false_val: "off".to_string(),
                                true_val: "on".to_string(),
                            })
                            .retained(true)
                            .settable(true)
                            .build(),
                    )
                    .add_property(
                        prop_light_brightness.id.clone(),
                        PropertyDescriptionBuilder::new(HomieDataType::Integer)
                            .name(Some("Brightness".to_owned()))
                            .format(HomiePropertyFormat::IntegerRange(IntegerRange {
                                min: Some(0),
                                max: Some(100),
                                step: None,
                            }))
                            .unit(Some(HOMIE_UNIT_PERCENT.to_string()))
                            .retained(true)
                            .settable(true)
                            .build(),
                    )
                    .build(),
            )
            .add_node(
                HomieID::new("node-2").unwrap(),
                NodeDescriptionBuilder::new()
                    .name(Some("Second Node - no props".to_owned()))
                    .build(),
            )
            .build();
        (desc, light_node, prop_light_state, prop_light_brightness)
    }

    pub async fn disconnect_device_and_close(&mut self) -> Result<(), anyhow::Error> {
        //Pinning the inner call. stupid async trait issues - go for a synchronous client if you can.
        HomieDevice::disconnect_device(self).await?;

        // wait 1 seconds to ensure all outstanding mqtt packets have been processed by the
        // eventloop - we need to find a proper solution later (if possible) but for now
        // this has to do.
        log::debug!("Waiting for mqtt eventloop to finish processing all requests...");
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.mqtt_client.disconnect().await?;
        self.client().disconnect().await?;

        Ok(())
    }
}

impl HomieDevice<AsyncClient> for LightDevice {
    type ResultError = anyhow::Error;

    fn homie_id(&self) -> &HomieID {
        &self.id
    }

    fn description(&self) -> &HomieDeviceDescription {
        &self.desc
    }

    fn client(&self) -> &AsyncClient {
        &self.mqtt_client
    }

    fn protcol(&self) -> &Homie5DeviceProtocol {
        &self.protocol
    }

    fn state(&self) -> HomieDeviceStatus {
        self.state
    }

    fn set_state(&mut self, state: HomieDeviceStatus) {
        self.state = state;
    }

    async fn publish_property_values(&mut self) -> Result<(), Self::ResultError> {
        self.publish_value(&self.prop_light_state, HomieValue::Bool(self.light_state))
            .await?;
        self.publish_value(&self.prop_light_brightness, HomieValue::Integer(self.brightness))
            .await?;
        Ok(())
    }

    async fn handle_set_command(&mut self, property: &PropertyRef, set_value: &str) -> Result<(), Self::ResultError> {
        if property == &self.prop_light_state {
            let value = self.publish_target(property, set_value).await?;

            // update internal state representation
            self.light_state = match value {
                HomieValue::Bool(val) => val,
                _ => false,
            };

            // ==> DO some actual change on a physical self here

            self.publish_value(property, set_value).await?;

        // if message is for the light brightness property
        } else if property == &self.prop_light_brightness {
            let value = self.publish_target(property, set_value).await?;
            // update internal state representation
            self.brightness = match value {
                HomieValue::Integer(val) => val,
                _ => 0,
            };

            // ==> DO some actual change on a physical self here
            self.publish_value(property, set_value).await?;
        }
        Ok(())
    }
}
