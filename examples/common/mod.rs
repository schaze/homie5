pub(crate) mod mqtt;
pub(crate) use mqtt::*;
use tokio::{runtime, sync::mpsc::Sender};

use std::env;

use homie5::{HomieDomain, DEFAULT_HOMIE_DOMAIN};

pub struct Settings {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub homie_domain: HomieDomain,
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
        String::from(DEFAULT_HOMIE_DOMAIN)
    };

    Settings {
        hostname,
        port,
        username,
        password,
        client_id,
        homie_domain: topic_root.try_into().unwrap(),
    }
}

pub fn setup_ctrlc<T>(ctrl_sender: Sender<T>, exit_variant: T)
where
    T: Send + Sync + Clone + 'static,
{
    if let Err(err) = ctrlc::set_handler(move || {
        let rt = runtime::Runtime::new().unwrap();

        let ctrl_sender = ctrl_sender.clone();
        let exit_variant_clone = exit_variant.clone(); // Clone exit_variant here
        rt.block_on(async move {
            ctrl_sender
                .send(exit_variant_clone)
                .await
                .expect("Error during application shutdown!");
        });
    }) {
        log::error!("Fatal Error: Cannot set ctrl-c app exit handler:\n{:#?}", err);
        panic!("Will exit now");
    }
}
