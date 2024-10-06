# Controller Example

Implements a homie5 controller that will discover all homie5 devices on a mqtt broker and print out the devices and their property updates

### How to run this example

This example needs a running mqtt broker.
Set the following environment variable to make it work:

```bash
export HOMIE_MQTT_HOST=[mqtt hostname]
export HOMIE_MQTT_PORT=1883
export HOMIE_MQTT_USERNAME=[username]
export HOMIE_MQTT_PASSWORD=[password]
export HOMIE_MQTT_CLIENT_ID=[client-id]
export HOMIE_MQTT_TOPIC_ROOT=[homie-dev]

RUST_LOG=error,controller_example=debug,warn,info,error,verbose cargo run --example controller_example
```

## TODO

check the code for now. Documentation will follow.

todo: add better documentation of the example

```

```
