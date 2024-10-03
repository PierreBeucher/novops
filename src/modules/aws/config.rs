use crate::modules::aws::assume_role::AwsAssumeRoleInput;

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsInput {
    pub assume_role: AwsAssumeRoleInput
}


/**
 * Generic AWS Client config wrapped around builder pattern
 * for easy loading from Novops config and per-module override
 */
#[derive(Default)]
pub struct AwsClientConfig {
    pub profile: Option<String>,
    pub endpoint: Option<String>,
    pub region: Option<String>,
    pub identity_cache: Option<IdentityCache>,
}

impl From<&AwsConfig> for AwsClientConfig {
    fn from(cf: &AwsConfig) -> AwsClientConfig{
        AwsClientConfig {
            profile: cf.profile.clone(),
            endpoint: cf.endpoint.clone(),
            region: cf.region.clone(),
            identity_cache: cf.identity_cache.clone()
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
    pub region: Option<String>,

    /// AWS SDK identity cache configuration
    pub identity_cache: Option<IdentityCache>
}

/// AWS SDK identity cache configuration
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct IdentityCache {
    /// Timeout to load identity (in seconds, default: 5s).
    /// Useful when asking for MFA authentication which may take more than 5 seconds for user to input.
    pub load_timeout: Option<u64>
}

