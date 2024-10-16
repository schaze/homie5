mod common;
use common::{run_homietests, HomieTest};
use homie5::*;

#[test]
fn test_homie_id_from_file() {
    let result = run_homietests("../homie-testsuite/homie5/values/id.yml", |test_definition| {
        if let HomieTest::HomieID(test) = test_definition {
            Ok(HomieID::try_from(test.input_data.clone()).is_ok())
        } else {
            Err(anyhow::anyhow!("Invalid Testdefinition in test file"))
        }
    });

    assert!(result.is_ok(), "{:?}", result);
}
