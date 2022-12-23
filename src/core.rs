use std::collections::HashMap;
use serde::Deserialize;
use async_trait::async_trait;
use anyhow;
use std::path::PathBuf;
use schemars::JsonSchema;

use crate::modules::hashivault::{config::HashivaultConfig, kv2::HashiVaultKeyValueV2Input, kv1::HashiVaultKeyValueV1Input};
use crate::modules::bitwarden;
use crate::modules::aws;
use crate::modules::gcloud;
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
    pub hashivault: Option<HashivaultConfig>,
    pub aws: Option<aws::config::AwsConfig>
}

impl Default for NovopsConfig {
    fn default() -> NovopsConfig {
        NovopsConfig {
            default: None,
            hashivault: None,
            aws: None
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
    pub aws: Option<aws::config::AwsInput>
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
    pub env_var_filepath: PathBuf,

    // enable dry run mode
    pub dry_run: bool
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
 * Centralize ResolveTo<String> to reduce boiler-plate code on other inputs
 * like FileInput and VariableInput
 * 
 * Most modules Inputs will resolve to a String which can be used either
 * as Variable or File. See [BytesResolvableInput] if you need to handle binary data.
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum StringResolvableInput {
    String(String),
    BitwardeItemInput(bitwarden::BitwardenItemInput),
    HashiVaultKeyValueV2Input(HashiVaultKeyValueV2Input),
    HashiVaultKeyValueV1Input(HashiVaultKeyValueV1Input),
    AwsSSMParamStoreInput(aws::ssm::AwsSSMParamStoreInput),
    AwsSecretsManagerSecretInput(aws::secretsmanager::AwsSecretsManagerSecretInput),
    GCloudSecretManagerSecretInput(gcloud::secretmanager::GCloudSecretManagerSecretInput)
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
        return match self {
            StringResolvableInput::String(s) => Ok(s.clone()),
            StringResolvableInput::BitwardeItemInput(bw) => bw.resolve(ctx).await,
            StringResolvableInput::HashiVaultKeyValueV2Input(hv) => hv.resolve(ctx).await,
            StringResolvableInput::HashiVaultKeyValueV1Input(hv) => hv.resolve(ctx).await,
            StringResolvableInput::AwsSSMParamStoreInput(p) => p.resolve(ctx).await,
            StringResolvableInput::AwsSecretsManagerSecretInput(s) => s.resolve(ctx).await,
            StringResolvableInput::GCloudSecretManagerSecretInput(s) => s.resolve(ctx).await
        }
    }
}


/**
 * Enum for all Inputs resolving to a Byte Vector (including [StringResolvableInput]
 * as Rust internal structure represents String as Byte vector)
 * This input is used by [FileInput] to resolve file content whic is not always a String but may also 
 * be binary or blob data. 
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum BytesResolvableInput {
    AwsSecretsManagerSecretInput(aws::secretsmanager::AwsSecretsManagerSecretInput),
    GCloudSecretManagerSecretInput(gcloud::secretmanager::GCloudSecretManagerSecretInput),
    StringResolvableInput(StringResolvableInput),
    ByteVec(Vec<u8>)
}

#[async_trait]
impl ResolveTo<Vec<u8>> for BytesResolvableInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<u8>, anyhow::Error> {

        let result = match self {
            BytesResolvableInput::ByteVec(z) => Ok(z.clone()),
            BytesResolvableInput::AwsSecretsManagerSecretInput(z) => z.resolve(ctx).await,
            BytesResolvableInput::GCloudSecretManagerSecretInput(z) => z.resolve(ctx).await,
            BytesResolvableInput::StringResolvableInput(z) => z.resolve(ctx).await.map(|x| x.into_bytes()),
        };
        
        return result;
    }
}