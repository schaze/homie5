use std::env;

use homie5::{device_description::HomieDeviceDescription, DeviceIdentifier, HomieDeviceStatus, DEFAULT_ROOT_TOPIC};

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

#[allow(dead_code)]
pub struct Device {
    pub ident: DeviceIdentifier,
    pub state: HomieDeviceStatus,
    pub description: Option<HomieDeviceDescription>,
}
