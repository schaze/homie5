# Device Example

Implements a simple LightDevice with state and brightness control properties

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

RUST_LOG=error,device_example=debug,warn,info,error,verbose cargo run --example device_example
```

## TODO

todo: add better documentation of the example
