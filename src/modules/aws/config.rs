
use std::str::FromStr;

use crate::{modules::aws::assume_role::AwsAssumeRoleInput};
use crate::core::{NovopsContext};

use schemars::JsonSchema;
use serde::Deserialize;
use aws_smithy_http::endpoint::Endpoint;
use http::Uri;
use anyhow::Error;
use log::debug;

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsInput {
    pub assume_role: AwsAssumeRoleInput
}

/**
 * Create an SdkConfig using optional overrides 
 */
pub async fn get_sdk_config(client_conf: &AwsClientConfig) -> Result<aws_config::SdkConfig, Error> {

    let mut aws_config = aws_config::from_env();
 
    match &client_conf.endpoint {
        Some(endpoint) => {
            let ep_uri = Uri::from_str(endpoint).unwrap();
            aws_config = aws_config.endpoint_resolver(Endpoint::immutable(ep_uri));
        },
        None => {},
    }

    match &client_conf.profile {
        Some(profile) => {
            aws_config = aws_config.credentials_provider(
                aws_config::profile::ProfileFileCredentialsProvider::builder()
                    .profile_name(profile)
                    .build()
            );
        },
        None => {},
    }

    return Ok(aws_config.load().await);
    
}

pub async fn get_iam_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_iam::Client, Error>{
    let conf = get_sdk_config(novops_aws).await?;
    
    debug!("Creating AWS IAM client with config {:?}", conf);
    return Ok(aws_sdk_iam::Client::new(&conf));
}

pub async fn get_sts_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_sts::Client, Error>{
    let conf = get_sdk_config(novops_aws).await?;

    debug!("Creating AWS STS client with config {:?}", conf);
    return Ok(aws_sdk_sts::Client::new(&conf));
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

pub fn build_mutable_client_config_from_context(ctx: &NovopsContext) -> AwsClientConfig {
    let client_conf = match &ctx.config_file_data.config {
        Some(config) => {
            match &config.aws {
                Some(aws) => {
                    AwsClientConfig::from(aws)
                },
                None => AwsClientConfig::default(),
            }
        },
        None => AwsClientConfig::default()
    };

    return client_conf;
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