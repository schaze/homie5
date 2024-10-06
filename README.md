# homie5

This is a very low level implemenation of the homie5 protocol in rust.
It aims to be as flexible and unopinionated as possible. There is no direct dependency to a mqtt library.
`homie5` provides a basically support for a protocol implementation for homie5 with clearly defined interface point to a mqtt library.
The library provides fully typed support for all homie5 datatypes.

Due to this, the usage of the library is a bit more involved as with a completly ready to use homie library. Benefit is however that you can use the library basically everywhere from a simple esp32, raspberrypi to a x86 machine.

## Content

<!-- TOC start (generated with https://github.com/derlin/bitdowntoc) -->

- [Installation and usage](#installation-and-usage)
- [Examples](#examples)
- [Documentation](#documentation)
  - [Library outline](#library-outline)
  - [MQTT "bindings"](#mqtt-bindings)
    - [Simple rumqttc binding](#simple-rumqttc-binding)
    - [More advanced rumqttc binding](#more-advanced-rumqttc-binding)
    - [ESP32MqttClient binding](#esp32mqttclient-binding)
  - [Parsing MQTT messages](#parsing-mqtt-messages)
  - [Parsing and constructing HomieValues](#parsing-and-constructing-homievalues)
  - [Homie Device implementation](#homie-device-implementation)
  - [Homie Controller implemention](#homie-controller-implemention)
- [References](#references)
- [Contributing](#contributing)
- [License](#license)

<!-- TOC end -->

<!-- TOC --><a name="installation-and-usage"></a>

# Installation and usage

Some details...
`cargo add homie5`

<!-- TOC --><a name="examples"></a>

# Examples

You can find working examples for both device and controller use case in the `examples/` folder:

- controller_example.rs
  Implements a homie5 controller that will discover all homie5 devices on a mqtt broker and print out the devices and their property updates ([more information](./examples/README_controller.md)).
- device_example.rs
  Implements a simple LightDevice with state and brightness control properties ([more information](./examples/README_device.md)).

Both examples use rumqttc as a mqtt client implementation and provide a best practice in homie5 usage and in how to integrate the 2 libraries.

<!-- TOC --><a name="documentation"></a>

# Documentation

todo: documentation

<!-- TOC --><a name="library-outline"></a>

## Library outline

Describe the different components and their usage:

- Homie5DeviceProtocol
- Homie5ControllerProtocol
- Homie5Message
- HomieDeviceDescription
- Homie MQTT "Artefacts"
- HomieValue

<!-- TOC --><a name="mqtt-bindings"></a>

## MQTT "bindings"

A few best practices to easily implement mqtt client "bindings".

<!-- TOC --><a name="simple-rumqttc-binding"></a>

### Simple rumqttc binding

Simple helper functions that take the rumqttc AsyncClient as a parameter and convert the homie5 types to rumqttc types

```rust
use homie5::client::{Publish, Subscription, Unsubscribe};
use rumqttc::AsyncClient;

fn qos_to_rumqttc(value: homie5::client::QoS) -> rumqttc::QoS {
    match value {
        homie5::client::QoS::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
        homie5::client::QoS::AtMostOnce => rumqttc::QoS::AtMostOnce,
        homie5::client::QoS::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
    }
}
fn lw_to_rumqttc(value: homie5::client::LastWill) -> rumqttc::LastWill {
    rumqttc::LastWill {
        topic: value.topic,
        message: value.message.into(),
        qos: qos_to_rumqttc(value.qos),
        retain: value.retain,
    }
}
async fn publish(client: &AsyncClient, p: Publish) -> Result<(), rumqttc::ClientError> {
    client
        .publish(p.topic, qos_to_rumqttc(p.qos), p.retain, p.payload)
        .await
}

async fn subscribe(client: &AsyncClient, subs: impl Iterator<Item = Subscription>) -> Result<(), rumqttc::ClientError> {
    for sub in subs {
        client.subscribe(sub.topic, qos_to_rumqttc(sub.qos)).await?;
    }
    Ok(())
}


```

<!-- TOC --><a name="more-advanced-rumqttc-binding"></a>

### More advanced rumqttc binding

This is a more advanced approach. We define a HomieMQTTClient trait that will accecpt the homi5e mqtt types directly and convert the actions to rumqttc AsyncClient actions

```rust

use homie5::client::{Publish, Subscription, Unsubscribe};
use rumqttc::AsyncClient;

pub trait HomieMQTTClient
where
    Self::ResultError: Send + Sync,
{
    type TargetQoS;
    type TargetLastWill;
    type ResultError;

    fn homie_map_qos(qos: homie5::client::QoS) -> Self::TargetQoS;
    fn homie_map_last_will(last_will: homie5::client::LastWill) -> Self::TargetLastWill;

    async fn homie_publish(&self, p: Publish) -> Result<(), Self::ResultError>;

    async fn homie_subscribe(&self, subs: impl Iterator<Item = Subscription> + Send) -> Result<(), Self::ResultError>;

    async fn homie_unsubscribe(&self, subs: impl Iterator<Item = Unsubscribe> + Send) -> Result<(), Self::ResultError>;
}

// Implement the trait for the rumqttc AsyncClient which will enable the client
// to directly use the homie5 mqtt artefacts
impl HomieMQTTClient for AsyncClient {
    type TargetQoS = rumqttc::QoS;
    type TargetLastWill = rumqttc::LastWill;
    type ResultError = anyhow::Error;

    fn homie_map_qos(qos: homie5::client::QoS) -> Self::TargetQoS {
        match qos {
            homie5::client::QoS::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
            homie5::client::QoS::AtMostOnce => rumqttc::QoS::AtMostOnce,
            homie5::client::QoS::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
        }
    }
    fn homie_map_last_will(last_will: homie5::client::LastWill) -> Self::TargetLastWill {
        rumqttc::LastWill {
            topic: last_will.topic,
            message: last_will.message.into(),
            qos: Self::homie_map_qos(last_will.qos),
            retain: last_will.retain,
        }
    }
    // Implementation for publishing messages
    async fn homie_publish(&self, p: Publish) -> Result<(), Self::ResultError> {
        self.publish(p.topic, Self::homie_map_qos(p.qos), p.retain, p.payload)
            .await?;
        Ok(())
    }

    // Implementation for subscribing to topics
    async fn homie_subscribe(&self, subs: impl Iterator<Item = Subscription> + Send) -> Result<(), Self::ResultError> {
        for sub in subs {
            self.subscribe(sub.topic, Self::homie_map_qos(sub.qos)).await?;
        }
        Ok(())
    }

    // Implementation for unsubscribing from topics
    async fn homie_unsubscribe(&self, subs: impl Iterator<Item = Unsubscribe> + Send) -> Result<(), Self::ResultError> {
        for sub in subs {
            self.unsubscribe(sub.topic).await?;
        }
        Ok(())
    }
}

```

<!-- TOC --><a name="esp32mqttclient-binding"></a>

### ESP32MqttClient binding

```rust
use embedded_svc::mqtt::client::QoS;
use esp_idf_svc::mqtt::client::{EspMqttClient, LwtConfiguration, MqttClientConfiguration};
use esp_idf_sys::EspError;
use homie5::{
    client::{Publish, Subscription},
};

pub fn qos_to_esp_qos(value: &homie5::client::QoS) -> QoS {
    match value {
        homie5::client::QoS::AtLeastOnce => QoS::AtLeastOnce,
        homie5::client::QoS::AtMostOnce => QoS::AtMostOnce,
        homie5::client::QoS::ExactlyOnce => QoS::ExactlyOnce,
    }
}
pub fn lw_to_esp_lw(value: &homie5::client::LastWill) -> LwtConfiguration {
    LwtConfiguration {
        topic: &value.topic,
        payload: &value.message,
        qos: qos_to_esp_qos(&value.qos),
        retain: value.retain,
    }
}

pub fn publish(client: &mut EspMqttClient<'_>, p: Publish) -> Result<(), EspError> {
    client.publish(&p.topic, qos_to_esp_qos(&p.qos), p.retain, &p.payload)?;
    Ok(())
}

pub fn subscribe(
    client: &mut EspMqttClient<'_>,
    subs: impl Iterator<Item = Subscription>,
) -> Result<(), EspError> {
    for sub in subs {
        client.subscribe(&sub.topic, qos_to_esp_qos(&sub.qos))?;
    }
    Ok(())
}


```

<!-- TOC --><a name="parsing-mqtt-messages"></a>

## Parsing MQTT messages

Show how to parse native mqtt messages to Homie5Messages and explain the different messages and their meaning.

```rust

pub enum Homie5Message {
    DeviceState {
        device: DeviceIdentifier,
        state: HomieDeviceStatus,
    },
    DeviceDescription {
        device: DeviceIdentifier,
        description: HomieDeviceDescription,
    },
    DeviceLog {
        device: DeviceIdentifier,
        log_msg: String,
    },
    DeviceAlert {
        device: DeviceIdentifier,
        alert_id: String,
        alert_msg: String,
    },

    PropertyValue {
        property: PropertyIdentifier,
        value: String,
    },
    PropertyTarget {
        property: PropertyIdentifier,
        target: String,
    },
    PropertySet {
        property: PropertyIdentifier,
        set_value: String,
    },

    Broadcast {
        topic_root: String,
        subtopic: String,
        data: String,
    },

    DeviceRemoval {
        device: DeviceIdentifier,
    },
}

```

todo: documentation

<!-- TOC --><a name="parsing-and-constructing-homievalues"></a>

## Parsing and constructing HomieValues

Explain details about HomieValue.

```rust

pub enum HomieValue {
    #[default]
    Empty,
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Enum(String),
    Color(HomieColorValue),
    DateTime(chrono::DateTime<chrono::Utc>),
    Duration(chrono::Duration),
    JSON(serde_json::Value),
}

```

<!-- TOC --><a name="homie-device-implementation"></a>

## Homie Device implementation

Show the basics of how to implement a device with homie5.
Usage of Homie5DeviceProtocol, builder for device, node and property descriptions, message flow and so on...
Also discuss number ranges, property format.

<!-- TOC --><a name="homie-controller-implemention"></a>

## Homie Controller implemention

Show basics of how to implement a controller with homie5.
Usage of Homie5ControllerProtocol, message flow and so on...

<!-- TOC --><a name="references"></a>

# References

<!-- TOC --><a name="contributing"></a>

# Contributing

<!-- TOC --><a name="license"></a>

# License

This project was released under the MIT License ([LICENSE](./LICENSE))
