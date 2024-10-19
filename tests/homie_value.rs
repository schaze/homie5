use common::{run_homietests, HomieTest};
use device_description::*;
use homie5::*;

use chrono::{Duration, Utc};
use serde_json::json;

mod common;

#[test]
fn test_homie_boolean_value_from_file() {
    let result = run_homietests("homie5/values/boolean.yml", |test_definition| {
        if let HomieTest::PropertyValue(test) = test_definition {
            Ok(HomieValue::parse(&test.input_data, &test.definition).is_ok())
        } else {
            Err(anyhow::anyhow!("Invalid Testdefinition in test file"))
        }
    });

    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_homie_integer_value_from_file() {
    let result = run_homietests("homie5/values/integer.yml", |test_definition| {
        if let HomieTest::PropertyValueInteger(test) = test_definition {
            let homie_value = HomieValue::parse(&test.input_data, &test.definition).unwrap();
            let HomieValue::Integer(value) = homie_value else {
                return Err(anyhow::anyhow!("Invalid Testdefinition in test file"));
            };
            Ok(value == test.output_data)
        } else {
            Err(anyhow::anyhow!("Invalid Testdefinition in test file"))
        }
    });

    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_homie_color_value_display_rgb() {
    let color = HomieColorValue::RGB(255, 100, 50);
    assert_eq!(color.to_string(), "rgb,255,100,50");
}

#[test]
fn test_homie_color_value_display_hsv() {
    let color = HomieColorValue::HSV(360, 100, 100);
    assert_eq!(color.to_string(), "hsv,360,100,100");
}

#[test]
fn test_homie_color_value_display_xyz() {
    let color = HomieColorValue::XYZ(0.3, 0.4, 0.3);
    assert_eq!(color.to_string(), "xyz,0.3,0.4");
}

#[test]
fn test_homie_color_value_new_xyz() {
    let color = HomieColorValue::new_xyz(0.3, 0.4);
    assert_eq!(color, HomieColorValue::XYZ(0.3, 0.4, 0.3));
}

#[test]
fn test_homie_color_value_from_str_rgb() {
    let color_str = "rgb,255,100,50";
    let color = color_str.parse::<HomieColorValue>().unwrap();
    assert_eq!(color, HomieColorValue::RGB(255, 100, 50));
}

#[test]
fn test_homie_color_value_from_str_hsv() {
    let color_str = "hsv,360,100,100";
    let color = color_str.parse::<HomieColorValue>().unwrap();
    assert_eq!(color, HomieColorValue::HSV(360, 100, 100));
}

#[test]
fn test_homie_color_value_from_str_xyz() {
    let color_str = "xyz,0.3,0.4";
    let color = color_str.parse::<HomieColorValue>().unwrap();
    assert_eq!(color, HomieColorValue::XYZ(0.3, 0.4, 0.3));
}

#[test]
fn test_homie_color_value_from_str_invalid() {
    let color_str = "invalid,255,100,50";
    assert!(color_str.parse::<HomieColorValue>().is_err());
}

fn create_prop_desc(dt: HomieDataType, pf: HomiePropertyFormat) -> HomiePropertyDescription {
    PropertyDescriptionBuilder::new(dt).format(pf).build()
}

#[test]
fn test_homie_value_display() {
    assert_eq!(HomieValue::Empty.to_string(), "");
    assert_eq!(HomieValue::String("test".to_string()).to_string(), "test");
    assert_eq!(HomieValue::Integer(42).to_string(), "42");
    assert_eq!(HomieValue::Float(3.12).to_string(), "3.12");
    assert_eq!(HomieValue::Bool(true).to_string(), "true");
    assert_eq!(
        HomieValue::Color(HomieColorValue::RGB(255, 100, 50)).to_string(),
        "rgb,255,100,50"
    );
}

#[test]
fn test_homie_value_parse_integer() {
    let desc = create_prop_desc(
        HomieDataType::Integer,
        HomiePropertyFormat::IntegerRange(IntegerRange {
            min: Some(0),
            max: Some(100),
            step: None,
        }),
    );

    assert_eq!(HomieValue::parse("42", &desc).unwrap(), HomieValue::Integer(42));
    assert!(HomieValue::parse("200", &desc).is_err());
}

#[test]
fn test_homie_value_parse_float() {
    let desc = create_prop_desc(
        HomieDataType::Float,
        HomiePropertyFormat::FloatRange(FloatRange {
            min: Some(0.0),
            max: Some(100.0),
            step: None,
        }),
    );
    assert_eq!(HomieValue::parse("42.42", &desc).unwrap(), HomieValue::Float(42.42));
    assert!(HomieValue::parse("200.0", &desc).is_err());
}

#[test]
fn test_homie_value_parse_bool() {
    let desc = create_prop_desc(HomieDataType::Boolean, HomiePropertyFormat::Empty);
    assert_eq!(HomieValue::parse("true", &desc).unwrap(), HomieValue::Bool(true));
    assert_eq!(HomieValue::parse("false", &desc).unwrap(), HomieValue::Bool(false));
    assert!(HomieValue::parse("not_a_bool", &desc).is_err());
}

#[test]
fn test_homie_value_parse_string() {
    let desc = create_prop_desc(HomieDataType::String, HomiePropertyFormat::Empty);
    assert_eq!(
        HomieValue::parse("hello", &desc).unwrap(),
        HomieValue::String("hello".to_string())
    );
}

#[test]
fn test_homie_value_parse_enum() {
    let desc = create_prop_desc(
        HomieDataType::Enum,
        HomiePropertyFormat::Enum(vec!["option1".to_string(), "option2".to_string()]),
    );
    assert_eq!(
        HomieValue::parse("option1", &desc).unwrap(),
        HomieValue::Enum("option1".to_string())
    );
    assert!(HomieValue::parse("invalid_option", &desc).is_err());
}

#[test]
fn test_homie_value_parse_color() {
    let desc = create_prop_desc(HomieDataType::Color, HomiePropertyFormat::Color(vec![ColorFormat::Rgb]));
    assert_eq!(
        HomieValue::parse("rgb,255,100,50", &desc).unwrap(),
        HomieValue::Color(HomieColorValue::RGB(255, 100, 50))
    );
    assert!(HomieValue::parse("hsv,360,100,100", &desc).is_err()); // Invalid color format
}

#[test]
fn test_homie_value_parse_datetime() {
    let desc = create_prop_desc(HomieDataType::Datetime, HomiePropertyFormat::Empty);
    let datetime = Utc::now();
    let datetime_str = datetime.to_rfc3339();
    assert_eq!(
        HomieValue::parse(&datetime_str, &desc).unwrap(),
        HomieValue::DateTime(datetime)
    );
}

#[test]
fn test_homie_value_parse_duration() {
    let desc = create_prop_desc(HomieDataType::Duration, HomiePropertyFormat::Empty);
    assert_eq!(
        HomieValue::parse("PT1H30M10S", &desc).unwrap(),
        HomieValue::Duration(Duration::seconds(5410))
    );
}

#[test]
fn test_homie_value_parse_json() {
    let desc = create_prop_desc(HomieDataType::JSON, HomiePropertyFormat::Empty);
    assert_eq!(
        HomieValue::parse("{\"key\":\"value\"}", &desc).unwrap(),
        HomieValue::JSON(json!({"key": "value"}))
    );
}

// Helper function to create HomiePropertyDescription for floats
fn create_float_desc(min: Option<f64>, max: Option<f64>, step: Option<f64>) -> HomiePropertyDescription {
    create_prop_desc(
        HomieDataType::Float,
        HomiePropertyFormat::FloatRange(FloatRange { min, max, step }),
    )
}

// Helper function to create HomiePropertyDescription for integers
fn create_integer_desc(min: Option<i64>, max: Option<i64>, step: Option<i64>) -> HomiePropertyDescription {
    create_prop_desc(
        HomieDataType::Integer,
        HomiePropertyFormat::IntegerRange(IntegerRange { min, max, step }),
    )
}

#[test]
fn test_float_value_with_step_rounding() {
    // Example: 2:6:2 will allow values 2, 4, and 6
    let desc = create_float_desc(Some(2.0), Some(6.0), Some(2.0));

    // Value within range and aligned to step
    assert_eq!(HomieValue::parse("4.0", &desc).unwrap(), HomieValue::Float(4.0));

    // Value rounded to nearest step
    assert_eq!(HomieValue::parse("3.5", &desc).unwrap(), HomieValue::Float(4.0));
    assert_eq!(HomieValue::parse("5.9", &desc).unwrap(), HomieValue::Float(6.0));

    // Value too low (rounded to nearest step but out of range)
    assert!(HomieValue::parse("0.9", &desc).is_err());

    // Value too high (rounded to nearest step but out of range)
    assert!(HomieValue::parse("7.1", &desc).is_err());
}

#[test]
fn test_float_value_with_open_ended_range() {
    // Example: 0: allows values >= 0
    let desc = create_float_desc(Some(0.0), None, Some(0.5));

    // Value within range
    assert_eq!(HomieValue::parse("1.0", &desc).unwrap(), HomieValue::Float(1.0));

    // Value rounded to nearest step
    assert_eq!(HomieValue::parse("1.2", &desc).unwrap(), HomieValue::Float(1.0));
    assert_eq!(HomieValue::parse("1.3", &desc).unwrap(), HomieValue::Float(1.5));

    // Value too low (below min)
    assert!(HomieValue::parse("-0.6", &desc).is_err());
}

#[test]
fn test_integer_value_with_step_rounding() {
    // Example: 5:15:3 will allow values 5, 8, 11, 14
    let desc = create_integer_desc(Some(5), Some(15), Some(3));

    // Value within range and aligned to step
    assert_eq!(HomieValue::parse("8", &desc).unwrap(), HomieValue::Integer(8));

    // Value rounded to nearest step
    assert_eq!(HomieValue::parse("7", &desc).unwrap(), HomieValue::Integer(8));
    assert_eq!(HomieValue::parse("13", &desc).unwrap(), HomieValue::Integer(14));

    // Value too low (rounded but out of range)
    assert!(HomieValue::parse("3", &desc).is_err());

    // Value too high (rounded but out of range)
    assert!(HomieValue::parse("16", &desc).is_err());
}

#[test]
fn test_integer_value_with_max_only_range() {
    // Example: :20 allows values <= 20
    let desc = create_integer_desc(None, Some(10), Some(2));

    // Value within range
    assert_eq!(HomieValue::parse("8", &desc).unwrap(), HomieValue::Integer(8));

    // Value rounded to nearest step
    assert_eq!(HomieValue::parse("9", &desc).unwrap(), HomieValue::Integer(10));

    // Value too high (out of range)
    assert!(HomieValue::parse("21", &desc).is_err());
}

#[test]
fn test_integer_value_with_no_step() {
    // Example: 5:15 with no step, any value between 5 and 15 is allowed
    let desc = create_integer_desc(Some(5), Some(15), None);

    // Value within range
    assert_eq!(HomieValue::parse("10", &desc).unwrap(), HomieValue::Integer(10));

    // Value out of range
    assert!(HomieValue::parse("16", &desc).is_err());
}

#[test]
fn test_float_value_with_no_step() {
    // Example: 1.0:10.0 with no step, any value between 1.0 and 10.0 is allowed
    let desc = create_float_desc(Some(1.0), Some(10.0), None);

    // Value within range
    assert_eq!(HomieValue::parse("5.5", &desc).unwrap(), HomieValue::Float(5.5));

    // Value out of range
    assert!(HomieValue::parse("10.1", &desc).is_err());
}

#[test]
fn test_integer_value_with_step_rounding_2() {
    // Example: 5:15:3 will allow values 5, 8, 11, 14
    let desc = create_integer_desc(Some(5), Some(15), Some(3));

    // Value within range and aligned to step
    assert_eq!(HomieValue::parse("8", &desc).unwrap(), HomieValue::Integer(8));

    // Value rounded to nearest step
    assert_eq!(HomieValue::parse("4", &desc).unwrap(), HomieValue::Integer(5));

    assert_eq!(HomieValue::parse("7", &desc).unwrap(), HomieValue::Integer(8));
    assert_eq!(HomieValue::parse("13", &desc).unwrap(), HomieValue::Integer(14));

    assert_eq!(HomieValue::parse("15", &desc).unwrap(), HomieValue::Integer(14));

    // Value too low (rounded but out of range)
    assert!(HomieValue::parse("3", &desc).is_err());

    // Value too high (rounded but out of range)
    assert!(HomieValue::parse("16", &desc).is_err());
}

// ORIG

#[test]
fn test_integer_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Integer)
        .name(Some("test".to_owned()))
        .build();
    assert_eq!(HomieValue::parse("122", &desc), Ok(HomieValue::Integer(122)));
}

