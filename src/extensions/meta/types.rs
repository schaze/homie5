use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Provider descriptor published at `{domain}/$meta/<provider>/$info`.
///
/// Every meta provider MUST publish a descriptor containing at least the `schema` version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaProviderInfo {
    /// Document schema version (currently `1`).
    pub schema: u32,
    /// Human-readable provider name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Brief description of the provider's purpose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A metadata annotation value: either a single string or a list of strings.
///
/// `List` is tried first during deserialization (serde untagged), so JSON arrays
/// always deserialize as `List` and plain strings as `Text`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetaValue {
    /// A list of string values (e.g., tags, groups).
    List(Vec<String>),
    /// A single string value (e.g., name, room, icon).
    Text(String),
}

impl MetaValue {
    /// Returns the text value if this is a `Text` variant.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MetaValue::Text(s) => Some(s.as_str()),
            MetaValue::List(_) => None,
        }
    }

    /// Returns the list value if this is a `List` variant.
    pub fn as_list(&self) -> Option<&[String]> {
        match self {
            MetaValue::List(v) => Some(v.as_slice()),
            MetaValue::Text(_) => None,
        }
    }

    /// Creates a `Text` value.
    pub fn text(s: impl Into<String>) -> Self {
        MetaValue::Text(s.into())
    }

    /// Creates a `List` value.
    pub fn list(v: impl IntoIterator<Item = impl Into<String>>) -> Self {
        MetaValue::List(v.into_iter().map(Into::into).collect())
    }

    /// Collects all string values into a `Vec`, regardless of variant.
    /// `Text("a")` → `["a"]`, `List(["a","b"])` → `["a","b"]`.
    fn to_values(&self) -> Vec<&str> {
        match self {
            MetaValue::Text(s) => vec![s.as_str()],
            MetaValue::List(v) => v.iter().map(String::as_str).collect(),
        }
    }
}

/// Arbitrary metadata annotations as key-value pairs.
///
/// Keys are free-form strings. Values are either a single string or a list of strings.
pub type MetaEntries = HashMap<String, MetaValue>;

/// Per-device overlay document published at `{domain}/$meta/<provider>/<device-id>`.
///
/// The document contains an optional `device` member that holds the full metadata tree:
/// device-level annotations, nodes, and properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetaDeviceOverlay {
    /// Document schema version.
    pub schema: u32,
    /// Device-level metadata (annotations, nodes, and properties).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device: Option<MetaDeviceLevel>,
}

/// Device-level metadata: annotations and an optional map of node-level metadata.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetaDeviceLevel {
    /// Device-level annotation entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<MetaEntries>,
    /// Per-node metadata, keyed by Homie node ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nodes: Option<HashMap<String, MetaNodeLevel>>,
}

/// Node-level metadata: annotations and an optional map of property-level metadata.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetaNodeLevel {
    /// Node-level annotation entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<MetaEntries>,
    /// Per-property metadata, keyed by Homie property ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, MetaPropertyLevel>>,
}

/// Property-level metadata: annotations only.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetaPropertyLevel {
    /// Property-level annotation entries.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<MetaEntries>,
}

// ── Merge helpers ────────────────────────────────────────────────────────────

/// Merge multiple annotation maps into a consolidated view.
///
/// For keys that appear in only one source, the value passes through unchanged.
/// For keys that appear in multiple sources:
/// - All values are collected into a single `List`, deduplicated, preserving first-seen order.
/// - A `Text` value is treated as a single-element list for merging purposes.
pub fn merge_meta_entries<'a>(sources: impl IntoIterator<Item = &'a MetaEntries>) -> MetaEntries {
    // Collect all values per key, preserving source order.
    let mut collected: HashMap<String, Vec<&MetaValue>> = HashMap::new();
    for source in sources {
        for (key, value) in source {
            collected.entry(key.clone()).or_default().push(value);
        }
    }

    let mut result = MetaEntries::new();
    for (key, values) in collected {
        if values.len() == 1 {
            // Single source — pass through unchanged.
            result.insert(key, values[0].clone());
        } else {
            // Multiple sources — merge into deduplicated list.
            let mut seen = Vec::new();
            for val in &values {
                for s in val.to_values() {
                    if !seen.contains(&s) {
                        seen.push(s);
                    }
                }
            }
            result.insert(key, MetaValue::List(seen.into_iter().map(String::from).collect()));
        }
    }
    result
}

