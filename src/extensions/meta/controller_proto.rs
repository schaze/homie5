use crate::{
    client::{QoS, Subscription, Unsubscribe},
    HomieDomain, HomieID,
};

use super::topic::{meta_topic_root, META_INFO_ATTRIBUTE};

/// Protocol implementation for a controller consuming meta overlay data.
///
/// Stateless, matching the `Homie5ControllerProtocol` pattern.
#[derive(Debug, Default, Clone)]
pub struct MetaControllerProtocol {}

impl MetaControllerProtocol {
    pub fn new() -> Self {
        Default::default()
    }

    /// Subscribe to provider discovery: `{domain}/5/$meta/+/$info`
    pub fn subscribe_provider_discovery(
        &self,
        homie_domain: &HomieDomain,
    ) -> impl Iterator<Item = Subscription> {
        std::iter::once(Subscription {
            topic: meta_topic_root(homie_domain)
                .add_attr("+")
                .add_attr(META_INFO_ATTRIBUTE)
                .build(),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Subscribe to all overlays from a specific provider: `{domain}/5/$meta/{provider_id}/+`
    pub fn subscribe_provider_overlays(
        &self,
        homie_domain: &HomieDomain,
        provider_id: &HomieID,
    ) -> impl Iterator<Item = Subscription> {
        std::iter::once(Subscription {
            topic: meta_topic_root(homie_domain)
                .add_id(provider_id)
                .add_attr("+")
                .build(),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Subscribe to all overlays from all providers: `{domain}/5/$meta/+/+`
    pub fn subscribe_all_overlays(
        &self,
        homie_domain: &HomieDomain,
    ) -> impl Iterator<Item = Subscription> {
        std::iter::once(Subscription {
            topic: meta_topic_root(homie_domain)
                .add_attr("+")
                .add_attr("+")
                .build(),
            qos: QoS::ExactlyOnce,
        })
    }

    /// Unsubscribe from provider discovery.
    pub fn unsubscribe_provider_discovery(
        &self,
        homie_domain: &HomieDomain,
    ) -> impl Iterator<Item = Unsubscribe> {
        std::iter::once(Unsubscribe {
            topic: meta_topic_root(homie_domain)
                .add_attr("+")
                .add_attr(META_INFO_ATTRIBUTE)
                .build(),
        })
    }

    /// Unsubscribe from a specific provider's overlays.
    pub fn unsubscribe_provider_overlays(
        &self,
        homie_domain: &HomieDomain,
        provider_id: &HomieID,
    ) -> impl Iterator<Item = Unsubscribe> {
        std::iter::once(Unsubscribe {
            topic: meta_topic_root(homie_domain)
                .add_id(provider_id)
                .add_attr("+")
                .build(),
        })
    }

    /// Unsubscribe from all provider overlays.
    pub fn unsubscribe_all_overlays(
        &self,
        homie_domain: &HomieDomain,
    ) -> impl Iterator<Item = Unsubscribe> {
        std::iter::once(Unsubscribe {
            topic: meta_topic_root(homie_domain)
                .add_attr("+")
                .add_attr("+")
                .build(),
        })
    }
}