#[test]
fn test_integer_nok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Integer)
        .name(Some("test".to_owned()))
        .build();
    assert!(matches!(
        HomieValue::parse("bla2", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidIntegerFormat(_)
        ))
    ));
    assert!(matches!(
        HomieValue::parse("122.22", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidIntegerFormat(_)
        ))
    ));
    assert!(matches!(
        HomieValue::parse("122,22", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidIntegerFormat(_)
        ))
    ));
    assert!(matches!(
        HomieValue::parse(" 122", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidIntegerFormat(_)
        ))
    ));
}

#[test]
fn test_float_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Float).build();
    assert_eq!(HomieValue::parse("122", &desc), Ok(HomieValue::Float(122.0)));
    assert_eq!(HomieValue::parse("122.12", &desc), Ok(HomieValue::Float(122.12)));
}

#[test]
fn test_float_nok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Float).build();
    assert!(matches!(
        HomieValue::parse("bla2", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidFloatFormat(_)
        ))
    ));
    assert!(matches!(
        HomieValue::parse("122,22", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidFloatFormat(_)
        ))
    ));
    assert!(matches!(
        HomieValue::parse(" 122", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidFloatFormat(_)
        ))
    ));
}

#[test]
fn test_bool_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Boolean).build();
    assert_eq!(HomieValue::parse("true", &desc), Ok(HomieValue::Bool(true)));
    assert_eq!(HomieValue::parse("false", &desc), Ok(HomieValue::Bool(false)));
}

