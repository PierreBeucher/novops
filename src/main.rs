use std::io::Error;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NovopsConfig {
    environments: HashMap<String, NovopsEnvironment>
}

#[derive(Debug, Deserialize)]
struct NovopsEnvironment {
    variables: HashMap<String, NovopsValue>,
    files: HashMap<String, NovopsFile>
}

#[derive(Debug, Deserialize)]
struct NovopsFile {
    dest: String,
    content: NovopsValue
}

/**
 * A Novops value is it's core: in can be a string, a secret, or something else
 */
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NovopsValue {
    String(String), 
    StringValue(StringValue),
    BitwardenItem(BitwardenItem)
}

/**
 * A string set with 'value' key such as 
 * 
 * myvar:
 *   value: foo
 */

#[derive(Debug, Deserialize)]
struct StringValue{
    value: String
}

/**
 * A BitWarden secret such as
 * 
 * myvar:
 *   bitwarden:
 *     entry: wordpress_prod
 *     field: login.password
 */

#[derive(Debug, Deserialize)]
 struct BitwardenItem {
    bitwarden: BitwardenValue,
}

#[derive(Debug, Deserialize)]
 struct BitwardenValue {
    entry: String,
    field: String
}


fn main() -> Result<(), Error> {

    let config = read_config(".novops.yml");
    println!("Found config: {:?}", config);
    Ok(())
}


fn read_config(config_path: &str) -> Result<NovopsConfig, serde_yaml::Error> {
    let f = std::fs::File::open(config_path).unwrap();
    let config: NovopsConfig = serde_yaml::from_reader(f).unwrap();

    return Ok(config);

    // match &config.environments["dev"].variables["AWS_DEFAULT_REGION"] {
    //     NovopsValue::String(v) => println!("Value: {}", v),
    //     NovopsValue::StringValue(v) => println!("Value: {}", v.value),
    //     NovopsValue::BitwardenItem(v) => println!("BitWarden: {}", v.bitwarden.entry)
    // }
}