mod common;

use common::{run_homietests, HomieTest};
use homie5::device_description::HomiePropertyDescription;

#[test]
fn test_homie_formats_boolean() {
    let result = run_homietests("homie5/formats/boolean.yml", |test_definition| {
        if let HomieTest::PropertyDescription(test) = test_definition {
            Ok(serde_yaml::from_value::<HomiePropertyDescription>(test.definition.clone()).is_ok())
        } else {
            Ok(test_definition.valid())
        }
    });

    assert!(result.is_ok(), "{:?}", result);
}
