use rumqttc::{AsyncClient, EventLoop};
use std::time::Duration;

use homie5::*;
use homie5::{client::*, device_description::*};

use std::env;

use homie5::DEFAULT_ROOT_TOPIC;

pub struct Settings {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub topic_root: String,
}

pub fn get_settings() -> Settings {
    let hostname = env::var("HOMIE_MQTT_HOST").unwrap_or_default();
    let port = if let Ok(port) = env::var("HOMIE_MQTT_PORT") {
        port.parse::<u16>().expect("Not a valid number for port!")
    } else {
        1883
    };
    let username = env::var("HOMIE_MQTT_USERNAME").unwrap_or_default();
    let password = env::var("HOMIE_MQTT_PASSWORD").unwrap_or_default();
    let client_id = if let Ok(client_id) = env::var("HOMIE_MQTT_CLIENT_ID") {
        client_id
    } else {
        String::from("aslkdnlauidhwwkednwek")
    };
    let topic_root = if let Ok(topic_root) = env::var("HOMIE_MQTT_TOPIC_ROOT") {
        topic_root
    } else {
        String::from(DEFAULT_ROOT_TOPIC)
    };
    Settings {
        hostname,
        port,
        username,
        password,
        client_id,
        topic_root,
    }
}

// Create mqtt binding code for rumqttc
#[allow(dead_code)]
fn qos_to_rumqttc(value: homie5::client::QoS) -> rumqttc::QoS {
    match value {
        homie5::client::QoS::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
        homie5::client::QoS::AtMostOnce => rumqttc::QoS::AtMostOnce,
        homie5::client::QoS::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
    }
}
#[allow(dead_code)]
fn lw_to_rumqttc(value: homie5::client::LastWill) -> rumqttc::LastWill {
    rumqttc::LastWill {
        topic: value.topic,
        message: value.message.into(),
        qos: qos_to_rumqttc(value.qos),
        retain: value.retain,
    }
}
#[allow(dead_code)]
async fn publish(client: &AsyncClient, p: Publish) -> Result<(), rumqttc::ClientError> {
    client
        .publish(p.topic, qos_to_rumqttc(p.qos), p.retain, p.payload)
        .await
}

