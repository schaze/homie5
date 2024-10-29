use common::{setup_ctrlc, HomieMQTTClient, Settings};
use devices::{HomieDevice, LightDevice};
use rumqttc::{AsyncClient, EventLoop};
use std::time::Duration;
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};

use homie5::{parse_mqtt_message, Homie5DeviceProtocol, Homie5Message, HomieID, ToTopic};

mod common;
mod devices;

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
    // create a channel used for sending and receiving application events
    let (channel_tx, mut channel_rx) = channel(65535);

    // Set Ctrl-C handler to exit the application cleanly
    setup_ctrlc(channel_tx.clone(), AppEvent::Exit);

    // start of actual application logic
    // ===================================================

    // get settings from the environment variables
    let settings = common::get_settings();

    let device_id = HomieID::try_from("test-dev-1")?;

    // create all client objects
    let (protocol, mqtt_client, eventloop) = create_client(&settings, &device_id);

    // run the mqtt eventloop
    let handle = run_mqtt_eventloop(eventloop, channel_tx).await;

    // create our example Homie Light Device
    let mut device = LightDevice::new(device_id, mqtt_client, protocol);

    // run the main processing loop
    loop {
        let Some(event) = channel_rx.recv().await else {
            continue;
        };

        match &event {
            AppEvent::Homie(message) => {
                log::debug!("Event: {:#?}", message);
                let Homie5Message::PropertySet { property, set_value } = message else {
                    continue; // we only handle PropertySet messages here
                };
                // if the set message is for our device (which it always should be as we did not
                // subscribe to any other devices /set topics)
                if property.node.device.id == *device.homie_id() {
                    match device.handle_set_command(property, set_value).await {
                        Ok(_) => {
                            log::debug!("Value updated {} - {}", property.to_topic().build(), set_value);
                        }
                        Err(e) => {
                            log::debug!("{}", e);
                        }
                    }
                }
            }
            AppEvent::MqttConnect => {
                log::debug!("Connected! Publishing Device");
                device.publish_device().await?;
            }
            AppEvent::MQTT(event) => {
                if let rumqttc::Event::Incoming(rumqttc::Packet::Publish(p)) = &event {
                    log::debug!("MQTT Publish: {:#?}", p);
                }
            }
            AppEvent::MqttDisconnect => {
                log::warn!("Mqtt Disconnected unexpectedly");
            }
            AppEvent::Exit => {
                // Disconnect the device, this will set the device state to disconnected and also
                // disconnect from the mqtt broker
                device.disconnect_device_and_close().await?;
                log::debug!("Exiting main event loop");
                break;
            }
        }
    }
    handle.await??;

    log::debug!("Exiting example app");
    Ok(())
}

fn create_client(_settings: &Settings, device_id: &HomieID) -> (Homie5DeviceProtocol, AsyncClient, EventLoop) {
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
    let (protocol, last_will) = Homie5DeviceProtocol::new(device_id.clone(), _settings.homie_domain.clone());

    // finalize mqtt options with last will from protocol generator
    mqttoptions.set_last_will(AsyncClient::homie_map_last_will(last_will));

    // create rumqttc AsyncClient
    let (mqtt_client, eventloop) = rumqttc::AsyncClient::new(mqttoptions, 65535);

    (protocol, mqtt_client, eventloop)
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
