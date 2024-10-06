// "Glue Code for using homie5 with rumqttc AsyncClient
// ============================================================
//

use homie5::client::{Publish, Subscription, Unsubscribe};
use rumqttc::AsyncClient;

// This is a more advanced approach. We define a HomieMQTTClient trait that will accecpt the homi5
// mqtt types directly and convert the actions to rumqttc AsyncClient actions
#[allow(dead_code)]
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

// alternatively one could just create simple helper functions that also take the client as a
// parameter and convert the homie5 types to rumqttc types

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
