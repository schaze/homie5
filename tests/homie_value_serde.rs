//! Serde (YAML/JSON) representation tests for `HomieValue`.
//!
//! `HomieValue` has a hand-written `Deserialize` impl that must accept the canonical
//! single-key map form (`{ Bool: true }`) in EVERY context:
//!
//! - the *direct* path (a plain struct field deserialized by serde_yaml itself), where a
//!   derived externally tagged enum would demand the `!Bool true` tag form, and
//! - the *buffered* path (inside `#[serde(untagged)]` / `#[serde(tag = "...")]` /
//!   `#[serde(flatten)]` containers), where serde's `Content` buffer only supports the map
//!   form.
//!
//! The YAML tag form (`!Bool true`) additionally keeps working on the direct path.

use homie5::{HomieColorValue, HomieValue};
use serde::Deserialize;

fn all_values() -> Vec<(&'static str, HomieValue)> {
    vec![
        ("{ String: hello }", HomieValue::String("hello".to_string())),
        ("{ Integer: 5 }", HomieValue::Integer(5)),
        ("{ Float: 5.5 }", HomieValue::Float(5.5)),
        ("{ Bool: true }", HomieValue::Bool(true)),
        ("{ Enum: low }", HomieValue::Enum("low".to_string())),
        ("{ Color: 'rgb,255,0,0' }", HomieValue::Color(HomieColorValue::RGB(255, 0, 0))),
        (
            "{ DateTime: '2024-10-08T10:15:30Z' }",
            HomieValue::DateTime(chrono::DateTime::parse_from_rfc3339("2024-10-08T10:15:30Z").unwrap().into()),
        ),
        (
            "{ Duration: PT12H5M46S }",
            HomieValue::Duration(chrono::Duration::seconds(12 * 3600 + 5 * 60 + 46)),
        ),
        (
            r#"{ JSON: { temperature: 21.5 } }"#,
            HomieValue::JSON(serde_json::json!({ "temperature": 21.5 })),
        ),
    ]
}

/// Direct path: the field is deserialized by serde_yaml itself.
#[derive(Debug, Deserialize)]
struct Direct {
    value: HomieValue,
}

/// Buffered path: untagged enum forces serde to buffer through `Content`.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Buffered {
    Value { value: HomieValue },
}

impl Buffered {
    fn into_value(self) -> HomieValue {
        match self {
            Buffered::Value { value } => value,
        }
    }
}

#[test]
fn map_form_parses_on_direct_path() {
    for (yaml, expected) in all_values() {
        let parsed: Direct = serde_yaml::from_str(&format!("value: {yaml}")).unwrap_or_else(|e| panic!("direct path failed for `{yaml}`: {e}"));
        assert_eq!(parsed.value, expected, "direct path mismatch for `{yaml}`");
    }
}

#[test]
fn map_form_parses_on_buffered_path() {
    for (yaml, expected) in all_values() {
        let parsed: Buffered = serde_yaml::from_str(&format!("value: {yaml}")).unwrap_or_else(|e| panic!("buffered path failed for `{yaml}`: {e}"));
        assert_eq!(parsed.into_value(), expected, "buffered path mismatch for `{yaml}`");
    }
}

#[test]
fn tag_form_still_parses_on_direct_path() {
    let cases = [
        ("!Bool true", HomieValue::Bool(true)),
        ("!Integer 5", HomieValue::Integer(5)),
        ("!String hello", HomieValue::String("hello".to_string())),
        ("!Duration PT5S", HomieValue::Duration(chrono::Duration::seconds(5))),
    ];
    for (yaml, expected) in cases {
        let parsed: Direct = serde_yaml::from_str(&format!("value: {yaml}")).unwrap_or_else(|e| panic!("tag form failed for `{yaml}`: {e}"));
        assert_eq!(parsed.value, expected, "tag form mismatch for `{yaml}`");
    }
}

#[test]
fn empty_parses_as_bare_string_and_map_form() {
    let direct: Direct = serde_yaml::from_str("value: Empty").unwrap();
    assert_eq!(direct.value, HomieValue::Empty);

    let buffered: Buffered = serde_yaml::from_str("value: Empty").unwrap();
    assert_eq!(buffered.into_value(), HomieValue::Empty);

    // JSON-style map form with null payload
    let direct: Direct = serde_yaml::from_str("value: { Empty: null }").unwrap();
    assert_eq!(direct.value, HomieValue::Empty);
}

#[test]
fn json_round_trip_is_unchanged() {
    for (_, value) in all_values() {
        let json = serde_json::to_string(&value).unwrap();
        let back: HomieValue = serde_json::from_str(&json).unwrap_or_else(|e| panic!("JSON round trip failed for {json}: {e}"));
        assert_eq!(back, value, "JSON round trip mismatch for {json}");
    }
    // Unit variant serializes as the bare string "Empty"
    assert_eq!(serde_json::to_string(&HomieValue::Empty).unwrap(), "\"Empty\"");
    assert_eq!(serde_json::from_str::<HomieValue>("\"Empty\"").unwrap(), HomieValue::Empty);
}

#[test]
fn unknown_variant_and_multi_key_maps_are_rejected() {
    assert!(serde_yaml::from_str::<Direct>("value: { Boolean: true }").is_err());
    assert!(serde_yaml::from_str::<Direct>("value: { Bool: true, Integer: 5 }").is_err());
    assert!(serde_yaml::from_str::<Direct>("value: NotAVariant").is_err());
}

#[test]
fn invalid_payloads_error_with_context() {
    assert!(serde_yaml::from_str::<Direct>("value: { Duration: nonsense }").is_err());
    assert!(serde_yaml::from_str::<Direct>("value: { DateTime: nonsense }").is_err());
    assert!(serde_yaml::from_str::<Direct>("value: { Color: nonsense }").is_err());
}
