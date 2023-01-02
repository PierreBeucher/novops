use crate::{modules::aws::assume_role::AwsAssumeRoleInput};

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
pub struct AwsClientConfig {
    pub profile: Option<String>,
    pub endpoint: Option<String>
}

impl Default for AwsClientConfig {
    fn default() -> Self {
        AwsClientConfig {
            profile: None,
            endpoint: None
        }
    }
}

impl From<&AwsConfig> for AwsClientConfig {
    fn from(cf: &AwsConfig) -> AwsClientConfig{
        return AwsClientConfig {
            profile: cf.profile.clone(),
            endpoint: cf.endpoint.clone()
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

/**
 * Global AWS config
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsConfig {
    
    /**
     * Override endpoint for all AWS services
     * Can be used with tools like LocalStack (http://localhost:4566/)
     */
    pub endpoint: Option<String>,

    /**
     * AWS Profile to use when resolving inputs
     */
    pub profile: Option<String>
}

impl Default for AwsConfig {
    fn default() -> AwsConfig {
        AwsConfig{
          endpoint: None,
          profile: None
        }
      }
}