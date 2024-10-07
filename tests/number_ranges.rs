use homie5::device_description::*;

#[test]
fn test_float_range_parse_normal_cases() {
    // Normal case with min, max, and step
    let range = FloatRange::parse("1.5:10.5:0.5").unwrap();
    assert_eq!(range.min, Some(1.5));
    assert_eq!(range.max, Some(10.5));
    assert_eq!(range.step, Some(0.5));

    // Case with only min and max
    let range = FloatRange::parse("2.0:20.0").unwrap();
    assert_eq!(range.min, Some(2.0));
    assert_eq!(range.max, Some(20.0));
    assert_eq!(range.step, None);

    // Open-ended max
    let range = FloatRange::parse("5.5:").unwrap();
    assert_eq!(range.min, Some(5.5));
    assert_eq!(range.max, None);
    assert_eq!(range.step, None);

    // Open-ended min
    let range = FloatRange::parse(":7.5").unwrap();
    assert_eq!(range.min, None);
    assert_eq!(range.max, Some(7.5));
    assert_eq!(range.step, None);

    // Open-ended min and max, with step
    let range = FloatRange::parse("::1.0").unwrap();
    assert_eq!(range.min, None);
    assert_eq!(range.max, None);
    assert_eq!(range.step, Some(1.0));
}

#[test]
fn test_float_range_parse_edge_cases() {
    // Min == Max
    let range = FloatRange::parse("5.0:5.0").unwrap();
    assert_eq!(range.min, Some(5.0));
    assert_eq!(range.max, Some(5.0));
    assert_eq!(range.step, None);

    // Step larger than range
    let range = FloatRange::parse("2.0:4.0:5.0").is_err();
    assert!(range);

    // Only step provided
    let range = FloatRange::parse("::2.5").unwrap();
    assert_eq!(range.min, None);
    assert_eq!(range.max, None);
    assert_eq!(range.step, Some(2.5));
}

#[test]
fn test_float_range_negative_cases() {
    // Invalid format
    let range = FloatRange::parse("invalid:range").is_err();
    assert!(range);

    // Min greater than Max
    let range = FloatRange::parse("10.0:5.0").is_err();
    assert!(range);

    // Step 0 or negative
    let range = FloatRange::parse("2.0:6.0:0").is_err();
    assert!(range);

    let range = FloatRange::parse("2.0:6.0:-1.0").is_err();
    assert!(range);
}

#[test]
fn test_integer_range_parse_normal_cases() {
    // Normal case with min, max, and step
    let range = IntegerRange::parse("1:10:2").unwrap();
    assert_eq!(range.min, Some(1));
    assert_eq!(range.max, Some(10));
    assert_eq!(range.step, Some(2));

    // Case with only min and max
    let range = IntegerRange::parse("2:20").unwrap();
    assert_eq!(range.min, Some(2));
    assert_eq!(range.max, Some(20));
    assert_eq!(range.step, None);

    // Open-ended max
    let range = IntegerRange::parse("5:").unwrap();
    assert_eq!(range.min, Some(5));
    assert_eq!(range.max, None);
    assert_eq!(range.step, None);

    // Open-ended min
    let range = IntegerRange::parse(":15").unwrap();
    assert_eq!(range.min, None);
    assert_eq!(range.max, Some(15));
    assert_eq!(range.step, None);

    // Open-ended min and max, with step
    let range = IntegerRange::parse("::3").unwrap();
    assert_eq!(range.min, None);
    assert_eq!(range.max, None);
    assert_eq!(range.step, Some(3));
}

#[test]
fn test_integer_range_parse_edge_cases() {
    // Min == Max
    let range = IntegerRange::parse("5:5").unwrap();
    assert_eq!(range.min, Some(5));
    assert_eq!(range.max, Some(5));
    assert_eq!(range.step, None);

    // Step larger than range
    let range = IntegerRange::parse("2:4:5").is_err();
    assert!(range);

    // Only step provided
    let range = IntegerRange::parse("::3").unwrap();
    assert_eq!(range.min, None);
    assert_eq!(range.max, None);
    assert_eq!(range.step, Some(3));
}

#[test]
fn test_integer_range_negative_cases() {
    // Invalid format
    let range = IntegerRange::parse("invalid:range").is_err();
    assert!(range);

    // Min greater than Max
    let range = IntegerRange::parse("10:5").is_err();
    assert!(range);

    // Step 0 or negative
    let range = IntegerRange::parse("2:6:0").is_err();
    assert!(range);

    let range = IntegerRange::parse("2:6:-1").is_err();
    assert!(range);
}

#[test]
fn test_float_range_display() {
    let range = FloatRange {
        min: Some(1.5),
        max: Some(10.5),
        step: Some(0.5),
    };
    assert_eq!(range.to_string(), "1.5:10.5:0.5");

    let range = FloatRange {
        min: Some(5.0),
        max: None,
        step: None,
    };
    assert_eq!(range.to_string(), "5:");

    let range = FloatRange {
        min: None,
        max: Some(8.0),
        step: None,
    };
    assert_eq!(range.to_string(), ":8");

    let range = FloatRange {
        min: None,
        max: None,
        step: Some(1.0),
    };
    assert_eq!(range.to_string(), "::1");
}

#[test]
fn test_integer_range_display() {
    let range = IntegerRange {
        min: Some(1),
        max: Some(10),
        step: Some(2),
    };
    assert_eq!(range.to_string(), "1:10:2");

    let range = IntegerRange {
        min: Some(5),
        max: None,
        step: None,
    };
    assert_eq!(range.to_string(), "5:");

    let range = IntegerRange {
        min: None,
        max: Some(15),
        step: None,
    };
    assert_eq!(range.to_string(), ":15");

    let range = IntegerRange {
        min: None,
        max: None,
        step: Some(3),
    };
    assert_eq!(range.to_string(), "::3");
}
