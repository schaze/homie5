//! JSON Schema tests for `HomieValue` (feature `schemars`).
//!
//! The schema must accept a document exactly when the hand-written `Deserialize` impl accepts it.
//! Each case is a YAML document with an independently known verdict; the test checks the schema
//! (draft-07, validated with `jsonschema`) and serde (`serde_yaml_ng`, on both the direct and the
//! buffered path) against it.
#![cfg(feature = "schemars")]

use homie5::HomieValue;
use schemars::generate::SchemaSettings;
use serde::Deserialize;

/// Buffered path: an untagged container makes serde buffer the value through `Content`, as
/// `#[serde(untagged)]`, `#[serde(tag = "...")]` and `#[serde(flatten)]` config types do.
#[derive(Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
enum Buffered {
    Value(HomieValue),
}

/// Whether `Deserialize` accepts the YAML document on the direct path and on the buffered path.
fn serde_verdicts(yaml: &str) -> (bool, bool) {
    (
        serde_yaml_ng::from_str::<HomieValue>(yaml).is_ok(),
        serde_yaml_ng::from_str::<Buffered>(yaml).is_ok(),
    )
}

fn root_schema() -> serde_json::Value {
    let schema = SchemaSettings::draft07()
        .into_generator()
        .into_root_schema_for::<HomieValue>();
    serde_json::to_value(schema).unwrap()
}

/// Converts YAML to the JSON value a schema validator sees, as editors and corpus tests do.
/// YAML tags (`!Bool true`) become single-key maps (`{ Bool: true }`).
fn yaml_to_json(yaml: &str) -> serde_json::Value {
    fn convert(value: serde_yaml_ng::Value) -> serde_json::Value {
        match value {
            serde_yaml_ng::Value::Tagged(tagged) => {
                let tag = tagged.tag.to_string();
                let key = tag.strip_prefix('!').unwrap_or(&tag).to_string();
                serde_json::json!({ key: convert(tagged.value) })
            }
            serde_yaml_ng::Value::Mapping(map) => serde_json::Value::Object(
                map.into_iter()
                    .map(|(k, v)| {
                        let key = match k {
                            serde_yaml_ng::Value::String(s) => s,
                            other => serde_yaml_ng::to_string(&other).unwrap().trim().to_string(),
                        };
                        (key, convert(v))
                    })
                    .collect(),
            ),
            serde_yaml_ng::Value::Sequence(seq) => serde_json::Value::Array(seq.into_iter().map(convert).collect()),
            other => serde_json::to_value(other).unwrap(),
        }
    }
    convert(serde_yaml_ng::from_str(yaml).unwrap())
}

const ACCEPT: bool = true;
const REJECT: bool = false;

