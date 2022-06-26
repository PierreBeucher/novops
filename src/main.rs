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

trait ResolvableNovopsValue {
    fn resolve(&self) -> String;
}

impl ResolvableNovopsValue for NovopsValue {
    fn resolve(&self) -> String {
        match self {
            NovopsValue::String(v) => return v.resolve(),
            NovopsValue::StringValue(v) => return v.resolve(),
            NovopsValue::BitwardenItem(v) => return v.resolve()
        }
    }
}

impl ResolvableNovopsValue for String {
    fn resolve(&self) -> String {
        return self.clone()
    }
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

impl ResolvableNovopsValue for StringValue {
    fn resolve(&self) -> String {
        return self.value.clone();
    }
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

impl ResolvableNovopsValue for BitwardenItem {
    fn resolve(&self) -> String {
        // Dummy implementation for now
        return format!("{}:{}", self.bitwarden.entry, self.bitwarden.field);
    }
}


#[derive(Debug, Deserialize)]
 struct BitwardenValue {
    entry: String,
    field: String
}


fn main() -> Result<(), Error> {

    let config = read_config(".novops.yml").unwrap();
    println!("Found config: {:?}", config);

    let (variable_map, file_content_map) = parse_config(&config, String::from("dev"));

    println!("Resolved vars: {:?}", variable_map);
    println!("Resolved files: {:?}", file_content_map);

    Ok(())
}


fn read_config(config_path: &str) -> Result<NovopsConfig, serde_yaml::Error> {
    let f = std::fs::File::open(config_path).unwrap();
    let config: NovopsConfig = serde_yaml::from_reader(f).unwrap();

    return Ok(config);
}

/**
 * Parse configuration into a concrete tuple of variable and files
 */
fn parse_config(config: &NovopsConfig, env_name: String) -> (HashMap<String, String>, HashMap<String, String>){
    let env_config: &NovopsEnvironment = &config.environments[&env_name];

    let mut variable_map: HashMap<String, String> = HashMap::new();
    for (var_name, var_abstract_value) in &env_config.variables {
        variable_map.insert(var_name.clone(), var_abstract_value.resolve());
    }

    let mut file_content_map: HashMap<String, String> = HashMap::new();
    for file_entry in env_config.files.values() {
        file_content_map.insert(file_entry.dest.clone(), file_entry.content.resolve());
    }

    return (variable_map, file_content_map)
}
