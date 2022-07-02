use std::collections::HashMap;
use serde::Deserialize;
use async_trait::async_trait;

use crate::bitwarden;
use crate::aws;
use crate::files::{FileInput};
use crate::variables::{VariableInput};

#[derive(Debug, Deserialize, Clone)]
pub struct NovopsConfig {
    pub name: String,
    pub environments: HashMap<String, NovopsEnvironmentInput>,
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
pub struct NovopsEnvironmentInput {
    pub variables: Vec<VariableInput>,
    pub files: Vec<FileInput>,
    pub aws: Option<aws::AwsInput>
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
 * Enum with Input that will always resolve to String
 * i.e. <impl ResolveTo<String>>
 */
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
#[enum_dispatch(ResolveTo<String>)]
pub enum StringResolvableInput {
    String(String),
    BitwardeItemInput(bitwarden::BitwardenItemInput)
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
impl ResolveTo<String> for StringResolvableInput {
    async fn resolve(&self, ctx: &NovopsContext) -> String {
        return match &self {
            &StringResolvableInput::String(s) => s.clone(),
            &StringResolvableInput::BitwardeItemInput(bw) => bw.resolve(ctx).await,
        }
    }
}
