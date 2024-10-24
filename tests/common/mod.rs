use std::{env, fs, path::Path};

use homie5::{
    device_description::{HomieDeviceDescription, HomiePropertyDescription},
    *,
};
use serde::{Deserialize, Serialize};

fn get_test_repo_path() -> String {
    env::var("TEST_REPO_PATH").unwrap_or_else(|_| "homie-testsuite".to_string())
}

#[allow(dead_code)]
pub struct Settings {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub homie_domain: String,
}

#[allow(dead_code)]
pub fn get_settings() -> Settings {
    let hostname = env::var("HOMIE_MQTT_HOST").unwrap_or_default();

    let port = if let Ok(port) = env::var("HOMIE_MQTT_PORT") {
        port.parse::<u16>().expect("Not a valid number for port!")
    } else {
        1883
    };

    let username = env::var("HOMIE_MQTT_USERNAME").unwrap_or_default();

    let password = env::var("HOMIE_MQTT_PASSWORD").unwrap_or_default();

    let client_id = if let Ok(client_id) = env::var("HOMIE_MQTT_CLIENT_ID") {
        client_id
    } else {
        String::from("aslkdnlauidhwwkednwek")
    };
    let homie_domain = if let Ok(homie_domain) = env::var("HOMIE_MQTT_DOMAIN") {
        homie_domain
    } else {
        String::from(DEFAULT_HOMIE_DOMAIN)
    };

    Settings {
        hostname,
        port,
        username,
        password,
        client_id,
        homie_domain,
    }
}

#[allow(dead_code)]
pub struct Device {
    pub ident: DeviceRef,
    pub state: HomieDeviceStatus,
    pub description: Option<HomieDeviceDescription>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HomieTestDefinition<DEFINITION, INPUTDATA, OUTPUTDATA> {
    pub description: String,
    pub definition: DEFINITION,
    pub input_data: INPUTDATA,
    pub output_data: OUTPUTDATA,
    pub valid: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "testtype", rename_all = "lowercase")]
pub enum HomieTest {
    PropertyDescription(HomieTestDefinition<serde_yaml::Value, Option<()>, Option<()>>),
    PropertyValue(HomieTestDefinition<HomiePropertyDescription, String, Option<()>>),
    PropertyValueInteger(HomieTestDefinition<HomiePropertyDescription, String, Option<i64>>),
    HomieID(HomieTestDefinition<Option<()>, String, Option<()>>),
}

#[allow(dead_code)]
impl HomieTest {
    pub fn description(&self) -> &str {
        match self {
            HomieTest::PropertyDescription(homie_test_definition) => &homie_test_definition.description,
            HomieTest::PropertyValue(homie_test_definition) => &homie_test_definition.description,
            HomieTest::PropertyValueInteger(homie_test_definition) => &homie_test_definition.description,
            HomieTest::HomieID(homie_test_definition) => &homie_test_definition.description,
        }
    }
    pub fn valid(&self) -> bool {
        match self {
            HomieTest::PropertyDescription(homie_test_definition) => homie_test_definition.valid,
            HomieTest::PropertyValue(homie_test_definition) => homie_test_definition.valid,
            HomieTest::PropertyValueInteger(homie_test_definition) => homie_test_definition.valid,
            HomieTest::HomieID(homie_test_definition) => homie_test_definition.valid,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HomieTestSet {
    pub description: String,
    pub tests: Vec<HomieTest>,
}

pub fn load_test_set_from_file(filename: &str) -> anyhow::Result<HomieTestSet> {
    // get the test repo folder from the environment
    let test_repo_path = get_test_repo_path();
    let test_file_path = Path::new(&test_repo_path).join(filename);

    // Read the file
    let contents = fs::read_to_string(test_file_path)?;

    // Deserialize the contents into the Config struct
    let test_set: HomieTestSet = serde_yaml::from_str(&contents)?;

    Ok(test_set)
}

pub fn run_homietests(filename: &str, result_fn: impl Fn(&HomieTest) -> anyhow::Result<bool>) -> anyhow::Result<()> {
    let tests = load_test_set_from_file(filename)?;
    for test_definition in tests.tests {
        let result = result_fn(&test_definition)?;
        assert_eq!(
            result,
            test_definition.valid(),
            "[{}] - Failed test: [{}]",
            tests.description,
            test_definition.description()
        );
    }

    Ok(())
}