/// (YAML document, verdict of the hand-written `Deserialize`).
const CASES: &[(&str, bool)] = &[
    // Bare unit variant.
    ("Empty", ACCEPT),
    ("'Empty'", ACCEPT),
    ("Bool", REJECT),
    ("String", REJECT),
    ("empty", REJECT),
    ("~", REJECT),
    ("true", REJECT),
    ("5", REJECT),
    ("[]", REJECT),
    ("{}", REJECT),
    // Empty: the payload is ignored.
    ("{ Empty: ~ }", ACCEPT),
    ("{ Empty: 5 }", ACCEPT),
    ("{ Empty: { a: 1 } }", ACCEPT),
    // String.
    ("{ String: hello }", ACCEPT),
    ("{ String: '' }", ACCEPT),
    ("{ String: '5' }", ACCEPT),
    ("{ String: [a] }", REJECT),
    // Integer.
    ("{ Integer: 5 }", ACCEPT),
    ("{ Integer: -9223372036854775808 }", ACCEPT),
    ("{ Integer: 5.5 }", REJECT),
    ("{ Integer: '5' }", REJECT),
    ("{ Integer: true }", REJECT),
    // Float.
    ("{ Float: 5.5 }", ACCEPT),
    ("{ Float: 5 }", ACCEPT),
    ("{ Float: -1e3 }", ACCEPT),
    ("{ Float: '5.5' }", REJECT),
    ("{ Float: ~ }", REJECT),
    // Bool.
    ("{ Bool: true }", ACCEPT),
    ("{ Bool: false }", ACCEPT),
    ("{ Bool: 'true' }", REJECT),
    ("{ Bool: 1 }", REJECT),
    // Enum.
    ("{ Enum: low }", ACCEPT),
    // Color.
    ("{ Color: 'rgb,255,0,0' }", ACCEPT),
    ("{ Color: 'rgb,1.5,0,0' }", ACCEPT),
    ("{ Color: 'hsv,300,50,75' }", ACCEPT),
    ("{ Color: 'xyz,0.25,0.34' }", ACCEPT),
    ("{ Color: 'rgb,1e2,+5,.5' }", ACCEPT),
    ("{ Color: 'rgb,5.,0,0' }", ACCEPT),
    ("{ Color: 'rgb, 5,0,0' }", REJECT),
    ("{ Color: 'rgb,,0,0' }", REJECT),
    ("{ Color: 'rgb,255,0' }", REJECT),
    ("{ Color: 'rgb,255,0,0,0' }", REJECT),
    ("{ Color: 'xyz,0.25,0.34,0.41' }", REJECT),
    ("{ Color: 'cmyk,1,2,3' }", REJECT),
    ("{ Color: 'rgb,a,0,0' }", REJECT),
    ("{ Color: 'rgb,inf,0,0' }", REJECT),
    ("{ Color: 'RGB,255,0,0' }", REJECT),
    ("{ Color: 5 }", REJECT),
    // DateTime.
    ("{ DateTime: '2024-10-08T10:15:30Z' }", ACCEPT),
    ("{ DateTime: '2024-10-08T10:15:30.123+02:00' }", ACCEPT),
    ("{ DateTime: '2024-10-08T10:15:30' }", ACCEPT),
    ("{ DateTime: '2024-10-08T10:15:30.5' }", ACCEPT),
    ("{ DateTime: '2024-10-08T10:15' }", ACCEPT),
    ("{ DateTime: '2024-10-08T10:15Z' }", ACCEPT),
    ("{ DateTime: '2024-10-08t10:15:30z' }", ACCEPT),
    ("{ DateTime: '2024-10-08 10:15:30Z' }", ACCEPT),
    ("{ DateTime: '2024-10-08T10:15:30.123456789Z' }", ACCEPT),
    ("{ DateTime: '2024-1-8T1:5:3' }", ACCEPT),
    ("{ DateTime: '2024-10-08T10:15:30+0200' }", REJECT),
    ("{ DateTime: '2024-10-08 10:15:30' }", REJECT),
    ("{ DateTime: '2024-10-08T10:15.5' }", REJECT),
    ("{ DateTime: '2024-10-08T10:15:30.' }", REJECT),
    ("{ DateTime: ' 2024-10-08T10:15:30Z' }", ACCEPT),
    ("{ DateTime: '2024- 10-08T 10:15' }", ACCEPT),
    ("{ DateTime: '+2024-10-08T10:15' }", ACCEPT),
    ("{ DateTime: '024-10-08T10:15' }", ACCEPT),
    ("{ DateTime: '12024-10-08T10:15' }", REJECT),
    ("{ DateTime: '2024-10-08T10:15:30Z ' }", REJECT),
    ("{ DateTime: '2024-10-08T10:15:30Zx' }", REJECT),
    ("{ DateTime: '2024-10-08' }", REJECT),
    ("{ DateTime: 'yesterday' }", REJECT),
    ("{ DateTime: 5 }", REJECT),
    // Duration.
    ("{ Duration: PT12H5M46S }", ACCEPT),
    ("{ Duration: PT1H }", ACCEPT),
    ("{ Duration: PT30M }", ACCEPT),
    ("{ Duration: PT0S }", ACCEPT),
    ("{ Duration: PT1H10S }", ACCEPT),
    ("{ Duration: PT }", REJECT),
    ("{ Duration: PT5 }", REJECT),
    ("{ Duration: PT1S1H }", REJECT),
    ("{ Duration: P1D }", REJECT),
    ("{ Duration: pt1h }", REJECT),
    ("{ Duration: 60 }", REJECT),
    // JSON: any value.
    ("{ JSON: { temperature: 21.5 } }", ACCEPT),
    ("{ JSON: [1, 2] }", ACCEPT),
    ("{ JSON: 5 }", ACCEPT),
    ("{ JSON: ~ }", ACCEPT),
    // Map shape.
    ("{ Bool: true, Integer: 5 }", REJECT),
    ("{ Empty: ~, String: a }", REJECT),
    ("{ Boolean: true }", REJECT),
    ("{ bool: true }", REJECT),
    ("{ Empty: ~, Empty2: ~ }", REJECT),
];

/// YAML tag form: parsed on the direct path only (serde's `Content` buffer cannot hold enums);
/// a schema validator sees it as the single-key map form.
const TAGGED: &[(&str, bool)] = &[
    ("!Bool true", ACCEPT),
    ("!Integer 5", ACCEPT),
    ("!String hello", ACCEPT),
    ("!Empty ~", ACCEPT),
    ("!Bool yes", REJECT),
    ("!Boolean true", REJECT),
];

