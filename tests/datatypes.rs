use homie5::*;
use std::collections::HashSet;
use std::str::FromStr;

#[test]
fn test_homie_data_type_serialize() {
    let data_type = HomieDataType::Integer;
    let serialized = serde_json::to_string(&data_type).unwrap();
    assert_eq!(serialized, "\"integer\"");

    let data_type = HomieDataType::JSON;
    let serialized = serde_json::to_string(&data_type).unwrap();
    assert_eq!(serialized, "\"json\"");
}

#[test]
fn test_homie_data_type_deserialize() {
    let json_str = "\"integer\"";
    let deserialized: HomieDataType = serde_json::from_str(json_str).unwrap();
    assert_eq!(deserialized, HomieDataType::Integer);

    let json_str = "\"json\"";
    let deserialized: HomieDataType = serde_json::from_str(json_str).unwrap();
    assert_eq!(deserialized, HomieDataType::JSON);
}

#[test]
fn test_homie_data_type_to_str() {
    assert_eq!(HomieDataType::Integer.to_str(), "integer");
    assert_eq!(HomieDataType::Float.to_str(), "float");
    assert_eq!(HomieDataType::Boolean.to_str(), "boolean");
    assert_eq!(HomieDataType::String.to_str(), "string");
    assert_eq!(HomieDataType::Enum.to_str(), "enum");
    assert_eq!(HomieDataType::Color.to_str(), "color");
    assert_eq!(HomieDataType::Datetime.to_str(), "datetime");
    assert_eq!(HomieDataType::Duration.to_str(), "duration");
    assert_eq!(HomieDataType::JSON.to_str(), "json");
}

#[test]
fn test_homie_data_type_display() {
    assert_eq!(format!("{}", HomieDataType::Integer), "integer");
    assert_eq!(format!("{}", HomieDataType::Float), "float");
    assert_eq!(format!("{}", HomieDataType::Boolean), "boolean");
    assert_eq!(format!("{}", HomieDataType::String), "string");
    assert_eq!(format!("{}", HomieDataType::Enum), "enum");
    assert_eq!(format!("{}", HomieDataType::Color), "color");
    assert_eq!(format!("{}", HomieDataType::Datetime), "datetime");
    assert_eq!(format!("{}", HomieDataType::Duration), "duration");
    assert_eq!(format!("{}", HomieDataType::JSON), "json");
}

#[test]
fn test_homie_data_type_debug() {
    assert_eq!(format!("{:?}", HomieDataType::Integer), "integer");
    assert_eq!(format!("{:?}", HomieDataType::Float), "float");
    assert_eq!(format!("{:?}", HomieDataType::Boolean), "boolean");
    assert_eq!(format!("{:?}", HomieDataType::String), "string");
    assert_eq!(format!("{:?}", HomieDataType::Enum), "enum");
    assert_eq!(format!("{:?}", HomieDataType::Color), "color");
    assert_eq!(format!("{:?}", HomieDataType::Datetime), "datetime");
    assert_eq!(format!("{:?}", HomieDataType::Duration), "duration");
    assert_eq!(format!("{:?}", HomieDataType::JSON), "json");
}

#[test]
fn test_homie_data_type_from_str() {
    assert_eq!(HomieDataType::from_str("integer").unwrap(), HomieDataType::Integer);
    assert_eq!(HomieDataType::from_str("float").unwrap(), HomieDataType::Float);
    assert_eq!(HomieDataType::from_str("boolean").unwrap(), HomieDataType::Boolean);
    assert_eq!(HomieDataType::from_str("string").unwrap(), HomieDataType::String);
    assert_eq!(HomieDataType::from_str("enum").unwrap(), HomieDataType::Enum);
    assert_eq!(HomieDataType::from_str("color").unwrap(), HomieDataType::Color);
    assert_eq!(HomieDataType::from_str("datetime").unwrap(), HomieDataType::Datetime);
    assert_eq!(HomieDataType::from_str("duration").unwrap(), HomieDataType::Duration);
    assert_eq!(HomieDataType::from_str("json").unwrap(), HomieDataType::JSON);
    assert!(HomieDataType::from_str("invalid").is_err());
}

#[test]
fn test_homie_data_type_hash_and_eq() {
    let mut set = HashSet::new();
    set.insert(HomieDataType::Integer);
    set.insert(HomieDataType::Float);
    set.insert(HomieDataType::Boolean);

    assert!(set.contains(&HomieDataType::Integer));
    assert!(set.contains(&HomieDataType::Float));
    assert!(set.contains(&HomieDataType::Boolean));
    assert!(!set.contains(&HomieDataType::String));
}
