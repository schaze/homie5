use controller::{Device, PropertyValueStore};
use rumqttc::{AsyncClient, EventLoop};
use std::{collections::HashMap, time::Duration};
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};

use common::{setup_ctrlc, HomieMQTTClient, Settings};
use homie5::{parse_mqtt_message, Homie5ControllerProtocol, Homie5Message, HomieID, ToTopic};

mod common;
mod controller;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum AppEvent {
    Homie(Homie5Message),
    MqttConnect,
    MqttDisconnect,
    MQTT(rumqttc::Event),
    Exit,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let (channel_tx, mut channel_rx) = channel(65535);

    setup_ctrlc(channel_tx.clone(), AppEvent::Exit);

    let settings = common::get_settings();

    let (mqtt_client, eventloop, protocol) = create_client(&settings);

    let handle = run_mqtt_eventloop(eventloop, channel_tx).await;

    let mut devices: HashMap<HomieID, Device> = HashMap::new();

    loop {
        let Some(event) = channel_rx.recv().await else {
            continue;
        };

        match event {
            // DISCOVERY STEP 1/3 - get devices state messages
            // when connected subscribe to ../+/$state for all devices to begin discovery
            AppEvent::MqttConnect => {
                log::debug!("Connected! Discovering devices");
                devices.clear();
                mqtt_client
                    .homie_subscribe(protocol.discover_devices(Some(&settings.topic_root)))
                    .await?;
                mqtt_client
                    .homie_subscribe(protocol.subscribe_broadcast(Some(&settings.topic_root)))
                    .await?;
            }

            // DISCOVERY STEP 2/3 - subscibe to device
            // when a state for a device is received it is either a new device we discovered or a
            // state update for an already discovered device
            // In case it is a new device we subscribe to all of its known attributes (e.g.
            // $description,...) to get more information
            AppEvent::Homie(Homie5Message::DeviceState { device, state }) => {
                // Check if the device already exists in the map
                if let std::collections::hash_map::Entry::Occupied(mut entry) = devices.entry(device.id.clone()) {
                    // If the device exists, update its state and log the update
                    log::debug!("[{}]: Received state update: {:#?}", device.to_topic(), state);
                    state.clone_into(&mut entry.get_mut().state);
                } else {
                    log::debug!("New Device discovered: {} - starting discovery", device.to_topic());

                    // Insert the new device into the map with the provided state and an empty description
                    devices.insert(
                        device.id.clone(),
                        Device {
                            ident: device.clone(),
                            state: state.to_owned(),
                            description: None, // No description available yet for the new device
                            properties: PropertyValueStore::new(),
                        },
                    );

                    // Subscribe to the new device's MQTT topics for updates
                    mqtt_client.homie_subscribe(protocol.subscribe_device(&device)).await?;
                }
            }

            // DISCOVERY STEP 3/3 - subscibe to properties
            // when a description is received we can subscribe to all the devices properties
            // ==> This basically concludes all steps needed for discovery
            AppEvent::Homie(Homie5Message::DeviceDescription { device, description }) => {
                // Handle error if device does not exist
                let Some(existing_device) = devices.get_mut(&device.id) else {
                    log::error!(
                        "ERROR: received description for non-existing id [{}]: {:#?}",
                        device.id,
                        description
                    );
                    continue;
                };

                // Unsubscribe from old properties if we have a description for the existing device
                if let Some(old_description) = &existing_device.description {
                    mqtt_client
                        .homie_unsubscribe(protocol.unsubscribe_props(&device, old_description))
                        .await?;
                }

                // Update the device description and subscribe to new properties
                existing_device.description = Some(description.clone());
                mqtt_client
                    .homie_subscribe(protocol.subscribe_props(&device, existing_device.description.as_ref().unwrap()))
                    .await?;
            }
            AppEvent::Homie(Homie5Message::DeviceLog { device, log_msg }) => {
                log::debug!("DeviceLog: {:?}o - {}", device.to_topic(), log_msg);
            }
            AppEvent::Homie(Homie5Message::PropertyValue { property, value }) => {
                log::debug!("PropertyValue: {} - {}", property.to_topic(), value);
                let Some(device) = devices.get_mut(&property.node.device.id) else {
                    continue;
                };
                device.store_value(property, value)?;
            }
            AppEvent::Homie(Homie5Message::PropertyTarget { property, target }) => {
                log::debug!("PropertyTarget: {} - {}", property.to_topic(), target);
                let Some(device) = devices.get_mut(&property.node.device.id) else {
                    continue;
                };
                device.store_target(property, target)?;
            }
            AppEvent::Homie(Homie5Message::Broadcast {
                topic_root: _,
                subtopic: _,
                data: _,
            }) => {
                log::debug!("{:#?}", event);
            }
            AppEvent::Homie(Homie5Message::DeviceRemoval { device }) => {
                // Unsubscribe from device-specific topics
                mqtt_client
                    .homie_unsubscribe(protocol.unsubscribe_device(&device))
                    .await?;

                // If device exists in the map, remove it, otherwise continue
                let Some(dev) = devices.remove(&device.id) else {
                    continue;
                };

                // If description exists, unsubscribe from property-specific topics
                if let Some(description) = dev.description {
                    mqtt_client
                        .homie_unsubscribe(protocol.unsubscribe_props(&device, &description))
                        .await?;
                }

                log::debug!("Device removed: {}", device.id);
            }
            AppEvent::Exit => {
                log::debug!("Disconnecting mqtt");
                mqtt_client.disconnect().await?;
                break;
            }
            AppEvent::MQTT(event) => match &event {
                rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(_ca)) => {}
                rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                    log::debug!("MQTT Publish: {:#?}", p);
                }
                _ => {}
            },
            _ => {}
        }
    }
    handle.await??;

    log::debug!("Exiting example app");
    Ok(())
}