#[test]
fn schema_accepts_the_tag_form_exactly_when_the_direct_path_does() {
    let schema = root_schema();
    let validator = jsonschema::draft7::new(&schema).expect("valid draft-07 schema");
    for &(yaml, expected) in TAGGED {
        assert_eq!(serde_verdicts(yaml).0, expected, "{yaml}: direct path");
        assert_eq!(validator.is_valid(&yaml_to_json(yaml)), expected, "{yaml}: schema");
    }
}

#[test]
fn schema_accepts_exactly_what_deserialize_accepts() {
    let schema = root_schema();
    let validator = jsonschema::draft7::new(&schema).expect("valid draft-07 schema");

    let mut mismatches = Vec::new();
    for &(yaml, expected) in CASES {
        let (direct, buffered) = serde_verdicts(yaml);
        let schema_ok = validator.is_valid(&yaml_to_json(yaml));
        if direct != expected || buffered != expected || schema_ok != expected {
            mismatches.push(format!(
                "{yaml:<45} expected {expected:<5} direct {direct:<5} buffered {buffered:<5} schema {schema_ok}"
            ));
        }
    }
    assert!(mismatches.is_empty(), "verdict mismatches:\n{}", mismatches.join("\n"));
}

/// Non-string scalars as `String`/`Enum` payloads: serde_yaml_ng's direct path reads any plain
/// scalar as a string, but the buffered path and JSON reject them. The schema follows the
/// format-independent form and rejects them; this pins that choice.
const DIRECT_YAML_ONLY: &[&str] = &["{ String: 5 }", "{ String: true }", "{ String: ~ }", "{ Enum: 1 }"];

#[test]
fn schema_rejects_scalars_only_the_direct_yaml_path_reads_as_strings() {
    let schema = root_schema();
    let validator = jsonschema::draft7::new(&schema).expect("valid draft-07 schema");
    for yaml in DIRECT_YAML_ONLY {
        assert_eq!(serde_verdicts(yaml), (true, false), "{yaml}: serde verdicts");
        assert!(!validator.is_valid(&yaml_to_json(yaml)), "{yaml}: schema accepts");
    }
}

/// Inputs that `Deserialize` rejects for reasons JSON Schema cannot express (numeric ranges,
/// calendar validity, i64 bounds). The schema accepts them; this pins the known gaps.
const SCHEMA_ONLY: &[&str] = &[
    "{ Color: 'rgb,256,0,0' }",
    "{ Color: 'hsv,361,50,75' }",
    "{ Color: 'xyz,0.9,0.9' }",
    "{ DateTime: '2024-13-08T10:15:30Z' }",
    "{ Duration: PT99999999999999999999H }",
    "{ Integer: 9223372036854775808 }",
];

#[test]
fn schema_is_wider_than_deserialize_only_for_inexpressible_checks() {
    let schema = root_schema();
    let validator = jsonschema::draft7::new(&schema).expect("valid draft-07 schema");
    for yaml in SCHEMA_ONLY {
        assert_eq!(serde_verdicts(yaml), (false, false), "{yaml}: Deserialize accepts");
        assert!(validator.is_valid(&yaml_to_json(yaml)), "{yaml}: schema rejects");
    }
}

#[test]
fn json_documents_match_deserialize() {
    let schema = root_schema();
    let validator = jsonschema::draft7::new(&schema).expect("valid draft-07 schema");
    for (json, expected) in [
        (r#""Empty""#, ACCEPT),
        (r#"{"Integer": 5}"#, ACCEPT),
        (r#"{"Float": 5}"#, ACCEPT),
        (r#"{"Bool": true, "Float": 1.0}"#, REJECT),
    ] {
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(
            serde_json::from_str::<HomieValue>(json).is_ok(),
            expected,
            "{json}: serde"
        );
        assert_eq!(validator.is_valid(&value), expected, "{json}: schema");
    }

    // Known gap: JSON Schema counts 5.0 as an integer, serde's i64 does not.
    let json = r#"{"Integer": 5.0}"#;
    assert!(serde_json::from_str::<HomieValue>(json).is_err());
    assert!(validator.is_valid(&serde_json::from_str::<serde_json::Value>(json).unwrap()));
}

#[test]
fn schema_is_a_documented_named_definition() {
    #[derive(schemars::JsonSchema)]
    #[allow(dead_code)]
    struct Holder {
        value: HomieValue,
    }
    let schema = serde_json::to_value(
        SchemaSettings::draft07()
            .into_generator()
            .into_root_schema_for::<Holder>(),
    )
    .unwrap();
    assert_eq!(schema["properties"]["value"]["$ref"], "#/definitions/HomieValue");
    let def = &schema["definitions"]["HomieValue"];
    assert!(def["description"].as_str().unwrap().contains("single-key"));
    let variants = def["oneOf"].as_array().unwrap();
    assert_eq!(variants.len(), 11);
    for variant in variants {
        assert!(variant["description"].is_string(), "undocumented variant: {variant}");
    }
}