#[test]
fn test_bool_nok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Boolean).build();
    assert!(matches!(
        HomieValue::parse("bla2", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidBooleanFormat(_)
        ))
    ));
    assert!(matches!(
        HomieValue::parse("TRUE", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidBooleanFormat(_)
        ))
    ));
    assert!(matches!(
        HomieValue::parse("False", &desc),
        Err(Homie5ProtocolError::InvalidHomieValue(
            Homie5ValueConversionError::InvalidBooleanFormat(_)
        ))
    ));
}

#[test]
fn test_string_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::String).build();
    assert_eq!(
        HomieValue::parse("blah", &desc),
        Ok(HomieValue::String("blah".to_owned()))
    );
}

#[test]
fn test_enum_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Enum)
        .format(HomiePropertyFormat::Enum(vec!["blah".to_owned()]))
        .build();
    assert_eq!(
        HomieValue::parse("blah", &desc),
        Ok(HomieValue::Enum("blah".to_owned()))
    );
}

#[test]
fn test_color_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Color).build();
    assert_eq!(
        HomieValue::parse("rgb,12,55,14", &desc),
        Ok(HomieValue::Color(HomieColorValue::RGB(12, 55, 14)))
    );
    assert_eq!(
        HomieValue::parse("hsv,112,155,55", &desc),
        Ok(HomieValue::Color(HomieColorValue::HSV(112, 155, 55)))
    );
    assert_eq!(
        HomieValue::parse("xyz,0.33453,0.123456", &desc),
        Ok(HomieValue::Color(HomieColorValue::XYZ(
            0.33453,
            0.123456,
            1.0 - 0.33453 - 0.123456
        )))
    );
}

