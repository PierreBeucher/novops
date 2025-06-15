
/// Simple wrapper around BitWarden CLI
/// Keep it simple as we intend to remove/deprecated Bitwarden usage in the future
use std::process::{Command, Output};
use std;
use serde_json;
use std::option::Option;
use serde::Deserialize;
use async_trait::async_trait;
use anyhow::{Context, anyhow};
use schemars::JsonSchema;

use crate::core;


/// A BitWarden secret reference 
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct BitwardenItemInput {
    bitwarden: BitwardenEntry,
}

#[async_trait]
impl core::ResolveTo<String> for BitwardenItemInput {
    async fn resolve(&self, ctx: &core::NovopsContext) -> Result<String, anyhow::Error> {

        if ctx.dry_run {
          return Ok(format!("RESULT:{:}.{:}", self.bitwarden.entry, self.bitwarden.field));
        }

        let json_result = get_item(&self.bitwarden.entry);

        let json_value = match json_result {
            Ok(v) => v,
            Err(e) => return Err(e)
        };

        // Novops config let user specify a string like "login.password"
        // we need to retrieve this field nested in our JSON (or fail if not found)
        let fields = self.bitwarden.field.split('.').map(String::from).collect();
        return Ok(
          get_string_in_value(&json_value, fields)
          .with_context(|| format!("Error retrieving field '{:?}' in value {:?} for input {:?}", self.bitwarden.field, &json_value, &self))?
          .to_string()
        )
    }
}

/// A BitWarden entry
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct BitwardenEntry {
    /// Entry name
    entry: String,

    /// Field in entry to use as value.
    field: String
}


/// Wraps Bitwarden command context
#[derive(Debug)]
pub struct BitwardenCommandContext {
  pub output: Option<Output>,
  pub command: String
}


/// Retrieve a Bitwarden item as a JSON value
pub fn get_item(item: &String) -> Result<serde_json::Value, anyhow::Error> {
  let command_str = format!("bw get item '{}'", item);

  let mut command_context = BitwardenCommandContext{command: command_str, output: None};

  let output = Command::new("bw")
    .arg("get")
    .arg("item")
    .arg(item)
    .output()
    .with_context(|| format!("Error running Bitwarden command {:?}", command_context))?;

  command_context.output = Some(output.clone());

  let stdout = std::str::from_utf8(&output.stdout)
    .with_context(|| format!("Couldn't decode stdout as UTF-8 for command: {:?}", command_context))?;
  
  if ! output.status.success() {
    return Err(anyhow!("Bitwarden command returned non-0 exit code, stderr probably has more details. For command: {:?}", command_context));
  };

  let json: serde_json::Value = serde_json::from_str(stdout)
    .with_context(|| format!("Couldn't parse Bitwarden stdout as JSON. {:?}", command_context))?;


  Ok(json)

}


/// Get a string from a JSON Value
/// 
/// Example: considering JSON { "login": { "password": "secret", "username": "foo" }} 
/// get_string_in_value(myJson, ["item", "foo"]) ==> "bar"
/// This is a wrapper for Novops config where client provide a string like "login.password" for the desired Bitwarden entry
pub fn get_string_in_value(value: &serde_json::Value, mut fields: Vec<String>) -> Result<String, anyhow::Error>{
  let field = fields.remove(0);

  let found_value = value.get(&field)
    .with_context(|| format!("Couldn't find field '{:}' in value {:?}", field, value))?;
  
  if !fields.is_empty() {
    get_string_in_value(found_value, fields)
  } else {
    Ok(found_value.as_str().with_context(|| format!("Couldn't convert to string: {:?}", found_value))?.to_string())
  }
}