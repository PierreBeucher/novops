use std::process::Command;
use std::option::Option;
use log::debug;
use serde::Deserialize;
use async_trait::async_trait;
use anyhow::{Context, anyhow};
use schemars::JsonSchema;

use crate::{core, modules::variables::VariableOutput};

/**
 * SOPS input to be used as file, variables or other kind of value input
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct SopsValueInput {
    sops: SopsValueFromFile
}

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct SopsValueFromFile {
      
    /**
    * Path to encrypted file
    */
    file: String,

    /**
    * Extract a specific field via --extract flag
    */
    extract: Option<String>,

    /**
    * Additional flags passed to sops
    * after --decrypt --extract
    */
    additional_flags: Option<Vec<String>>,
}

/**
 * SOPS input directly under an environment
 * to load file content as environment variables
 * Encrypted SOPS files must be in a valid dotenv format
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct SopsDotenvInput {

    /**
     * Path to encrypted file
     */
    file: String,

    /**
     * Additional flags passed to sops
     */
    additional_flags: Option<Vec<String>>,

    /**
    * Extract a specific field via --extract flag
    */
    extract: Option<String>,
}

#[async_trait]
impl core::ResolveTo<String> for SopsValueInput {
    async fn resolve(&self, ctx: &core::NovopsContext) -> Result<String, anyhow::Error> {

        if ctx.dry_run {          
          return Ok(format!("RESULT:{:}:{:}", &self.sops.file, &self.sops.extract.clone().unwrap_or(String::from(""))));
        }

        let mut args = vec![];
        
        // add --extract flag if specidief in input
        if let Some(e) = self.sops.extract.clone() {
          args.push(String::from("--extract"));
          args.push(e);
        }

        // Add additional flags if any
        if let Some(af) = self.sops.additional_flags.clone() { args.extend(af); }
        
        let output = run_sops_decrypt(args, &self.sops.file).with_context(|| "Error running sops command.")?;

        Ok(output.to_string())

    }
}

#[async_trait]
impl core::ResolveTo<Vec<VariableOutput>> for SopsDotenvInput {
    async fn resolve(&self, ctx: &core::NovopsContext) -> Result<Vec<VariableOutput>, anyhow::Error> {
        
        if ctx.dry_run {          
          return Ok(vec![VariableOutput {
            name: String::from("RESULT"),
            value: format!("{}:{}", &self.file, &self.additional_flags.clone().unwrap_or_default().join("-"))
          }]);
        }

        let mut args = vec![
          String::from("--output-type"),
          String::from("dotenv")
        ];

        // add --extract flag if specidief in input
        if let Some(e) = self.extract.clone() {
          args.push(String::from("--extract"));
          args.push(e);
        }

        // Add additional flags if any
        if let Some(af) = self.additional_flags.clone() { args.extend(af); }

        let output = run_sops_decrypt(args, &self.file).with_context(|| "Error running sops command.")?;

        // trust sops output of dotenv format using linefeeds
        // and simply split each newline
        let mut variables = vec![];

        for line in output.lines() {
          if line.starts_with('#') {
              continue;
          }

          let (name, value) = line.split_once('=').unwrap();
          
          
          variables.push(VariableOutput {
            name: name.to_string(),
            value: value.to_string()
          });
        }

        Ok(variables)

    }
}

/**
 * Simple wrapper around sops cli
 */
pub fn run_sops_decrypt(additional_args: Vec<String>, file: &str) -> Result<String, anyhow::Error> {

  let mut final_args: Vec<String> = vec![String::from("--decrypt")];
  final_args.extend(additional_args.clone());
  final_args.push(String::from(file));

  debug!("Running sops command with args: {:?}", &final_args);

  let output = Command::new("sops")
    .args(&final_args)
    .output()
    .with_context(|| format!("Error running sops command with arguments {:?}", &final_args))?;

  let stdout = std::str::from_utf8(&output.stdout)
    .with_context(|| format!("Couldn't decode stdout as UTF-8 for sops command with args: {:?}", &final_args))?;
  
  let stderr = std::str::from_utf8(&output.stderr)
    .with_context(|| format!("Couldn't decode stderr as UTF-8 for sops command with args: {:?}", &final_args))?;

  // sops should not output any secret 
  debug!("sops stderr: '{:}'", &stderr);

  if ! output.status.success() {
    return Err(anyhow!("sops command returned non-0 exit code. args: {:?}, stderr: '{:?}'", &final_args, &stderr));
  };

  Ok(stdout.to_string())

}