#[test]
fn test_color_nok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Color).build();
    assert!(HomieValue::parse("rgb,12,55", &desc).is_err());
    assert!(HomieValue::parse("HSV,12,55,14", &desc).is_err());
    assert!(HomieValue::parse("rgb ,12,55,14", &desc).is_err());
    assert!(HomieValue::parse("xyz/12,55", &desc).is_err());
}

#[test]
fn test_datetime_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Datetime).build();
    assert_eq!(
        HomieValue::parse("2023-09-26T10:54:59+00:00", &desc),
        Ok(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("2023-09-26T11:54:59+01:00", &desc),
        Ok(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("2023-09-26T10:54:59Z", &desc),
        Ok(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("2023-09-26T10:54:59", &desc),
        Ok(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("2023-09-26T10:54:59.100", &desc),
        Ok(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 100000000).unwrap()
        ))
    );
}
#[test]
fn test_duration_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Duration).build();
    assert_eq!(
        HomieValue::parse("PT12H4M2S", &desc),
        Ok(HomieValue::Duration(
            chrono::Duration::from_std(std::time::Duration::from_secs(12 * 60 * 60 + 4 * 60 + 2)).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("PT43442S", &desc),
        Ok(HomieValue::Duration(
            chrono::Duration::from_std(std::time::Duration::from_secs(12 * 60 * 60 + 4 * 60 + 2)).unwrap()
        ))
    );
}

#[test]
fn test_json_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::JSON).build();
    let json = HomieValue::parse("{ \"test\": \"test\" }", &desc);
    assert!(json.is_ok());
    assert_eq!(
        match json {
            Ok(HomieValue::JSON(data)) => data.get("test").unwrap().as_str().unwrap().to_owned(),
            _ => "".to_owned(),
        },
        "test"
    );
}

