use std::path::PathBuf;
use convert_case::{Case, Casing};
use async_trait::async_trait;
use serde::Deserialize;
use anyhow;
use schemars::JsonSchema;

use crate::core::{ResolveTo, NovopsContext, StringResolvableInput};
use crate::variables::{VariableOutput};

/**
 * Content is any input
 * File can also define a destination (by default, will use XDG Runtime directory, or a secure /tmp subfolder in XDG is not available)
 * Dest will override default destination to a custom path
 * Variable is an environment variable output pointing to generated file
 * 
 * Example:
 * 
 * dog:
 *   content: "wouf"
 * 
 * Would generate Outputs:
 * - a file such as /run/user/1000/novops/animals/dev/dog
 * - an environment variable such as NOVOPS_ANIMALS_DEV_FILE_DOG="/run/user/1000/novops/animals/dev/dog"
 * 
 * cat:
 *   dest: /tmp/thecat
 *   variable: CAT_LOCATION
 *   content: "meow"
 * 
 * Would generate Outputs:
 * - a file such as /tmp/thecat
 * - an environment variable such as CAT_LOCATION="/tmp/thecat"
 * 
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct FileInput {
    /// name to use when auto-generating file and variable name
    /// if not specified, the YAML key for file will be used
    pub name: Option<String>,

    pub dest: Option<String>,
    
    pub variable: Option<String>,
    
    pub content: StringResolvableInput
}

/**
 * Output for FileInput, with final dest, variable and content
 */
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub struct FileOutput {
    pub dest: PathBuf,
    pub variable: VariableOutput,
    pub content: String // TODO buffer? content may be long
}

/**
 * Resolve a FileInput to its FileOutput for given context
 */
#[async_trait]
impl ResolveTo<FileOutput> for FileInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<FileOutput, anyhow::Error> {
        
        // enforce either name or variable as name is used to auto-generate variable 
        // otherwise we can't affect a deterministic variable name from config
        if self.name.is_none() && self.variable.is_none(){
            panic!("You must specify at least a name or a variable for file {:?}, otherwise associated variable name won't be deterministic", self)
        }

        // if name is provided, use it
        // otherwise use dest as snake user
        // and default a uuid
        let fname = match &self.name {
            Some(s) => s.clone(),
            None => uuid::Uuid::new_v4().to_string()
        };

        // if dest provided, use it
        // otherwise use working directory and file name
        let dest = match &self.dest {
            Some(s) => PathBuf::from(&s),
            None => ctx.workdir.join(format!("file_{:}", &fname))
        };

        // variable pointing to file path
        // if variable name is provided, use it
        // otherwise default to NOVOPS_<env>_<key>
        let variable_name = match &self.variable {
            Some(v) => v.clone(),
            None => format!("NOVOPS_{:}_FILE_{:}", 
                &ctx.app_name.to_case(Case::Snake).to_uppercase(), 
                fname.to_case(Case::Snake).to_uppercase()),
        };

        let content = match self.content.resolve(ctx).await {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        return Ok(
            FileOutput {
                dest:  PathBuf::from(&dest),
                variable: VariableOutput {
                    name: variable_name,
                    value: dest.into_os_string().into_string().unwrap()
                },
                content: content
            }
        )

    }
}