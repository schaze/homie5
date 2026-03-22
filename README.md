# homie5

This is a very low level implemenation of the homie5 protocol in rust.
It aims to be as flexible and unopinionated as possible. There is no direct dependency to a mqtt library.
`homie5` provides basic support for a protocol implementation for homie5 with clearly defined interface point to a mqtt library.
The library provides fully typed support for all homie5 datatypes.

Due to this, the usage of the library is a bit more involved as with a completly ready to use homie library. Benefit is however that you can use the library basically everywhere from a simple esp32, raspberrypi to a x86 machine.

[![works with MQTT Homie](https://homieiot.github.io/img/works-with-homie.png)](https://homieiot.github.io/)

## Content

<!-- TOC start (generated with https://github.com/derlin/bitdowntoc) -->

- [Installation and usage](#installation-and-usage)
- [Feature Flags](#feature-flags)
- [Examples](#examples)
- [Documentation](#documentation)
  - [TLDR;](#tldr)
  - [Reference Types](#reference-types)
  - [MQTT "bindings"](#mqtt-bindings)
  - [Create device description](#create-device-description)
  - [Parsing MQTT messages](#parsing-mqtt-messages)
  - [Parsing HomieValues](#parsing-homievalues)
  - [Homie device protocol implementation](#homie-device-protocol)
  - [Homie controller protocol implementation](#homie-controller-protocol)
- [References](#references)
- [Contributing](#contributing)
- [License](#license)

<!-- TOC end -->

<!-- TOC --><a name="installation-and-usage"></a>

# Installation and usage

Add homie5 to your project:

```sh
cargo add homie5
```

The library has no MQTT client dependency — you bring your own. See [MQTT "bindings"](#mqtt-bindings)
for how to integrate with your MQTT client of choice.

<!-- TOC --><a name="feature-flags"></a>

# Feature Flags

| Feature    | Description |
|------------|-------------|
| `ext-meta` | Enables the metadata extension (`homie5::extensions::meta`) for device metadata support |

Enable features in your `Cargo.toml`:
```toml
[dependencies]
homie5 = { version = "0.8", features = ["ext-meta"] }
```

<!-- TOC --><a name="examples"></a>

# Examples

You can find working examples for both device and controller use case in the `examples/` folder:

- controller_example.rs
  Implements a homie5 controller that will discover all homie5 devices on a mqtt broker and print out the devices and their property updates ([more information](./examples/README_controller.md)).
- device_example.rs
  Implements a simple LightDevice with state and brightness control properties ([more information](./examples/README_device.md)).
- device_mini_example.rs
  A minimal device example that demonstrates the simplest possible homie5 device setup.

Both examples use rumqttc as a mqtt client implementation and provide a best practice in homie5 usage and in how to integrate the 2 libraries.

<!-- TOC --><a name="documentation"></a>

# Documentation

<!-- TOC --><a name="tldr"></a>

## TLDR;

How does this library work?

Basically homie5 provides you the means to safely generate publish pakets and subscribe requests which you can pass to your mqtt client library to run a homie device or a homie controller.
You will need to convert these types into your specific mqtt clients packages. Find more information about this under [MQTT "bindings"](#mqtt-bindings).

Besides this the libray provides a fully typed way to build a `DeviceDescription`, parse incoming messages into `Homie5Message` enum for safe handling, and parse the incoming /set value including validation according to the `DeviceDescription` into a `HomieValue`.

It might at first seem odd but there is actually no HomieDevice or HomieController struct provided with this library. Due to the fact that there is no direct dependency on a mqtt client library you will need to run the mqtt event loop by yourself and drive the protocol using the tools provided by this library.
In the future there might be features added to homie5 that will implement a Device or Controller for certain mqtt clients with more ease of use. But for now this is out of scope.

Even though the initial effort to get an application started is higher with this approach in the end the addtional code is negligable compared to the finished program.
Please check the [examples](./examples/) folder for reusable code blocks and traits that can get you started quickly with a reasonable well designed HomieDevice trait and bindings for rumqttc.

<!-- TOC --><a name="reference-types"></a>

## Reference Types

homie5 provides validated reference types for addressing entities in the Homie topic hierarchy:

- **`HomieID`** — A validated identifier (lowercase a-z, 0-9, hyphens only, non-empty). Created via
  `HomieID::try_from("my-device")` or `HomieID::new_const("my-device")` (panics on invalid).
- **`HomieDomain`** — The MQTT root topic. Variants: `Default` ("homie"), `All` ("+"), or
  `Custom(...)` for custom domains.
- **`DeviceRef`** — Identifies a device (domain + device ID).
- **`NodeRef`** — Identifies a node (device ref + node ID).
- **`PropertyRef`** — Identifies a property (device ref + node ID + property ID).
- **`PropertyPointer`** — Lightweight property identifier (node ID + property ID, without device context).

All reference types implement the `ToTopic` trait for generating MQTT topic strings.

The library also provides predefined unit constants for common measurement units:
`HOMIE_UNIT_DEGREE_CELSIUS` ("°C"), `HOMIE_UNIT_PERCENT` ("%"), `HOMIE_UNIT_VOLT` ("V"),
`HOMIE_UNIT_WATT` ("W"), `HOMIE_UNIT_LUX` ("lx"), and many more. See the `HOMIE_UNIT_*` constants.

<!-- TOC --><a name="mqtt-bindings"></a>

## MQTT "bindings"

This library provides MQTT primitives for library-agnostic message handling.

These types represent the fundamental building blocks of MQTT communication, such as publishing,
subscribing, and managing QoS levels. They serve as a common interface that can be adapted or
converted to the specific types used by any MQTT client library.

#### Purpose

These types are designed to be the lowest common denominator of MQTT functionality,
ensuring compatibility across various MQTT client libraries. By abstracting these core MQTT concepts,
this module allows for greater flexibility and modularity in MQTT-based applications.

Users of the homie5 library are expected to convert these types into the corresponding types of their chosen
MQTT client library when performing actual MQTT operations.

#### Primitives

The module includes:

- `Publish`: Represents a publish message, including the topic, payload, QoS level, and retain flag.
- `Subscription`: Represents a subscription to a specific MQTT topic with a defined QoS level.
- `Unsubscribe`: Represents a request to unsubscribe from a topic.
- `LastWill`: Represents the "Last Will" message used to notify others in case of unexpected disconnection.
- `QoS`: Represents the three levels of Quality of Service in MQTT (AtMostOnce, AtLeastOnce, ExactlyOnce).

These primitives form the backbone of MQTT communication and can be converted to their equivalents in
various MQTT libraries, making this module a flexible foundation for MQTT client implementations.

<details>
    <summary>show simple example</summary>
Simple helper functions that take the rumqttc AsyncClient as a parameter and convert the homie5 types to rumqttc types

```rust
// Create mqtt binding code for rumqttc
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

</details>

<details>
    <summary>Additional examples (advanced rumqttc and esp32)</summary>

##### Trait based approach

This is a more advanced approach. We define a HomieMQTTClient trait that will accecpt the homi5 mqtt types directly and convert the actions to rumqttc AsyncClient actions

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

##### ESP32MqttClient binding

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

</details>

<!-- TOC --><a name="create-device-description"></a>

## Create device description

To create a device description use the provided builders from homie5::device_description module:

- DeviceDescriptionBuilder
- NodeDescriptionBuilder
- PropertyDescriptionBuilder

<details>
    <summary>show example</summary>

```rust
    let light_node = NodeRef::new(HomieDomain::Default, "device-1".try_into().unwrap(), "light".try_into().unwrap());
    let prop_light_state = PropertyRef::from_node(light_node.clone(), "state".try_into().unwrap());
    let prop_light_brightness = PropertyRef::from_node(light_node.clone(), "brightness".try_into().unwrap());

    // Build the device description
    let desc = DeviceDescriptionBuilder::new()
        .name("homie5client test-device-1")
        .add_node(
            light_node.node_id().clone(),
            NodeDescriptionBuilder::new()
                .name("Light node")
                .add_property(
                    prop_light_state.prop_id().clone(),
                    PropertyDescriptionBuilder::boolean()
                        .name("Light state")
                        .boolean_labels("off", "on")
                        .settable(true)
                        .build(),
                )
                .add_property(
                    prop_light_brightness.prop_id().clone(),
                    PropertyDescriptionBuilder::integer()
                        .name("Brightness")
                        .integer_range(IntegerRange {
                            min: Some(0),
                            max: Some(100),
                            step: None,
                        })
                        // or using a native Rust range: .integer_range((0..=100_i64).into())
                        .unit(HOMIE_UNIT_PERCENT)
                        .settable(true)
                        .build(),
                )
                .build(),
        )
        .build();
```

</details>

<!-- TOC --><a name="parsing-mqtt-messages"></a>

## Parsing MQTT messages

homie5 provides an easy way to parse incoming mqtt messages into the possible homie5 protocol variantions.
Use the `parse_mqtt_message` function to parse a mqtt topic and a payload into a `Homie5Message` enum.
For a Homie Device only the Homie5Message::PropertySet and Homie5Message::Broadcast variants are relevant.
A controller will use all the other variants. (see controller_example for more details).

<details>
<summary>show example (HomieDevice)</summary>

```rust
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
                        protocol.publish_value(
                            prop_light_state.node_id(),
                            prop_light_state.prop_id(),
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
                        protocol.publish_value(
                            prop_light_brightness.node_id(),
                            prop_light_brightness.prop_id(),
                            prop_light_brightness_value.to_string(),
                            true,
                        ),
                    )
                    .await?;
                }
            }
        }
        Homie5Message::Broadcast {
            homie_domain,
            subtopic,
            data,
        } => {
            log::debug!("Broadcast received: {} | {} | {}", homie_domain, subtopic, data);
        }
        _ => (),
    }
}

```

</details>

<!-- TOC --><a name="parsing-and-constructing-homievalues"></a>

## Parsing HomieValues

homie5 provides a sum type enum called `HomieValue` which can hold all supported homie type values.

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

To parse a raw value into a HomieValue based on its property description use the `HomieValue::parse` function.

This function attempts to convert a string representation of a property value into
a specific `HomieValue` type, depending on the data type and format defined in the
associated `HomiePropertyDescription`. Supported data types include integers, floats,
booleans, strings, enums, colors, datetime, duration, and JSON.

##### Arguments

- `raw`: The raw string value to be parsed.
- `property_desc`: A reference to the property description that defines the expected data type
  and format of the property.

##### Returns

- `Ok(HomieValue)`: If the parsing is successful and the value conforms to the expected type.
- `Err(Homie5ValueConversionError)`: If parsing fails, or the value is not valid for the given type.

##### Errors

The function returns `Err(Homie5ValueConversionError)` in the following cases:

- The raw string cannot be parsed into the expected type (e.g., invalid integer or float).
- The parsed value does not conform to the expected range or set of valid values.
- The property format does not match the expected format for certain types, like enums or colors.

<details>
    <summary>show example</summary>

```rust
   Homie5Message::PropertySet { property, set_value } => {
       // parse the value
       let value = device_desc
           // get the property description for the property identifier and pass it to the parse function
           .with_property(&property, |prop| HomieValue::parse(&set_value, prop))
           // check for None and log out in case we did not find the property description (message not for this device)
           .ok_or_else(|| {
               log::debug!("Cannot set value for: {}", property.to_topic());
               Homie5ProtocolError::PropertyNotFound
           })?
           // Check the inner result and log if we had a conversion error.
           .map_err(|err| {
               log::debug!(
                   "Invalid value provided for property: {} -- {:?}",
                   property.to_topic(),
                   err
               );
               Homie5ProtocolError::InvalidPayload
           })?;
```

</details>

<!-- TOC --><a name="homie-device-protocol"></a>

## Homie device protocol implementation

Use the `Homie5DeviceProtocol` struct and its functions to run the protocol for a (or several) homie device(s).

The steps are pretty simple:

1. create a struct representing your device (usually holding the state, description, device protocol and field for the property values the device manages.
2. create the device description (see above)
3. set state to `HomieDeviceStatus::Init`
4. connect to mqtt and run the eventloop
5. publish the device (there is a helper function `homie_device_publish_steps` for this that will provide the steps in the correct order as an iterator)
6. handle /set messages (see above)

<details>
    <summary>show example for device publishing</summary>

```rust
rumqttc::Event::Incoming(rumqttc::Incoming::ConnAck(_)) => {
    log::debug!("HOMIE: Connected");
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
                publish(&mqtt_client, protocol.publish_value(
                        prop_light_state.node_id(),
                        prop_light_state.prop_id(),
                        prop_light_state_value.to_string(),
                        true,
                    )).await?;
                publish(&mqtt_client, protocol.publish_value(
                        prop_light_brightness.node_id(),
                        prop_light_brightness.prop_id(),
                        prop_light_brightness_value.to_string(),
                        true,
                    )).await?;
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

```

</details>

### Device Reconfiguration

When a device needs to change its configuration (e.g. adding/removing nodes or properties), use the
`homie_device_reconfigure_steps()` helper which provides the correct step sequence:

1. `DeviceStateInit` — Set state back to "init"
2. `UnsubscribeProperties` — Unsubscribe from old property /set topics
3. `Reconfigure` — Apply changes to the device description
4. `DeviceDescription` — Publish updated description
5. `PropertyValues` — Publish new property values
6. `SubscribeProperties` — Subscribe to new settable property /set topics
7. `DeviceStateReady` — Set state to "ready"

### Device Disconnection

For a graceful disconnect, use `homie_device_disconnect_steps()`:

1. `DeviceStateDisconnect` — Publish "disconnected" state
2. `UnsubscribeProperties` — Unsubscribe from all property /set topics

### Device Removal

To permanently remove a device from the broker, use `protocol.remove_device(&description)`.
This clears all retained messages (`$state`, `$description`, `$alert`, and all retained property values).
Controllers will receive a `Homie5Message::DeviceRemoval` when the `$state` topic is cleared.

### Property Target

Devices can publish a target value for properties using
`protocol.publish_target(node_id, prop_id, value, retain)`. This represents the intended/desired
state of a property (e.g. a thermostat set-point) and is published under `<property>/$target`.

### Device Alerts & Logging

Devices can publish log messages and alerts:

- **Logging**: `protocol.publish_log(DeviceLogLevel::Info, "sensor calibrated")` — publishes to
  `homie/5/<device-id>/$log/<level>`. Log levels: `Debug`, `Info`, `Warn`, `Error`, `Fatal`.
- **Alerts**: `protocol.publish_alert(&alert_id, "Sensor malfunction")` — publishes a retained alert.
  Clear with `protocol.publish_clear_alert(&alert_id)`.

Controllers receive these as `Homie5Message::DeviceLog` and `Homie5Message::DeviceAlert`.

### Child Devices

The Homie protocol supports device hierarchies. A root device can declare child devices
via the `children` field in its description, and child devices reference back via `root`
(and optionally `parent`):

```rust
// Root device description with a child
let root_desc = DeviceDescriptionBuilder::new()
    .name("Gateway")
    .add_child("sensor-1".try_into().unwrap())
    .build();

// Create protocol for child device
let child_protocol = root_protocol.clone_for_child("sensor-1".try_into().unwrap());

// Child device description must set `root`
let child_desc = DeviceDescriptionBuilder::new()
    .name("Temperature Sensor")
    .root("gateway".try_into().unwrap())
    .build();
```

<!-- TOC --><a name="homie-controller-protocol"></a>

## Homie controller protocol implementation

Use the `Homie5ControllerProtocol` struct and its functions to run the protocol for a homie controller.

##### The general order for discovering devices is as follows:

Connect to mqtt, run the event loop and then proceed with the following steps:

1. Start with the Subscriptions returned by `Homie5ControllerProtocol::subscribe_device_discovery`
   This will subscribe to the $state attribute of all devices.
2. When receiving a `Homie5Message::DeviceState` message, check if the device is already
   known, if not subscribe to the device using `Homie5ControllerProtocol::subscribe_device`.
   This will subscibe to all the other device attributes like $log/$description/$alert
3. When receiving a `Homie5Message::DeviceDescription` message, store the description for the
   device and subscibe to all the property values using
   `Homie5ControllerProtocol::subscribe_props`
4. after this you will start receiving `Homie5Message::PropertyValue` and
   `Homie5Message::PropertyTarget` messages for the properties of the device

<!-- TOC --><a name="references"></a>

# References

- [Homie Convention v5.0](https://homieiot.github.io/)
- [API Documentation on docs.rs](https://docs.rs/homie5)

<!-- TOC --><a name="contributing"></a>

# Contributing

Contributions are welcome! Please open an issue or pull request on
[GitHub](https://github.com/schaze/homie5).

<!-- TOC --><a name="license"></a>

# License

This project was released under the MIT License ([LICENSE](./LICENSE))
