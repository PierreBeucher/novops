#[macro_use]
extern crate enum_dispatch;
mod novops;
mod bitwarden;

use novops::{NovopsConfig, NovopsEnvironment, ResolvableNovopsValue};
use std::io::Error;
use std::collections::HashMap;

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
    for (var_name, var_value) in &env_config.variables {
        variable_map.insert(var_name.clone(), var_value.resolve());
    }

    let mut file_content_map: HashMap<String, String> = HashMap::new();
    for file_entry in env_config.files.values() {
        file_content_map.insert(file_entry.dest.clone(), file_entry.content.resolve());
    }

    return (variable_map, file_content_map)
}