/// Merge multiple device overlays into a single consolidated overlay.
///
/// Merges device-level annotations, then per-node annotations, then per-property
/// annotations. The `schema` field is taken as the maximum across all overlays.
pub fn merge_device_overlays<'a>(
    overlays: impl IntoIterator<Item = &'a MetaDeviceOverlay>,
) -> MetaDeviceOverlay {
    let overlays: Vec<&MetaDeviceOverlay> = overlays.into_iter().collect();

    let schema = overlays.iter().map(|o| o.schema).max().unwrap_or(0);

    let device_levels: Vec<&MetaDeviceLevel> =
        overlays.iter().filter_map(|o| o.device.as_ref()).collect();

    if device_levels.is_empty() {
        return MetaDeviceOverlay { schema, device: None };
    }

    // Merge device-level annotations.
    let ann_sources: Vec<&MetaEntries> = device_levels
        .iter()
        .filter_map(|d| d.annotations.as_ref())
        .collect();
    let annotations = if ann_sources.is_empty() {
        None
    } else {
        Some(merge_meta_entries(ann_sources))
    };

    // Collect all node IDs across overlays.
    let mut all_node_ids: Vec<String> = Vec::new();
    for dl in &device_levels {
        if let Some(nodes) = &dl.nodes {
            for key in nodes.keys() {
                if !all_node_ids.contains(key) {
                    all_node_ids.push(key.clone());
                }
            }
        }
    }

    let nodes = if all_node_ids.is_empty() {
        None
    } else {
        let mut merged_nodes = HashMap::new();
        for node_id in all_node_ids {
            // Merge node-level annotations.
            let node_ann_sources: Vec<&MetaEntries> = device_levels
                .iter()
                .filter_map(|d| d.nodes.as_ref())
                .filter_map(|nodes| nodes.get(&node_id))
                .filter_map(|n| n.annotations.as_ref())
                .collect();

            let node_annotations = if node_ann_sources.is_empty() {
                None
            } else {
                Some(merge_meta_entries(node_ann_sources))
            };

            // Collect all property IDs for this node across overlays.
            let mut all_prop_ids: Vec<String> = Vec::new();
            for dl in &device_levels {
                if let Some(nodes) = &dl.nodes {
                    if let Some(node) = nodes.get(&node_id) {
                        if let Some(props) = &node.properties {
                            for key in props.keys() {
                                if !all_prop_ids.contains(key) {
                                    all_prop_ids.push(key.clone());
                                }
                            }
                        }
                    }
                }
            }

            let properties = if all_prop_ids.is_empty() {
                None
            } else {
                let mut merged_props = HashMap::new();
                for prop_id in all_prop_ids {
                    let prop_ann_sources: Vec<&MetaEntries> = device_levels
                        .iter()
                        .filter_map(|d| d.nodes.as_ref())
                        .filter_map(|nodes| nodes.get(&node_id))
                        .filter_map(|n| n.properties.as_ref())
                        .filter_map(|props| props.get(&prop_id))
                        .filter_map(|p| p.annotations.as_ref())
                        .collect();

                    if !prop_ann_sources.is_empty() {
                        merged_props.insert(
                            prop_id,
                            MetaPropertyLevel {
                                annotations: Some(merge_meta_entries(prop_ann_sources)),
                            },
                        );
                    }
                }
                Some(merged_props)
            };

            merged_nodes.insert(
                node_id,
                MetaNodeLevel {
                    annotations: node_annotations,
                    properties,
                },
            );
        }
        Some(merged_nodes)
    };

    MetaDeviceOverlay {
        schema,
        device: Some(MetaDeviceLevel { annotations, nodes }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── MetaValue serde ──────────────────────────────────────────────────

    #[test]
    fn test_meta_value_text_roundtrip() {
        let val = MetaValue::text("hello");
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#""hello""#);
        let parsed: MetaValue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, val);
    }

    #[test]
    fn test_meta_value_list_roundtrip() {
        let val = MetaValue::list(["a", "b", "c"]);
        let json = serde_json::to_string(&val).unwrap();
        assert_eq!(json, r#"["a","b","c"]"#);
        let parsed: MetaValue = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, val);
    }

    #[test]
    fn test_meta_value_accessors() {
        let text = MetaValue::text("hello");
        assert_eq!(text.as_text(), Some("hello"));
        assert_eq!(text.as_list(), None);

        let list = MetaValue::list(["a", "b"]);
        assert_eq!(list.as_text(), None);
        assert_eq!(list.as_list(), Some(&["a".to_string(), "b".to_string()][..]));
    }

    #[test]
    fn test_meta_entries_roundtrip() {
        let mut entries = MetaEntries::new();
        entries.insert("name".into(), MetaValue::text("Living Room Light"));
        entries.insert("tags".into(), MetaValue::list(["zigbee", "light"]));
        entries.insert("icon".into(), MetaValue::text("mdi:lightbulb"));

        let json = serde_json::to_string(&entries).unwrap();
        let parsed: MetaEntries = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, entries);
    }

    // ── Document structure serde ─────────────────────────────────────────

    #[test]
    fn test_full_overlay_roundtrip() {
        let overlay = MetaDeviceOverlay {
            schema: 2,
            device: Some(MetaDeviceLevel {
                annotations: Some(HashMap::from([
                    ("name".into(), MetaValue::text("Test Device")),
                    ("tags".into(), MetaValue::list(["zigbee"])),
                ])),
                nodes: Some(HashMap::from([(
                    "sensor".into(),
                    MetaNodeLevel {
                        annotations: Some(HashMap::from([(
                            "name".into(),
                            MetaValue::text("Temp Sensor"),
                        )])),
                        properties: Some(HashMap::from([(
                            "temp".into(),
                            MetaPropertyLevel {
                                annotations: Some(HashMap::from([(
                                    "icon".into(),
                                    MetaValue::text("mdi:thermometer"),
                                )])),
                            },
                        )])),
                    },
                )])),
            }),
        };
        let json = serde_json::to_string(&overlay).unwrap();
        let parsed: MetaDeviceOverlay = serde_json::from_str(&json).unwrap();
        assert_eq!(overlay, parsed);
    }

    #[test]
    fn test_overlay_from_json() {
        let json = r#"{
            "schema": 2,
            "device": {
                "annotations": {
                    "name": "Bedroom Window",
                    "room": "bedroom",
                    "groups": ["windows"]
                },
                "nodes": {
                    "contact": {
                        "annotations": {
                            "name": "Window Contact"
                        },
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

        let overlay: MetaDeviceOverlay = serde_json::from_str(json).unwrap();
        assert_eq!(overlay.schema, 2);

        let device = overlay.device.unwrap();
        let ann = device.annotations.unwrap();
        assert_eq!(ann.get("name").and_then(MetaValue::as_text), Some("Bedroom Window"));
        assert_eq!(ann.get("room").and_then(MetaValue::as_text), Some("bedroom"));
        assert_eq!(
            ann.get("groups").and_then(MetaValue::as_list),
            Some(&["windows".to_string()][..])
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
        assert_eq!(prop_ann.get("name").and_then(MetaValue::as_text), Some("State"));
        assert_eq!(
            prop_ann.get("icon").and_then(MetaValue::as_text),
            Some("mdi:window-closed")
        );
    }

    #[test]
    fn test_overlay_device_annotations_only() {
        let json = r#"{"schema": 2, "device": {"annotations": {"name": "Lamp"}}}"#;
        let overlay: MetaDeviceOverlay = serde_json::from_str(json).unwrap();
        let device = overlay.device.unwrap();
        assert!(device.nodes.is_none());
        assert_eq!(
            device.annotations.unwrap().get("name").and_then(MetaValue::as_text),
            Some("Lamp")
        );
    }

    #[test]
    fn test_overlay_minimal() {
        let json = r#"{"schema": 2}"#;
        let overlay: MetaDeviceOverlay = serde_json::from_str(json).unwrap();
        assert_eq!(overlay.schema, 2);
        assert!(overlay.device.is_none());
    }

    // ── merge_meta_entries ───────────────────────────────────────────────

    #[test]
    fn test_merge_single_source_passthrough() {
        let mut src = MetaEntries::new();
        src.insert("name".into(), MetaValue::text("Device A"));
        src.insert("tags".into(), MetaValue::list(["zigbee"]));

        let merged = merge_meta_entries([&src]);
        assert_eq!(merged.get("name"), Some(&MetaValue::text("Device A")));
        assert_eq!(merged.get("tags"), Some(&MetaValue::list(["zigbee"])));
    }

    #[test]
    fn test_merge_text_text_same_key() {
        let mut a = MetaEntries::new();
        a.insert("room".into(), MetaValue::text("livingroom"));
        let mut b = MetaEntries::new();
        b.insert("room".into(), MetaValue::text("dining room"));

        let merged = merge_meta_entries([&a, &b]);
        assert_eq!(
            merged.get("room"),
            Some(&MetaValue::list(["livingroom", "dining room"]))
        );
    }

    #[test]
    fn test_merge_list_list_same_key() {
        let mut a = MetaEntries::new();
        a.insert("tags".into(), MetaValue::list(["zigbee", "light"]));
        let mut b = MetaEntries::new();
        b.insert("tags".into(), MetaValue::list(["light", "favorite"]));

        let merged = merge_meta_entries([&a, &b]);
        assert_eq!(
            merged.get("tags"),
            Some(&MetaValue::list(["zigbee", "light", "favorite"]))
        );
    }

    #[test]
    fn test_merge_mixed_text_list() {
        let mut a = MetaEntries::new();
        a.insert("tags".into(), MetaValue::text("zigbee"));
        let mut b = MetaEntries::new();
        b.insert("tags".into(), MetaValue::list(["bluetooth", "zigbee"]));

        let merged = merge_meta_entries([&a, &b]);
        assert_eq!(
            merged.get("tags"),
            Some(&MetaValue::list(["zigbee", "bluetooth"]))
        );
    }

    #[test]
    fn test_merge_disjoint_keys() {
        let mut a = MetaEntries::new();
        a.insert("name".into(), MetaValue::text("Device A"));
        let mut b = MetaEntries::new();
        b.insert("room".into(), MetaValue::text("kitchen"));

        let merged = merge_meta_entries([&a, &b]);
        assert_eq!(merged.get("name"), Some(&MetaValue::text("Device A")));
        assert_eq!(merged.get("room"), Some(&MetaValue::text("kitchen")));
    }

    #[test]
    fn test_merge_deduplication() {
        let mut a = MetaEntries::new();
        a.insert("room".into(), MetaValue::text("kitchen"));
        let mut b = MetaEntries::new();
        b.insert("room".into(), MetaValue::text("kitchen"));

        let merged = merge_meta_entries([&a, &b]);
        assert_eq!(merged.get("room"), Some(&MetaValue::list(["kitchen"])));
    }

    // ── merge_device_overlays ────────────────────────────────────────────

    #[test]
    fn test_merge_device_overlays_basic() {
        let a = MetaDeviceOverlay {
            schema: 2,
            device: Some(MetaDeviceLevel {
                annotations: Some(HashMap::from([
                    ("name".into(), MetaValue::text("Device A")),
                    ("room".into(), MetaValue::text("kitchen")),
                ])),
                nodes: None,
            }),
        };
        let b = MetaDeviceOverlay {
            schema: 2,
            device: Some(MetaDeviceLevel {
                annotations: Some(HashMap::from([
                    ("room".into(), MetaValue::text("dining")),
                    ("icon".into(), MetaValue::text("mdi:lamp")),
                ])),
                nodes: None,
            }),
        };

        let merged = merge_device_overlays([&a, &b]);
        assert_eq!(merged.schema, 2);
        let ann = merged.device.unwrap().annotations.unwrap();
        assert_eq!(ann.get("name"), Some(&MetaValue::text("Device A")));
        assert_eq!(
            ann.get("room"),
            Some(&MetaValue::list(["kitchen", "dining"]))
        );
        assert_eq!(ann.get("icon"), Some(&MetaValue::text("mdi:lamp")));
    }

    #[test]
    fn test_merge_device_overlays_with_nodes() {
        let a = MetaDeviceOverlay {
            schema: 2,
            device: Some(MetaDeviceLevel {
                annotations: None,
                nodes: Some(HashMap::from([(
                    "sensor".into(),
                    MetaNodeLevel {
                        annotations: Some(HashMap::from([(
                            "name".into(),
                            MetaValue::text("Temp Sensor"),
                        )])),
                        properties: Some(HashMap::from([(
                            "temp".into(),
                            MetaPropertyLevel {
                                annotations: Some(HashMap::from([(
                                    "icon".into(),
                                    MetaValue::text("mdi:thermometer"),
                                )])),
                            },
                        )])),
                    },
                )])),
            }),
        };
        let b = MetaDeviceOverlay {
            schema: 2,
            device: Some(MetaDeviceLevel {
                annotations: None,
                nodes: Some(HashMap::from([(
                    "sensor".into(),
                    MetaNodeLevel {
                        annotations: Some(HashMap::from([(
                            "room".into(),
                            MetaValue::text("kitchen"),
                        )])),
                        properties: Some(HashMap::from([(
                            "temp".into(),
                            MetaPropertyLevel {
                                annotations: Some(HashMap::from([(
                                    "name".into(),
                                    MetaValue::text("Temperature"),
                                )])),
                            },
                        )])),
                    },
                )])),
            }),
        };

        let merged = merge_device_overlays([&a, &b]);
        let nodes = merged.device.unwrap().nodes.unwrap();
        let sensor = nodes.get("sensor").unwrap();
        let node_ann = sensor.annotations.as_ref().unwrap();
        assert_eq!(node_ann.get("name"), Some(&MetaValue::text("Temp Sensor")));
        assert_eq!(node_ann.get("room"), Some(&MetaValue::text("kitchen")));

        let props = sensor.properties.as_ref().unwrap();
        let temp = props.get("temp").unwrap();
        let prop_ann = temp.annotations.as_ref().unwrap();
        assert_eq!(
            prop_ann.get("icon"),
            Some(&MetaValue::text("mdi:thermometer"))
        );
        assert_eq!(
            prop_ann.get("name"),
            Some(&MetaValue::text("Temperature"))
        );
    }
}
