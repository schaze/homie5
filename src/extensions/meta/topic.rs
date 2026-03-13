use crate::{HomieDomain, HomieID, TopicBuilder};

/// The `$meta` reserved topic segment for the overlay namespace.
pub const META_TOPIC_SEGMENT: &str = "$meta";

/// The `$info` attribute published by providers to advertise themselves.
pub const META_INFO_ATTRIBUTE: &str = "$info";

/// Build the `$meta` namespace root: `{domain}/5/$meta`
pub fn meta_topic_root(homie_domain: &HomieDomain) -> TopicBuilder {
    TopicBuilder::new_for_extension(homie_domain, META_TOPIC_SEGMENT)
}

/// Build the provider info topic: `{domain}/5/$meta/{provider_id}/$info`
pub fn meta_provider_info_topic(homie_domain: &HomieDomain, provider_id: &HomieID) -> String {
    meta_topic_root(homie_domain)
        .add_id(provider_id)
        .add_attr(META_INFO_ATTRIBUTE)
        .build()
}

/// Build the device overlay topic: `{domain}/5/$meta/{provider_id}/{device_id}`
pub fn meta_device_overlay_topic(
    homie_domain: &HomieDomain,
    provider_id: &HomieID,
    device_id: &HomieID,
) -> String {
    meta_topic_root(homie_domain)
        .add_id(provider_id)
        .add_id(device_id)
        .build()
}
