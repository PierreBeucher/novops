use std::collections::HashMap;
use serde::Deserialize;
use convert_case::{Case, Casing};
use async_trait::async_trait;

use crate::bitwarden;
use crate::aws;

#[derive(Debug, Deserialize, Clone)]
pub struct NovopsConfig {
    pub name: String,
    pub environments: HashMap<String, NovopsEnvironment>,
    pub default: Option<NovopsConfigDefault>
}

#[derive(Debug, Deserialize, Clone)]
pub struct NovopsConfigDefault {
    pub environment: Option<String>
}

/**
 * A Novops environment defining Input and Output
 * Environment name is the corresponding YAML key
 * 
 * Available modules:
 * - Variable are simpple Key/Valye using any String input
 * - File are defined using a specific Input struct
 * - AWS allow to assume IAM Role (Output: env vars)
 */
#[derive(Debug, Deserialize, Clone)]
pub struct NovopsEnvironment {
    pub variables: Vec<VariableInput>,
    pub files: Vec<FileInput>,
    pub aws: Option<aws::AwsModule>
}

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
#[derive(Debug, Deserialize, Clone)]
pub struct FileInput {
    /// name to use when auto-generating file and variable name
    /// if not specified, the YAML key for file will be used
    pub name: Option<String>,

    pub dest: Option<String>,
    
    pub variable: Option<String>,
    
    pub content: AnyStringInput
}

/**
 * Output for FileInput, with final dest, variable and content
 */
#[derive(Debug, Clone)]
pub struct FileOutput {
    pub dest: String,
    pub variable: VariableOutput,
    pub content: String // TODO buffer? content may be long
}

/**
 * An environment variable (key / value)
 */
#[derive(Debug, Deserialize, Clone)]
pub struct VariableInput {
    name: String,
    value: AnyStringInput
}

/**
 * Output for VariableInput, with final name and value
 */
#[derive(Debug, Clone)]
pub struct VariableOutput {
    pub name: String,
    pub value: String
}

/**
 * An Input that will always take a String as final Output form
 */
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
#[enum_dispatch(ResolveTo<String>)]
pub enum AnyStringInput {
    String(String),
    BitwardeItemInput(bitwarden::BitwardenItemInput)
}

/**
 * Context in which an environment is loaded. Passed to Inputs with ResolveTo() to generate related Output 
 */
pub struct NovopsContext {

    /// environment name
    pub env_name: String,

    // application name
    pub app_name: String,

    /// working directory under which files are stored
    pub workdir: String,

    /// original config loaded at runtime
    pub config: NovopsConfig,
}

/**
 * Trait all Input are implement to generate their final Output value
 */
#[async_trait]
pub trait ResolveTo<T> {
    async fn resolve(&self, ctx: &NovopsContext) -> T;
}

/**
 * String is the most simple Input to resolve: it resolve to itself
 */
#[async_trait]
impl ResolveTo<String> for String {
    async fn resolve(&self, _: &NovopsContext) -> String {
        return self.clone();
    }
}

#[async_trait]
impl ResolveTo<String> for AnyStringInput {
    async fn resolve(&self, ctx: &NovopsContext) -> String {
        return match &self {
            &AnyStringInput::String(s) => s.clone(),
            &AnyStringInput::BitwardeItemInput(bw) => bw.resolve(ctx).await,
        }
    }
}

#[async_trait]
impl ResolveTo<VariableOutput> for VariableInput {
    async fn resolve(&self, ctx: &NovopsContext) -> VariableOutput {
        return VariableOutput { 
            name: self.name.clone(), 
            value: self.value.resolve(ctx).await
        }
    }
}

/**
 * Resolve a FileInput to its FileOutput for given context
 */
#[async_trait]
impl ResolveTo<FileOutput> for FileInput {
    async fn resolve(&self, ctx: &NovopsContext) -> FileOutput {

        // if name is provided, use it
        // otherwise generate random name
        let fname = match &self.name {
            Some(s) => s.clone(),
            None => uuid::Uuid::new_v4().to_string()
        };

        // if dest provided, use it
        // otherwise use working directory and a random name
        let dest = match &self.dest {
            Some(s) => s.clone(),
            None => format!("{:}/file_{:}", &ctx.workdir, &fname)
        };

        // variable pointing to file path
        // if variable name is provided, use it
        // otherwise default to NOVOPS_<env>_<key>
        let variable_name = match &self.variable {
            Some(v) => v.clone(),
            None => format!("NOVOPS_{:}_{:}_FILE_{:}", 
                &ctx.app_name.to_case(Case::Snake).to_uppercase(), 
                &ctx.env_name.to_case(Case::Snake).to_uppercase(), 
                fname.to_case(Case::Snake).to_uppercase()),
        };
        
        return FileOutput {
            dest: dest.clone(),
            variable: VariableOutput {
                name: variable_name,
                value: dest.clone()
            },
            content: self.content.resolve(ctx).await
        };

    }
}