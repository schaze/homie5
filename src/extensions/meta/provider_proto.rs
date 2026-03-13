use crate::{
    client::{Publish, QoS},
    HomieDomain, HomieID,
};

use super::{
    error::MetaError,
    topic::{meta_device_overlay_topic, meta_provider_info_topic},
    types::{MetaDeviceOverlay, MetaProviderInfo},
};

/// Protocol implementation for a meta overlay **provider**.
///
/// A provider publishes overlay documents into the `$meta` namespace.
/// This is a new role — not derived from `Homie5DeviceProtocol`.
#[derive(Clone, Debug)]
pub struct MetaProviderProtocol {
    provider_id: HomieID,
    homie_domain: HomieDomain,
}

impl MetaProviderProtocol {
    pub fn new(provider_id: HomieID, homie_domain: HomieDomain) -> Self {
        Self {
            provider_id,
            homie_domain,
        }
    }

    /// Returns the provider's ID.
    pub fn provider_id(&self) -> &HomieID {
        &self.provider_id
    }

    /// Returns the Homie domain this provider operates in.
    pub fn homie_domain(&self) -> &HomieDomain {
        &self.homie_domain
    }

    /// Publish the provider's `$info` descriptor.
    ///
    /// Topic: `{domain}/5/$meta/{provider_id}/$info`
    pub fn publish_provider_info(&self, info: &MetaProviderInfo) -> Result<Publish, MetaError> {
        Ok(Publish {
            topic: meta_provider_info_topic(&self.homie_domain, &self.provider_id),
            retain: true,
            payload: serde_json::to_string(info)?.into(),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Publish a device overlay document.
    ///
    /// Topic: `{domain}/5/$meta/{provider_id}/{device_id}`
    pub fn publish_device_overlay(
        &self,
        device_id: &HomieID,
        overlay: &MetaDeviceOverlay,
    ) -> Result<Publish, MetaError> {
        Ok(Publish {
            topic: meta_device_overlay_topic(&self.homie_domain, &self.provider_id, device_id),
            retain: true,
            payload: serde_json::to_string(overlay)?.into(),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Remove a device overlay by publishing an empty retained message.
    ///
    /// Topic: `{domain}/5/$meta/{provider_id}/{device_id}`
    pub fn remove_device_overlay(&self, device_id: &HomieID) -> Publish {
        Publish {
            topic: meta_device_overlay_topic(&self.homie_domain, &self.provider_id, device_id),
            retain: true,
            payload: Vec::new(),
            qos: QoS::ExactlyOnce,
        }
    }

    /// Remove the provider's `$info` descriptor by publishing an empty retained message.
    ///
    /// Topic: `{domain}/5/$meta/{provider_id}/$info`
    pub fn remove_provider_info(&self) -> Publish {
        Publish {
            topic: meta_provider_info_topic(&self.homie_domain, &self.provider_id),
            retain: true,
            payload: Vec::new(),
            qos: QoS::ExactlyOnce,
        }
    }
}
