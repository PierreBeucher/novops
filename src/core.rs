use std::collections::HashMap;
use serde::Deserialize;
use async_trait::async_trait;
use anyhow;
use std::path::PathBuf;
use schemars::JsonSchema;

use crate::modules::hashivault::{
    self,
    config::HashivaultConfig, 
    kv2::HashiVaultKeyValueV2Input, 
    kv1::HashiVaultKeyValueV1Input, 
};
use crate::modules::bitwarden;
use crate::modules::aws;
use crate::modules::gcloud;
use crate::modules::azure;
use crate::modules::files::{FileInput};
use crate::modules::variables::{VariableInput};

/// Available environments. Keys are environment names. 
type NovopsEnvironments = HashMap<String, NovopsEnvironmentInput>;

///
/// Main Novops config file
/// 
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NovopsConfigFile {
    /// Application name. Informational only. 
    /// 
    /// If not specified, use current directory name
    pub name: Option<String>,

    /// Source of truth defining files and variables loaded by Novops
    /// 
    /// Environments are named uniquely (such as "dev", "prod"...) 
    /// to allow for different configs to be loaded in various contexts
    pub environments: NovopsEnvironments,

    /// Global configurations for Novops and modules 
    pub config: Option<NovopsConfig>
}

///
/// Global Novops configuration defining behavior for modules
/// 
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NovopsConfig {
    /// Novops default configurations
    pub default: Option<NovopsConfigDefault>,

    /// Hashicorp Vault module configs
    pub hashivault: Option<HashivaultConfig>,

    /// AWS module configs
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
    /// Default environment name, selected by default if no user input is provided
    pub environment: Option<String>,
}

impl Default for NovopsConfigDefault {
    fn default() -> NovopsConfigDefault {
        NovopsConfigDefault {
            environment: None,
        }
    }
}


/// Modules to be loaded for an environment. Each module defines one or more Input
/// which will be resolved into Outputs (files & variables)
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct NovopsEnvironmentInput {
    
    /// Variables resolving to environment variables from provided source
    pub variables: Option<Vec<VariableInput>>,

    /// Files resolving to concrete files on local filesystem and environment variables pointing to file
    pub files: Option<Vec<FileInput>>,

    /// Assume an AWS Role from local config. 
    /// 
    /// Outputs environment variables `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_SESSION_TOKEN` 
    /// with temporary credentials for IAM Role.
    pub aws: Option<aws::config::AwsInput>,

    /// Reference one or more Hashicorp Vault Secret Engines to generate either files or variables.
    pub hashivault: Option<hashivault::config::HashiVaultInput>
}


/// Context in which an environment is loaded. Passed to Inputs with ResolveTo() to generate related Output 
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


/// Trait all Input are implement to generate their final Output value
#[async_trait]
pub trait ResolveTo<T> {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<T, anyhow::Error>;
}


/// All possible inputs resolving to a string value
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum StringResolvableInput {
    String(String),
    BitwardeItemInput(bitwarden::BitwardenItemInput),
    HashiVaultKeyValueV2Input(HashiVaultKeyValueV2Input),
    HashiVaultKeyValueV1Input(HashiVaultKeyValueV1Input),
    AwsSSMParamStoreInput(aws::ssm::AwsSSMParamStoreInput),
    AwsSecretsManagerSecretInput(aws::secretsmanager::AwsSecretsManagerSecretInput),
    GCloudSecretManagerSecretInput(gcloud::secretmanager::GCloudSecretManagerSecretInput),
    AzureKeyvaultSecretInput(azure::vault::AzureKeyvaultSecretInput)
}


/// String is the most simple Input to resolve: it resolve to itself
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
            StringResolvableInput::GCloudSecretManagerSecretInput(s) => s.resolve(ctx).await,
            StringResolvableInput::AzureKeyvaultSecretInput(z) => z.resolve(ctx).await,
        }
    }
}



/// Any input to be used for file content. 
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum BytesResolvableInput {
    AwsSecretsManagerSecretInput(aws::secretsmanager::AwsSecretsManagerSecretInput),
    GCloudSecretManagerSecretInput(gcloud::secretmanager::GCloudSecretManagerSecretInput),
    StringResolvableInput(StringResolvableInput),

    // skip for schema doc generation as it's useless for human user
    // only useful for internal transformation of blobs into strings
    #[schemars(skip)] 
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