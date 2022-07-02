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
    async fn resolve(&self, _: &novops::NovopsContext) -> Result<String, Box<dyn Error>> {
        let json_result = get_item(&self.bitwarden.entry);

        let json_value = match json_result {
            Ok(v) => v,
            Err(e) => return Err(e.into())
        };

        // Novops config let user specify a string like "login.password"
        // we need to retrieve this field nexted in our JSON (or fail if not found)
        let fields = self.bitwarden.field.split(".").map(|s| String::from(s)).collect();
        let val = get_string_in_value(&json_value, fields);
        
        match val {
            Ok(s) => return Ok(s.to_string()),
            Err(e) => return Err(e.into()),
        };
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
#[derive(Debug)]
pub struct BitwardenCommandError {
  pub message: String,
  pub output: Output,
  pub error: Option<Box<dyn Error>>,
  pub command: String
}

impl Error for BitwardenCommandError {}

impl fmt::Display for BitwardenCommandError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Bitwarden CLI command errored ({:}): {:}\nCommand: {:}\nstdout: {:?}\nstderr: {:?}", 
      &self.output.status, &self.message, &self.command, 
      String::from_utf8(self.output.stdout.clone()).unwrap(), 
      String::from_utf8(self.output.stderr.clone()).unwrap())
  }
}

/**
 * Retrieve a Bitwarden item as a JSON value
 */
pub fn get_item(item: &String) -> Result<serde_json::Value, BitwardenCommandError> {
  let command_str = format!("bw get item '{}'", item);

  let output = Command::new("bw")
    .arg("get")
    .arg("item")
    .arg(item)
    .output()
    .expect("Error running bw command...");

  let stdout = match std::str::from_utf8(&output.stdout) {
      Ok(s) => s,
      Err(e) => {
        return Err(BitwardenCommandError {
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
      return Err(BitwardenCommandError {
        message: String::from("Couldn't decode stderr as UTF-8"),
        output: output,
        error: Some(Box::new(e)),
        command: command_str
      });
    }
  };

  if ! output.status.success() {
    return Err(BitwardenCommandError {
      message: String::from("Bitwarden CLI returned non-0 exit code, stderr probably has more details."),
      output: output,
      error: None,
      command: command_str
    });
  }

  let json: serde_json::Value = match serde_json::from_str(stdout) {
    Ok(json) => json,
    Err(e) => {
      return Err(BitwardenCommandError {
        message: String::from("Couldn't parse Bitwarden stdout as JSON"),
        output: output,
        error: Some(Box::new(e)),
        command: command_str
      });
    },
  };

  return Ok(json);

}

#[derive(Debug)]
pub struct BitwardenFieldValueError {
  pub message: String,
  pub field: String,
  pub value: serde_json::Value
}

impl Error for BitwardenFieldValueError{}

impl fmt::Display for BitwardenFieldValueError {
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
pub fn get_string_in_value(value: &serde_json::Value, mut fields: Vec<String>) -> Result<&str, BitwardenFieldValueError>{
  let field = fields.remove(0);

  let found_value = match value.get(&field) {
    Some(v) => v,
    None => {
      return Err(BitwardenFieldValueError {
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
        return Err(BitwardenFieldValueError{
          message: String::from("Value must be a string for field"),
          field: field,
          value: value.clone()
        })
      }
     };
    return Ok(result);
  }
}