#[test]
fn test_json_nok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::JSON).build();
    let json = HomieValue::parse("{ \"test\": failure }", &desc);
    assert!(json.is_err());
}

#[test]
fn test_validation_float_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Float)
        .format(
            // Test cases for FloatRange
            HomiePropertyFormat::FloatRange(FloatRange {
                min: Some(-6.0),
                max: Some(6.0),
                step: Some(3.0),
            }),
        )
        .build();

    let float_values = vec![
        "-12", "-7.5", "-7.1", "-6.4", "-6.1", "-6.0", "-5.9", "-4.5", "-0.5", "0.0", "1.0", "2.0", "2.5", "4.7",
        "6.0", "6.5", "7.2", "8.0",
    ];

    println!("FloatRange Format: {:?}", &desc.format);
    println!("Value | Rounded Value | Result");
    println!("------|---------------|-------");
    for value in float_values {
        match HomieValue::parse(value, &desc) {
            Ok(rounded) => println!("{}   | {:?}          | Success", value, rounded),
            Err(err) => println!("{}   |               | {:?}", value, err),
        }
    }
}

#[test]
fn test_validation_integer_ok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Integer)
        .format(
            // Test cases for IntegerRange
            HomiePropertyFormat::IntegerRange(IntegerRange {
                min: None,
                max: Some(10),
                step: Some(2),
            }),
        )
        .build();

    let _float_values = vec![
        "-12", "-7", "-6", "-5", "-4", "-2", "-1", "0", "1", "2", "3", "4", "6", "7", "8", "10", "12", "15", "16",
        "17", "20",
    ];

    println!("Integer Format: {:?}", &desc.format);
    println!("Value | Rounded Value | Result");
    println!("------|---------------|-------");
    for value in -10..30 {
        match HomieValue::parse(&value.to_string(), &desc) {
            Ok(rounded) => println!("{}   | {:?}          | Success", value, rounded),
            Err(err) => println!("{}   |               | {:?}", value, err),
        }
    }
}
#[test]
fn test_validation_integer_nok() {
    let desc = PropertyDescriptionBuilder::new(HomieDataType::Integer).build();
    let json = HomieValue::parse("{ \"test\": failure }", &desc);
    assert!(json.is_err());
}
