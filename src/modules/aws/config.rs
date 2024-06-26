use crate::modules::aws::assume_role::AwsAssumeRoleInput;

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsInput {
    pub assume_role: AwsAssumeRoleInput
}


/**
 * Generic AWS Client config xrapped around builder pattern
 * for easy loading from Novops config and per-module override
 */
#[derive(Default)]
pub struct AwsClientConfig {
    pub profile: Option<String>,
    pub endpoint: Option<String>,
    pub region: Option<String>,
}

impl From<&AwsConfig> for AwsClientConfig {
    fn from(cf: &AwsConfig) -> AwsClientConfig{
        AwsClientConfig {
            profile: cf.profile.clone(),
            endpoint: cf.endpoint.clone(),
            region: cf.region.clone(),
        }
    }
}

impl AwsClientConfig {
    pub fn profile<'a>(&'a mut self, profile: &str) ->  &'a mut AwsClientConfig {
        self.profile = Some(profile.to_string());
        self
    }

    pub fn endpoint<'a>(&'a mut self, endpoint: &str)->  &'a mut AwsClientConfig{
        self.endpoint = Some(endpoint.to_string());
        self
    }
}


/// Global AWS config
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct AwsConfig {
    
    /// Override endpoint for all AWS services
    /// Can be used with tools like LocalStack, for example http://localhost:4566/
    pub endpoint: Option<String>,

    /// AWS Profile name. Must exist locally in AWS config. 
    /// 
    /// It's advised not to use this directly as profile name configuration is higly dependent
    /// on local configuration. Prefer using AWS_PROFILE environment variable where needed. 
    pub profile: Option<String>,

    /// AWS region to use. Default to currently configured region. 
    pub region: Option<String>
}

