use crate::{client::mqtt_payload_to_string, HomieDomain, HomieID, HOMIE_VERSION};

use super::{
    error::MetaError,
    topic::{META_INFO_ATTRIBUTE, META_TOPIC_SEGMENT},
    types::{MetaDeviceOverlay, MetaProviderInfo},
};

/// Parsed messages from the `$meta` overlay namespace.
#[derive(Debug, Clone)]
pub enum MetaMessage {
    /// A provider published or updated its `$info` descriptor.
    ProviderInfo {
        homie_domain: HomieDomain,
        provider_id: HomieID,
        info: MetaProviderInfo,
    },
    /// A provider's `$info` was removed (empty payload).
    ProviderRemoval {
        homie_domain: HomieDomain,
        provider_id: HomieID,
    },
    /// A provider published or updated a device overlay document.
    DeviceOverlay {
        homie_domain: HomieDomain,
        provider_id: HomieID,
        device_id: HomieID,
        overlay: MetaDeviceOverlay,
    },
    /// A device overlay was removed (empty payload).
    DeviceOverlayRemoval {
        homie_domain: HomieDomain,
        provider_id: HomieID,
        device_id: HomieID,
    },
}

/// Parse an MQTT message from the `$meta` overlay namespace.
///
/// Returns:
/// - `Ok(Some(msg))` if the topic matches `$meta` and was parsed successfully
/// - `Ok(None)` if the topic does not belong to the `$meta` namespace
/// - `Err(MetaError)` if the topic matches `$meta` but the payload is invalid
pub fn parse_meta_message(topic: &str, payload: &[u8]) -> Result<Option<MetaMessage>, MetaError> {
    let tokens: Vec<&str> = topic.split('/').collect();

    // Minimum: {domain}/5/$meta/{provider}/{target}  → 5 tokens
    if tokens.len() < 5 {
        return Ok(None);
    }

    // Verify homie version
    if tokens[1] != HOMIE_VERSION {
        return Ok(None);
    }

    // Check for $meta segment
    if tokens[2] != META_TOPIC_SEGMENT {
        return Ok(None);
    }

    // From here on the topic matched $meta — errors are real parse errors
    let homie_domain: HomieDomain = tokens[0].to_owned().try_into()?;
    let provider_id: HomieID = tokens[3].to_string().try_into()?;

    match tokens.len() {
        // {domain}/5/$meta/{provider}/$info
        5 if tokens[4] == META_INFO_ATTRIBUTE => {
            if payload.is_empty() {
                Ok(Some(MetaMessage::ProviderRemoval {
                    homie_domain,
                    provider_id,
                }))
            } else {
                let info: MetaProviderInfo =
                    serde_json::from_str(&mqtt_payload_to_string(payload)?)?;
                Ok(Some(MetaMessage::ProviderInfo {
                    homie_domain,
                    provider_id,
                    info,
                }))
            }
        }
        // {domain}/5/$meta/{provider}/{device_id}
        5 => {
            let device_id: HomieID = tokens[4].to_string().try_into()?;
            if payload.is_empty() {
                Ok(Some(MetaMessage::DeviceOverlayRemoval {
                    homie_domain,
                    provider_id,
                    device_id,
                }))
            } else {
                let overlay: MetaDeviceOverlay =
                    serde_json::from_str(&mqtt_payload_to_string(payload)?)?;
                Ok(Some(MetaMessage::DeviceOverlay {
                    homie_domain,
                    provider_id,
                    device_id,
                    overlay,
                }))
            }
        }
        _ => Err(MetaError::InvalidTopic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::meta::types::*;
    #[allow(unused_imports)]
    use std::collections::HashMap;

    #[test]
    fn test_parse_provider_info() {
        let topic = "homie/5/$meta/my-provider/$info";
        let payload = br#"{"schema":1,"title":"My Provider","description":"test"}"#;

        let msg = parse_meta_message(topic, payload).unwrap().unwrap();
        match msg {
            MetaMessage::ProviderInfo {
                provider_id, info, ..
            } => {
                assert_eq!(provider_id.as_str(), "my-provider");
                assert_eq!(info.schema, 1);
                assert_eq!(info.title.as_deref(), Some("My Provider"));
                assert_eq!(info.description.as_deref(), Some("test"));
            }
            _ => panic!("Expected ProviderInfo"),
        }
    }

    #[test]
    fn test_parse_provider_removal() {
        let topic = "homie/5/$meta/my-provider/$info";
        let payload = b"";

        let msg = parse_meta_message(topic, payload).unwrap().unwrap();
        match msg {
            MetaMessage::ProviderRemoval { provider_id, .. } => {
                assert_eq!(provider_id.as_str(), "my-provider");
            }
            _ => panic!("Expected ProviderRemoval"),
        }
    }

    #[test]
    fn test_parse_device_overlay() {
        let topic = "homie/5/$meta/my-provider/device-01";
        let payload = br#"{"schema":2,"device":{"annotations":{"name":"Living Room Light","room":"living-room","tags":["zigbee"]}}}"#;

        let msg = parse_meta_message(topic, payload).unwrap().unwrap();
        match msg {
            MetaMessage::DeviceOverlay {
                provider_id,
                device_id,
                overlay,
                ..
            } => {
                assert_eq!(provider_id.as_str(), "my-provider");
                assert_eq!(device_id.as_str(), "device-01");
                assert_eq!(overlay.schema, 2);
                let ann = overlay.device.unwrap().annotations.unwrap();
                assert_eq!(
                    ann.get("name").and_then(MetaValue::as_text),
                    Some("Living Room Light")
                );
                assert_eq!(
                    ann.get("room").and_then(MetaValue::as_text),
                    Some("living-room")
                );
                assert_eq!(
                    ann.get("tags").and_then(MetaValue::as_list),
                    Some(&["zigbee".to_string()][..])
                );
            }
            _ => panic!("Expected DeviceOverlay"),
        }
    }

    #[test]
    fn test_parse_device_overlay_removal() {
        let topic = "homie/5/$meta/my-provider/device-01";
        let payload = b"";

        let msg = parse_meta_message(topic, payload).unwrap().unwrap();
        match msg {
            MetaMessage::DeviceOverlayRemoval {
                provider_id,
                device_id,
                ..
            } => {
                assert_eq!(provider_id.as_str(), "my-provider");
                assert_eq!(device_id.as_str(), "device-01");
            }
            _ => panic!("Expected DeviceOverlayRemoval"),
        }
    }

    #[test]
    fn test_parse_non_meta_topic_returns_none() {
        let topic = "homie/5/device-01/$state";
        let payload = b"ready";
        assert!(parse_meta_message(topic, payload).unwrap().is_none());
    }

    #[test]
    fn test_parse_too_short_topic_returns_none() {
        let topic = "homie/5/$meta";
        let payload = b"";
        assert!(parse_meta_message(topic, payload).unwrap().is_none());
    }

    #[test]
    fn test_parse_wrong_version_returns_none() {
        let topic = "homie/4/$meta/provider/$info";
        let payload = b"{}";
        assert!(parse_meta_message(topic, payload).unwrap().is_none());
    }

    #[test]
    fn test_parse_too_many_segments_returns_error() {
        let topic = "homie/5/$meta/provider/device/extra";
        let payload = b"{}";
        assert!(parse_meta_message(topic, payload).is_err());
    }

    #[test]
    fn test_parse_overlay_with_nodes() {
        let payload = br#"{
            "schema": 2,
            "device": {
                "annotations": {"name": "Window Sensor"},
                "nodes": {
                    "contact": {
                        "annotations": {"name": "Window Contact"},
                        "properties": {
                            "state": {
                                "annotations": {
                                    "name": "State",
                                    "icon": "mdi:window-closed"
                                }
                            }
                        }
                    }
                }
            }
        }"#;
        let topic = "homie/5/$meta/bridge/window-sensor-01";

        let msg = parse_meta_message(topic, payload).unwrap().unwrap();
        match msg {
            MetaMessage::DeviceOverlay { overlay, .. } => {
                let device = overlay.device.unwrap();
                let dev_ann = device.annotations.unwrap();
                assert_eq!(
                    dev_ann.get("name").and_then(MetaValue::as_text),
                    Some("Window Sensor")
                );
                let nodes = device.nodes.unwrap();
                let contact = nodes.get("contact").unwrap();
                let node_ann = contact.annotations.as_ref().unwrap();
                assert_eq!(
                    node_ann.get("name").and_then(MetaValue::as_text),
                    Some("Window Contact")
                );
                let props = contact.properties.as_ref().unwrap();
                let state = props.get("state").unwrap();
                let prop_ann = state.annotations.as_ref().unwrap();
                assert_eq!(
                    prop_ann.get("name").and_then(MetaValue::as_text),
                    Some("State")
                );
                assert_eq!(
                    prop_ann.get("icon").and_then(MetaValue::as_text),
                    Some("mdi:window-closed")
                );
            }
            _ => panic!("Expected DeviceOverlay"),
        }
    }

    #[test]
    fn test_parse_custom_domain() {
        let topic = "my-domain/5/$meta/prov/dev-01";
        let payload = br#"{"schema":1}"#;

        let msg = parse_meta_message(topic, payload).unwrap().unwrap();
        match msg {
            MetaMessage::DeviceOverlay {
                homie_domain,
                provider_id,
                device_id,
                overlay,
            } => {
                assert_eq!(homie_domain.as_str(), "my-domain");
                assert_eq!(provider_id.as_str(), "prov");
                assert_eq!(device_id.as_str(), "dev-01");
                assert_eq!(overlay.schema, 1);
            }
            _ => panic!("Expected DeviceOverlay"),
        }
    }

    #[test]
    fn test_roundtrip_provider_info() {
        let info = MetaProviderInfo {
            schema: 1,
            title: Some("Test".into()),
            description: None,
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: MetaProviderInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, parsed);
    }

    #[test]
    fn test_roundtrip_device_overlay() {
        let overlay = MetaDeviceOverlay {
            schema: 2,
            device: Some(MetaDeviceLevel {
                annotations: Some(HashMap::from([
                    ("name".into(), MetaValue::text("Test Device")),
                    ("room".into(), MetaValue::text("kitchen")),
                    ("groups".into(), MetaValue::list(["lights"])),
                    ("tags".into(), MetaValue::list(["zigbee"])),
                    ("icon".into(), MetaValue::text("mdi:lightbulb")),
                    ("hidden".into(), MetaValue::text("false")),
                ])),
                nodes: None,
            }),
        };
        let json = serde_json::to_string(&overlay).unwrap();
        let parsed: MetaDeviceOverlay = serde_json::from_str(&json).unwrap();
        assert_eq!(overlay, parsed);
    }
}