#[allow(dead_code)]
async fn subscribe(client: &AsyncClient, subs: impl Iterator<Item = Subscription>) -> Result<(), rumqttc::ClientError> {
    for sub in subs {
        client.subscribe(sub.topic, qos_to_rumqttc(sub.qos)).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // start of actual application logic
    // ===================================================

    // get settings from the environment variables
    let settings = get_settings();

    let device_id = "test-dev-1".to_owned();
    let (device_desc, _, prop_light_state, prop_light_brightness) =
        make_device_description(&settings.topic_root, &device_id);
    let mut state = HomieDeviceStatus::Init;
    let mut prop_light_state_value = false;
    let mut prop_light_brightness_value = 0;
    // create all client objects
    let (protocol, mqtt_client, mut eventloop) = create_client(&settings, &device_id);

    loop {
        match eventloop.poll().await {
            Ok(event) => match &event {
                rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                    // ===================
                    // Handle property /set message
                    // ===================
                    match parse_mqtt_message(&p.topic, &p.payload)? {
                        Homie5Message::PropertySet { property, set_value } => {
                            // parse the value (to keep it simple for the example the whole loop will
                            // fail in case of a invalid payload. Don't do it this way in real life!
                            let value = device_desc
                                .with_property(&property, |prop| HomieValue::parse(&set_value, prop))
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
                                    Homie5ProtocolError::InvalidPayload
                                })?;

                            // if the message was for light state, update our state and publish the new
                            // value
                            if property == prop_light_state {
                                if let HomieValue::Bool(value) = value {
                                    prop_light_state_value = value;
                                    log::debug!("light state: {}", prop_light_state_value);
                                    publish(
                                        &mqtt_client,
                                        protocol.publish_value_prop(
                                            &prop_light_state,
                                            prop_light_state_value.to_string(),
                                            true,
                                        ),
                                    )
                                    .await?;
                                }
                            // if the message was for light brightness, update our brightness state and publish the new
                            // value
                            } else if property == prop_light_brightness {
                                if let HomieValue::Integer(value) = value {
                                    prop_light_brightness_value = value;
                                    log::debug!("light brightness: {}", prop_light_brightness_value);
                                    publish(
                                        &mqtt_client,
                                        protocol.publish_value_prop(
                                            &prop_light_brightness,
                                            prop_light_brightness_value.to_string(),
                                            true,
                                        ),
                                    )
                                    .await?;
                                }
                            }
                        }
                        Homie5Message::Broadcast {
                            topic_root,
                            subtopic,
                            data,
                        } => {
                            log::debug!("Broadcast received: {} | {} | {}", topic_root, subtopic, data);
                        }
                        _ => (),
                    }
                    // invalid messages get ignored for now...
                }
                rumqttc::Event::Incoming(rumqttc::Incoming::ConnAck(_)) => {
                    log::debug!("HOMIE: Connected");
                    // ===================
                    // Publishing device
                    // ===================
                    log::debug!("Publishing device");
                    for step in homie_device_publish_steps() {
                        match step {
                            DevicePublishStep::DeviceStateInit => {
                                state = HomieDeviceStatus::Init;
                                publish(&mqtt_client, protocol.publish_state(state)).await?;
                            }
                            DevicePublishStep::DeviceDescription => {
                                publish(&mqtt_client, protocol.publish_description(&device_desc)?).await?;
                            }
                            DevicePublishStep::PropertyValues => {
                                publish(
                                    &mqtt_client,
                                    protocol.publish_value_prop(
                                        &prop_light_state,
                                        prop_light_state_value.to_string(),
                                        true,
                                    ),
                                )
                                .await?;
                                publish(
                                    &mqtt_client,
                                    protocol.publish_value_prop(
                                        &prop_light_brightness,
                                        prop_light_brightness_value.to_string(),
                                        true,
                                    ),
                                )
                                .await?;
                            }
                            DevicePublishStep::SubscribeProperties => {
                                subscribe(&mqtt_client, protocol.subscribe_props(&device_desc)?).await?;
                            }
                            DevicePublishStep::DeviceStateReady => {
                                state = HomieDeviceStatus::Ready;
                                publish(&mqtt_client, protocol.publish_state(state)).await?;
                            }
                        }
                    }
                }
                rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect) => {
                    log::debug!("HOMIE: Connection closed from our side. Will exit");
                }
                _ => (),
            },
            Err(err) => {
                log::error!("Error connecting mqtt. {:#?}", err);
                tokio::time::sleep(Duration::from_secs(1)).await;
                break;
            }
        }
    }

    log::debug!("Exiting example app");
    Ok(())
}

fn create_client(_settings: &Settings, device_id: &str) -> (Homie5DeviceProtocol, AsyncClient, EventLoop) {
    // start building the mqtt options
    let mut mqttoptions = rumqttc::MqttOptions::new(
        _settings.client_id.clone() + "_dev",
        _settings.hostname.clone(),
        _settings.port,
    );
    mqttoptions.set_credentials(_settings.username.clone(), _settings.password.clone());
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_clean_session(true);

    // create device protocol generater
    let (protocol, last_will) = Homie5DeviceProtocol::new(device_id.to_string(), Some(_settings.topic_root.clone()));

    // finalize mqtt options with last will from protocol generator
    mqttoptions.set_last_will(lw_to_rumqttc(last_will));

    // create rumqttc AsyncClient
    let (mqtt_client, eventloop) = rumqttc::AsyncClient::new(mqttoptions, 65535);

    (protocol, mqtt_client, eventloop)
}

fn make_device_description(
    topic_root: &str,
    device_id: &str,
) -> (
    HomieDeviceDescription,
    NodeIdentifier,
    PropertyIdentifier,
    PropertyIdentifier,
) {
    let light_node = NodeIdentifier::new(topic_root.to_string(), device_id.to_string(), "light".to_owned());
    let prop_light_state = PropertyIdentifier::from_node(light_node.clone(), "state".to_owned());
    let prop_light_brightness = PropertyIdentifier::from_node(light_node.clone(), "brightness".to_owned());

    // Build the device description
    let desc = DeviceDescriptionBuilder::new()
        .name(Some("homie5client test-device-1".to_owned()))
        .add_node(
            &light_node.id,
            NodeDescriptionBuilder::new()
                .name(Some("Light node".to_owned()))
                .add_property(
                    &prop_light_state.id,
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
                    &prop_light_brightness.id,
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
            "node-2",
            NodeDescriptionBuilder::new()
                .name(Some("Second Node - no props".to_owned()))
                .build(),
        )
        .build();
    (desc, light_node, prop_light_state, prop_light_brightness)
}
