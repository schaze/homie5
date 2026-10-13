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
            let Ok(homie_value) = HomieValue::parse(&test.input_data, &test.definition) else {
                return Ok(false);
            };
            let HomieValue::Integer(value) = homie_value else {
                return Err(anyhow::anyhow!("Invalid Testdefinition in test file"));
            };
            Ok(Some(value) == test.output_data)
        } else {
            Err(anyhow::anyhow!("Invalid Testdefinition in test file"))
        }
    });

    assert!(result.is_ok(), "{:?}", result);
}

#[test]
fn test_homie_float_value_from_file() {
    let result = run_homietests("homie5/values/float.yml", |test_definition| {
        if let HomieTest::PropertyValueFloat(test) = test_definition {
            let Ok(homie_value) = HomieValue::parse(&test.input_data, &test.definition) else {
                return Ok(false);
            };
            let HomieValue::Float(value) = homie_value else {
                return Err(anyhow::anyhow!("Invalid Testdefinition in test file"));
            };
            Ok(Some(value) == test.output_data)
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

fn assert_invalid_color_format(input: &str, expected_reason: &str) {
    match input.parse::<HomieColorValue>().unwrap_err() {
        Homie5ValueConversionError::InvalidColorFormat { value, reason } => {
            assert_eq!(value, input);
            assert!(
                reason.contains(expected_reason),
                "reason {reason:?} did not contain {expected_reason:?}"
            );
        }
        err => panic!("expected InvalidColorFormat, got {err:?}"),
    }
}

fn assert_invalid_color_value(input: &str, expected_value: HomieColorValue, expected_reason: &str) {
    match input.parse::<HomieColorValue>().unwrap_err() {
        Homie5ValueConversionError::InvalidColorValue { value, reason } => {
            assert_eq!(value, expected_value);
            assert!(
                reason.contains(expected_reason),
                "reason {reason:?} did not contain {expected_reason:?}"
            );
        }
        err => panic!("expected InvalidColorValue, got {err:?}"),
    }
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
    assert_invalid_color_format("", "expected color format");
    assert_invalid_color_format("invalid,255,100,50", "expected color format");
    assert_invalid_color_format("rgb,red,100,50", "RGB requires three finite numbers");
    assert_invalid_color_value(
        "rgb,300,100,50",
        HomieColorValue::RGB(300, 100, 50),
        "RGB requires three finite numbers",
    );
    assert_invalid_color_value(
        "hsv,120,101,50",
        HomieColorValue::HSV(120, 101, 50),
        "HSV requires three finite numbers",
    );
    assert_invalid_color_value(
        "xyz,0.9961,0.9961",
        HomieColorValue::XYZ(0.9961, 0.9961, 1.0 - 0.9961 - 0.9961),
        "XYZ requires numbers between 0.0 and 1.0",
    );
}

#[test]
fn test_homie_color_value_from_str_rejects_extra_components() {
    assert_invalid_color_format("rgb,255,100,50,0", "RGB requires three finite numbers");
    assert_invalid_color_format("hsv,120,100,100,0", "HSV requires three finite numbers");
    assert_invalid_color_format("xyz,0.3,0.4,0.3", "XYZ requires numbers between 0.0 and 1.0");
}

#[test]
fn test_homie_color_value_validate_rejects_invalid_direct_values() {
    assert!(HomieColorValue::RGB(256, 0, 0).validate().is_err());
    assert!(HomieColorValue::HSV(0, 101, 0).validate().is_err());

    let err = HomieColorValue::XYZ(0.3, 0.4, 0.4).validate().unwrap_err();
    assert!(matches!(
        err,
        Homie5ValueConversionError::InvalidColorValue {
            value: HomieColorValue::XYZ(0.3, 0.4, 0.4),
            reason
        } if reason.contains("z must equal 1.0 - x - y")
    ));
}

fn create_prop_desc(dt: HomieDataType, pf: HomiePropertyFormat) -> HomiePropertyDescription {
    HomiePropertyDescription {
        name: None,
        datatype: dt,
        format: pf,
        settable: false,
        retained: true,
        unit: None,
    }
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
    let desc = PropertyDescriptionBuilder::integer().name("test").build();
    assert!(matches!(HomieValue::parse("122", &desc), Ok(HomieValue::Integer(122))));
}

#[test]
fn test_integer_nok() {
    let desc = PropertyDescriptionBuilder::integer().name("test").build();
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
    let desc = PropertyDescriptionBuilder::float().build();
    assert!(matches!(HomieValue::parse("122", &desc), Ok(HomieValue::Float(122.0))));
    assert!(matches!(
        HomieValue::parse("122.12", &desc),
        Ok(HomieValue::Float(122.12))
    ));
}

#[test]
fn test_float_nok() {
    let desc = PropertyDescriptionBuilder::float().build();
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
    let desc = PropertyDescriptionBuilder::boolean().build();
    assert!(matches!(HomieValue::parse("true", &desc), Ok(HomieValue::Bool(true))));
    assert!(matches!(HomieValue::parse("false", &desc), Ok(HomieValue::Bool(false))));
}

#[test]
fn test_bool_nok() {
    let desc = PropertyDescriptionBuilder::boolean().build();
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
    let desc = PropertyDescriptionBuilder::string().build();
    assert_eq!(
        HomieValue::parse("blah", &desc).ok(),
        Some(HomieValue::String("blah".to_owned()))
    );
}

#[test]
fn test_enum_ok() {
    let desc = PropertyDescriptionBuilder::enumeration(["blah"]).unwrap().build();
    assert_eq!(
        HomieValue::parse("blah", &desc).ok(),
        Some(HomieValue::Enum("blah".to_owned()))
    );
}

#[test]
fn test_color_ok() {
    let desc = PropertyDescriptionBuilder::color([ColorFormat::Rgb, ColorFormat::Hsv, ColorFormat::Xyz])
        .unwrap()
        .build();
    assert_eq!(
        HomieValue::parse("rgb,12,55,14", &desc).ok(),
        Some(HomieValue::Color(HomieColorValue::RGB(12, 55, 14)))
    );
    assert_eq!(
        HomieValue::parse("hsv,112,85,55", &desc).ok(),
        Some(HomieValue::Color(HomieColorValue::HSV(112, 85, 55)))
    );
    assert_eq!(
        HomieValue::parse("xyz,0.33453,0.123456", &desc).ok(),
        Some(HomieValue::Color(HomieColorValue::XYZ(
            0.33453,
            0.123456,
            1.0 - 0.33453 - 0.123456
        )))
    );
}

#[test]
fn test_color_nok() {
    let desc = PropertyDescriptionBuilder::color([ColorFormat::Rgb]).unwrap().build();
    assert!(HomieValue::parse("rgb,12,55", &desc).is_err());
    assert!(HomieValue::parse("hsv,12,55,14", &desc).is_err());
    assert!(HomieValue::parse("HSV,12,55,14", &desc).is_err());
    assert!(HomieValue::parse("rgb ,12,55,14", &desc).is_err());
    assert!(HomieValue::parse("xyz/12,55", &desc).is_err());
}

#[test]
fn test_color_validate_rejects_invalid_direct_value() {
    let desc = PropertyDescriptionBuilder::color([ColorFormat::Rgb]).unwrap().build();

    assert!(!HomieValue::Color(HomieColorValue::RGB(256, 0, 0)).validate(&desc));
}

#[test]
fn test_color_without_format_is_invalid_description() {
    let desc = HomiePropertyDescription {
        name: None,
        datatype: HomieDataType::Color,
        format: HomiePropertyFormat::Empty,
        settable: false,
        retained: true,
        unit: None,
    };
    assert!(matches!(
        desc.validate_strict(),
        Err(PropertyDescriptionValidationError::MissingColorFormat)
    ));
}

#[test]
fn test_enum_without_format_is_invalid_description() {
    let desc = HomiePropertyDescription {
        name: None,
        datatype: HomieDataType::Enum,
        format: HomiePropertyFormat::Empty,
        settable: false,
        retained: true,
        unit: None,
    };
    assert!(matches!(
        desc.validate_strict(),
        Err(PropertyDescriptionValidationError::MissingEnumFormat)
    ));
}

#[test]
fn test_parse_enum_rejects_missing_format() {
    let desc = HomiePropertyDescription {
        name: None,
        datatype: HomieDataType::Enum,
        format: HomiePropertyFormat::Empty,
        settable: false,
        retained: true,
        unit: None,
    };

    assert!(HomieValue::parse("option1", &desc).is_err());
}

#[test]
fn test_parse_color_rejects_missing_format() {
    let desc = HomiePropertyDescription {
        name: None,
        datatype: HomieDataType::Color,
        format: HomiePropertyFormat::Empty,
        settable: false,
        retained: true,
        unit: None,
    };

    assert!(HomieValue::parse("rgb,12,55,14", &desc).is_err());
}

#[test]
fn test_datetime_ok() {
    let desc = PropertyDescriptionBuilder::datetime().build();
    assert_eq!(
        HomieValue::parse("2023-09-26T10:54:59+00:00", &desc).ok(),
        Some(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("2023-09-26T11:54:59+01:00", &desc).ok(),
        Some(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("2023-09-26T10:54:59Z", &desc).ok(),
        Some(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("2023-09-26T10:54:59", &desc).ok(),
        Some(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 0).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("2023-09-26T10:54:59.100", &desc).ok(),
        Some(HomieValue::DateTime(
            chrono::DateTime::<chrono::Utc>::from_timestamp(1695725699, 100000000).unwrap()
        ))
    );
}
#[test]
fn test_duration_ok() {
    let desc = PropertyDescriptionBuilder::duration().build();
    assert_eq!(
        HomieValue::parse("PT12H4M2S", &desc).ok(),
        Some(HomieValue::Duration(
            chrono::Duration::from_std(std::time::Duration::from_secs(12 * 60 * 60 + 4 * 60 + 2)).unwrap()
        ))
    );
    assert_eq!(
        HomieValue::parse("PT43442S", &desc).ok(),
        Some(HomieValue::Duration(
            chrono::Duration::from_std(std::time::Duration::from_secs(12 * 60 * 60 + 4 * 60 + 2)).unwrap()
        ))
    );
}

#[test]
fn test_json_ok() {
    let desc = PropertyDescriptionBuilder::json().build();
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
    let desc = PropertyDescriptionBuilder::json().build();
    let json = HomieValue::parse("{ \"test\": failure }", &desc);
    assert!(json.is_err());
}

#[test]
fn test_validation_float_ok() {
    let desc = PropertyDescriptionBuilder::float()
        .float_range(FloatRange {
            min: Some(-6.0),
            max: Some(6.0),
            step: Some(3.0),
        })
        .build();

    let float_values = vec![
        "-12", "-7.5", "-7.1", "-6.4", "-6.1", "-6.0", "-5.9", "-4.5", "-0.5", "0.0", "1.0", "2.0", "2.5", "4.7",
        "6.0", "6.5", "7.2", "8.0",
    ];

    println!("FloatRange Format: {:?}", desc.format);
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
    let desc = PropertyDescriptionBuilder::integer()
        .integer_range(IntegerRange {
            min: None,
            max: Some(10),
            step: Some(2),
        })
        .build();

    let _float_values = vec![
        "-12", "-7", "-6", "-5", "-4", "-2", "-1", "0", "1", "2", "3", "4", "6", "7", "8", "10", "12", "15", "16",
        "17", "20",
    ];

    println!("Integer Format: {:?}", desc.format);
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
    let desc = PropertyDescriptionBuilder::integer().build();
    let json = HomieValue::parse("{ \"test\": failure }", &desc);
    assert!(json.is_err());
}

// ---------------------------------------------------------------------------
// Regression tests: spec-compliance and robustness fixes
// ---------------------------------------------------------------------------

#[test]
fn test_color_parse_accepts_float_components_rounded() {
    // Per Homie v5 the rgb/hsv payload numbers are floats; components are rounded
    // to the nearest whole number for storage.
    assert_eq!(
        "rgb,100.5,50.4,50".parse::<HomieColorValue>().unwrap(),
        HomieColorValue::RGB(101, 50, 50)
    );
    assert_eq!(
        "hsv,300.4,50.5,75".parse::<HomieColorValue>().unwrap(),
        HomieColorValue::HSV(300, 51, 75)
    );
    // Rounding may not push a component out of range
    assert!("rgb,255.6,0,0".parse::<HomieColorValue>().is_err());
}

#[test]
fn test_color_parse_rejects_non_finite_components() {
    assert!("rgb,NaN,0,0".parse::<HomieColorValue>().is_err());
    assert!("rgb,inf,0,0".parse::<HomieColorValue>().is_err());
    assert!("hsv,120,-inf,50".parse::<HomieColorValue>().is_err());
    assert!("xyz,NaN,0.4".parse::<HomieColorValue>().is_err());
}

#[test]
fn test_float_parse_rejects_non_finite_payloads() {
    // The convention: "Representations of numeric concepts such as NaN and Infinity
    // are not a valid payload"
    let desc = PropertyDescriptionBuilder::float().build();
    assert!(HomieValue::parse("NaN", &desc).is_err());
    assert!(HomieValue::parse("inf", &desc).is_err());
    assert!(HomieValue::parse("-inf", &desc).is_err());
    assert!(HomieValue::parse("infinity", &desc).is_err());

    let desc_min_only = PropertyDescriptionBuilder::float().range(0.0..).build();
    assert!(HomieValue::parse("inf", &desc_min_only).is_err());
}

#[test]
fn test_float_step_keeps_values_already_on_grid() {
    // 0.3 lies exactly on the 0:1:0.1 grid; parse must not mutate it into
    // 0.30000000000000004 and validate must accept it.
    let desc = PropertyDescriptionBuilder::float()
        .float_range(FloatRange {
            min: Some(0.0),
            max: Some(1.0),
            step: Some(0.1),
        })
        .build();
    assert_eq!(HomieValue::parse("0.3", &desc).unwrap(), HomieValue::Float(0.3));
    assert_eq!(HomieValue::parse("0.7", &desc).unwrap(), HomieValue::Float(0.7));
    assert!(HomieValue::Float(0.3).validate(&desc));
    assert!(HomieValue::Float(0.7).validate(&desc));
}

#[test]
fn test_float_step_rounding_follows_convention() {
    // Examples from the convention's step-rounding table
    let desc = PropertyDescriptionBuilder::float()
        .float_range(FloatRange {
            min: Some(0.0),
            max: Some(10.0),
            step: Some(2.0),
        })
        .build();
    assert_eq!(HomieValue::parse("5", &desc).unwrap(), HomieValue::Float(6.0));

    let desc_max_base = PropertyDescriptionBuilder::float()
        .float_range(FloatRange {
            min: None,
            max: Some(10.0),
            step: Some(2.0),
        })
        .build();
    assert_eq!(HomieValue::parse("5", &desc_max_base).unwrap(), HomieValue::Float(6.0));
}

#[test]
fn test_integer_step_preserves_large_values() {
    // Values above 2^53 must not be corrupted by float conversions.
    let desc = PropertyDescriptionBuilder::integer()
        .integer_range(IntegerRange {
            min: Some(0),
            max: None,
            step: Some(1),
        })
        .build();
    let big = (1i64 << 62) + 1;
    assert_eq!(
        HomieValue::parse(&big.to_string(), &desc).unwrap(),
        HomieValue::Integer(big)
    );
    assert!(HomieValue::Integer(big).validate(&desc));
}

#[test]
fn test_integer_step_no_overflow_at_extremes() {
    // base = i64::MIN, value = i64::MAX must not panic on `value - base`.
    let desc = PropertyDescriptionBuilder::integer()
        .integer_range(IntegerRange {
            min: Some(i64::MIN),
            max: Some(i64::MAX),
            step: Some(1),
        })
        .build();
    assert_eq!(
        HomieValue::parse(&i64::MAX.to_string(), &desc).unwrap(),
        HomieValue::Integer(i64::MAX)
    );
    assert_eq!(
        HomieValue::parse(&i64::MIN.to_string(), &desc).unwrap(),
        HomieValue::Integer(i64::MIN)
    );
}

#[test]
fn test_integer_step_rounding_follows_convention() {
    // Examples from the convention's step-rounding table
    let desc = PropertyDescriptionBuilder::integer()
        .integer_range(IntegerRange {
            min: Some(0),
            max: Some(10),
            step: Some(2),
        })
        .build();
    assert_eq!(HomieValue::parse("5", &desc).unwrap(), HomieValue::Integer(6));

    let desc_max_base = PropertyDescriptionBuilder::integer()
        .integer_range(IntegerRange {
            min: None,
            max: Some(10),
            step: Some(2),
        })
        .build();
    assert_eq!(HomieValue::parse("5", &desc_max_base).unwrap(), HomieValue::Integer(6));
}

#[test]
fn test_parse_duration_rejects_overflow_without_panic() {
    // i64 overflow in a single component
    assert!(HomieValue::parse_duration("PT99999999999999999999H").is_err());
    // arithmetic overflow when combining components
    assert!(HomieValue::parse_duration("PT9223372036854775807H").is_err());
    // out of range for chrono::Duration
    assert!(HomieValue::parse_duration("PT9000000000000000000S").is_err());
}

#[test]
fn test_duration_display_is_spec_compliant() {
    // PTxHxMxS output that round-trips through parse_duration
    let cases = [
        (Duration::seconds(43546), "PT12H5M46S"),
        (Duration::seconds(300), "PT5M"),
        (Duration::seconds(3600), "PT1H"),
        (Duration::seconds(0), "PT0S"),
        // negative durations clamp to PT0S (not expressible in the convention format)
        (Duration::seconds(-5), "PT0S"),
        // sub-second parts are truncated
        (Duration::milliseconds(1500), "PT1S"),
    ];
    for (duration, expected) in cases {
        let displayed = HomieValue::Duration(duration).to_string();
        assert_eq!(displayed, expected, "for duration {duration}");
        assert!(
            HomieValue::parse_duration(&displayed).is_ok(),
            "displayed duration {displayed:?} must be parseable"
        );
    }
}

#[test]
fn test_mqtt_payload_to_string_strict_empty_encoding() {
    // Only the exact single-byte 0x00 payload is the Homie empty-string encoding.
    assert_eq!(client::mqtt_payload_to_string(&[0]).unwrap(), "");
    assert_eq!(client::mqtt_payload_to_string(b"abc").unwrap(), "abc");
    // A longer payload starting with 0x00 must not be silently truncated to "".
    assert_eq!(client::mqtt_payload_to_string(&[0, b'x']).unwrap(), "\0x");
    // Round-trip with the encoder
    assert_eq!(client::mqtt_payload_to_string(&homie_str_to_vecu8("")).unwrap(), "");
}

#[test]
fn test_partial_ord_consistent_with_eq_across_types() {
    let a = HomieValue::Enum("a".into());
    let b = HomieValue::String("a".into());
    assert_ne!(a, b);
    assert_eq!(a.partial_cmp(&b), None);
    assert_eq!(b.partial_cmp(&a), None);
    assert!(HomieValue::Enum("a".into()) < HomieValue::Enum("b".into()));
}

#[test]
fn test_duration_validate_rejects_unrepresentable_values() {
    let desc = PropertyDescriptionBuilder::duration().build();
    assert!(HomieValue::Duration(Duration::seconds(90)).validate(&desc));
    assert!(HomieValue::Duration(Duration::zero()).validate(&desc));
    assert!(!HomieValue::Duration(Duration::seconds(-1)).validate(&desc));
    assert!(!HomieValue::Duration(Duration::milliseconds(1500)).validate(&desc));
}

#[test]
fn test_datetime_fallback_formats() {
    let desc = PropertyDescriptionBuilder::datetime().build();
    let expected = chrono::NaiveDate::from_ymd_opt(2024, 10, 8)
        .unwrap()
        .and_hms_opt(10, 15, 0)
        .unwrap()
        .and_utc();
    for raw in [
        "2024-10-08T10:15:00Z",
        "2024-10-08T10:15:00",
        "2024-10-08T10:15:00.000",
        "2024-10-08T10:15",
        "2024-10-08T10:15Z",
    ] {
        assert_eq!(
            HomieValue::parse(raw, &desc).ok(),
            Some(HomieValue::DateTime(expected)),
            "{raw}"
        );
    }
    assert!(HomieValue::parse("2024-10-08", &desc).is_err());
    assert!(HomieValue::parse("not a date", &desc).is_err());
}
