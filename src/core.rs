use std::collections::HashMap;
use serde::Deserialize;
use async_trait::async_trait;
use anyhow;
use std::path::PathBuf;
use schemars::JsonSchema;

use crate::modules::hashivault;
use crate::modules::bitwarden;
use crate::modules::aws;
use crate::modules::files::{FileInput};
use crate::modules::variables::{VariableInput};

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NovopsConfigFile {
    pub name: String,
    pub environments: HashMap<String, NovopsEnvironmentInput>,
    pub config: Option<NovopsConfig>
}

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NovopsConfig {
    pub default: Option<NovopsConfigDefault>,
    pub hashivault: Option<hashivault::HashivaultConfig>
}

impl Default for NovopsConfig {
    fn default() -> NovopsConfig {
        NovopsConfig {
            default: None,
            hashivault: None
        }
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]    
pub struct NovopsConfigDefault {
    pub environment: Option<String>,
}

impl Default for NovopsConfigDefault {
    fn default() -> NovopsConfigDefault {
        NovopsConfigDefault {
            environment: None,
        }
    }
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
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NovopsEnvironmentInput {
    pub variables: Option<Vec<VariableInput>>,
    pub files: Option<Vec<FileInput>>,
    pub aws: Option<aws::AwsInput>
}

/**
 * Context in which an environment is loaded. Passed to Inputs with ResolveTo() to generate related Output 
 */
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct NovopsContext {

    /// environment name
    pub env_name: String,

    // application name
    pub app_name: String,

    /// working directory under which files are stored
    pub workdir: PathBuf,

    /// original config loaded at runtime
    pub config_file_data: NovopsConfigFile,

    /// path to sourceable environment variable file
    pub env_var_filepath: PathBuf
}

/**
 * Trait all Input are implement to generate their final Output value
 */
#[async_trait]
pub trait ResolveTo<T> {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<T, anyhow::Error>;
}

/**
 * Enum with Input that will always resolve to String
 * i.e. <impl ResolveTo<String>>
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
#[enum_dispatch(ResolveTo<String>)]
pub enum StringResolvableInput {
    String(String),
    BitwardeItemInput(bitwarden::BitwardenItemInput),
    HashiVaultKeyValueV2Input(hashivault::HashiVaultKeyValueV2Input)
}

/**
 * String is the most simple Input to resolve: it resolve to itself
 */
#[async_trait]
impl ResolveTo<String> for String {
    async fn resolve(&self, _: &NovopsContext) -> Result<String, anyhow::Error> {
        return Ok(self.clone());
    }
}

#[async_trait]
impl ResolveTo<String> for StringResolvableInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {
        return match &self {
            &StringResolvableInput::String(s) => Ok(s.clone()),
            &StringResolvableInput::BitwardeItemInput(bw) => bw.resolve(ctx).await,
            &StringResolvableInput::HashiVaultKeyValueV2Input(hv) => hv.resolve(ctx).await,
        }
    }
}
