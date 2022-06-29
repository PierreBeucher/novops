/**
 * Simple wrapper around BitWarden CLI
 * Keep it simple as we intend to remove/deprecated Bitwarden usage in the future
*/

use std::{process::{Command, Output}, fmt};
use std::error::Error;
use serde_json;
use std::option::Option;
use serde::Deserialize;
use async_trait::async_trait;

use crate::novops;

/**
 * A BitWarden secret such as
 * 
 * myvar:
 *   bitwarden:
 *     entry: wordpress_prod
 *     field: login.password
 */

#[derive(Debug, Deserialize, Clone)]
pub struct BitwardenItemInput {
    bitwarden: BitwardenEntry,
}

#[async_trait]
impl novops::ResolveTo<String> for BitwardenItemInput {
    async fn resolve(&self, _: &novops::NovopsContext) -> String {
        let json_value = get_item(&self.bitwarden.entry).expect(&String::from("Error fetching Bitwarden entry"));

        // Novops config let use specify a string like "login.password"
        // we need to retrieve this field nexted in our JSON (or fail if not found)
        let fields = self.bitwarden.field.split(".").map(|s| String::from(s)).collect();
        let val = get_string_in_value(&json_value, fields);
        
        return val.expect("Couldn't get value from Bitwarden entry").to_string();
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BitwardenEntry {
    entry: String,
    field: String
}

/**
 * Error wrapping Bitwarden CLI errors using Command module
 */
pub struct CommandError {
  pub message: String,
  pub output: Output,
  pub error: Option<Box<dyn Error>>,
  pub command: String
}

impl fmt::Display for CommandError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}\nCommand: {:?}\nOutput: {:?}", self.message, self.command, self.output)
  }
}

impl fmt::Debug for CommandError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}\nCommand: {:?}\nOutput: {:?}", self.message, self.command, self.output)
  }
}

/**
 * Retrieve a Bitwarden item as a JSON value
 */
pub fn get_item(item: &String) -> Result<serde_json::Value, CommandError> {
  let command_str = format!("bw get item {}", item);

  let output = Command::new("bw")
    .arg("get")
    .arg("item")
    .arg(item)
    .output()
    .expect("Error running bw command...");

  let stdout = match std::str::from_utf8(&output.stdout) {
      Ok(s) => s,
      Err(e) => {
        return Err(CommandError {
          message: String::from("Couldn't decode stdout as UTF-8"),
          output: output,
          error: Some(Box::new(e)),
          command: command_str
        });
      }
  };

  match std::str::from_utf8(&output.stderr) {
    Ok(_) => {},
    Err(e) => {
      return Err(CommandError {
        message: String::from("Couldn't decode stderr as UTF-8"),
        output: output,
        error: Some(Box::new(e)),
        command: command_str
      });
    }
  };

  if ! output.status.success() {
    return Err(CommandError {
      message: String::from("Bitwarden CLI returned non-0 exit code"),
      output: output,
      error: None,
      command: command_str
    });
  }

  let json: serde_json::Value = match serde_json::from_str(stdout) {
    Ok(json) => json,
    Err(e) => {
      return Err(CommandError {
        message: String::from("Couldn't parse Bitwarden stdout as JSON"),
        output: output,
        error: Some(Box::new(e)),
        command: command_str
      });
    },
  };

  return Ok(json);

}

pub struct ValueError {
  pub message: String,
  pub field: String,
  pub value: serde_json::Value
}

impl fmt::Display for ValueError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}\nField: {:?}\nValue: {:?}", self.message, self.field, self.value)
  }
}

impl fmt::Debug for ValueError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}\nField: {:?}\nValue: {:?}", self.message, self.field, self.value)
  }
}

/**
 * Get a string from a JSON Value
 * 
 * Example: considering JSON { "login": { "password": "secret", "username": "foo" }} 
 * get_string_in_value(myJson, ["item", "foo"]) ==> "bar"
 * This is a wrapper for Novops config where client provide a string like "login.password" for the desired Bitwarden entry
 */
pub fn get_string_in_value(value: &serde_json::Value, mut fields: Vec<String>) -> Result<&str, ValueError>{
  let field = fields.remove(0);

  let found_value = match value.get(&field) {
    Some(v) => v,
    None => {
      return Err(ValueError {
        message: String::from("Couldn't find field on value"),
        field: field,
        value: value.clone()
      });
    }
  };

  if fields.len() > 0 {
    return get_string_in_value(found_value, fields);
  } else {
    let result = match found_value.as_str() {
      Some(s) => s,
      None => { 
        return Err(ValueError{
          message: String::from("Value must be a string for field"),
          field: field,
          value: value.clone()
        })
      }
     };
    return Ok(result);
  }
}