fn create_client(settings: &Settings) -> (AsyncClient, EventLoop, Homie5ControllerProtocol) {
    let mut mqttoptions =
        rumqttc::MqttOptions::new(settings.client_id.clone(), settings.hostname.clone(), settings.port);
    mqttoptions.set_credentials(settings.username.clone(), settings.password.clone());

    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_clean_session(true);

    let client = Homie5ControllerProtocol::default();
    let (mqtt_client, eventloop) = rumqttc::AsyncClient::new(mqttoptions, 65535);

    (mqtt_client, eventloop, client)
}

/// spawn the mqtt event loop task
/// this will run the mqtt eventloop, parse mqtt messages into homie5 messages or otherwise
/// keep the raw mqtt event and push them into the application eventloop
async fn run_mqtt_eventloop(mut eventloop: EventLoop, channel_tx: Sender<AppEvent>) -> JoinHandle<anyhow::Result<()>> {
    tokio::task::spawn(async move {
        let mut connected = false;
        let mut exit = false;
        loop {
            match eventloop.poll().await {
                Ok(event) => match &event {
                    rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) => {
                        if let Ok(event) = parse_mqtt_message(&p.topic, &p.payload) {
                            channel_tx.send(AppEvent::Homie(event)).await?;
                        }
                        // invalid messages get ignored for now...
                    }
                    rumqttc::Event::Incoming(rumqttc::Incoming::ConnAck(_)) => {
                        log::debug!("HOMIE: Connected");
                        connected = true;
                        channel_tx.send(AppEvent::MqttConnect).await?;
                    }
                    rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect) => {
                        log::debug!("HOMIE: Connection closed from our side. Will exit");
                        exit = true;
                    }
                    _ => (),
                },
                Err(err) => {
                    if exit {
                        break;
                    }
                    if connected {
                        connected = false;
                        channel_tx.send(AppEvent::MqttDisconnect).await?;
                    }
                    log::error!("Error connecting mqtt. {:#?}", err);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
        log::debug!("HOMIE: exiting eventloop");
        Ok(())
    })
}
