use crate::core::{NovopsContext};
use super::config::AwsClientConfig;
use aws_sdk_secretsmanager::output::GetSecretValueOutput;
use aws_sdk_ssm::{output::GetParameterOutput, model::Parameter};
use aws_sdk_sts::output::AssumeRoleOutput;
use aws_sdk_sts::model::Credentials;
use aws_smithy_http::endpoint::Endpoint;
use http::Uri;
use anyhow::Context;
use log::debug;
use std::str::FromStr;
use async_trait::async_trait;

/**
 * SIngle wrapper around various AWS clients
 */
#[async_trait]
pub trait AwsClient {
    async fn get_secret_value(&self, id: &str, version_id: Option<String>, version_stage: Option<String>) -> Result<GetSecretValueOutput, anyhow::Error>;

    async fn get_ssm_parameter(&self, name: &str, decrypt: Option<bool>) -> Result<GetParameterOutput, anyhow::Error>;

    async fn assume_role(&self, role_arn: &str, session_name: &str) -> Result<AssumeRoleOutput, anyhow::Error>;
}

pub async fn get_client(ctx: &NovopsContext) -> Box<dyn AwsClient + Send + Sync> {
    if ctx.dry_run {
        return Box::new(DryRunAwsClient{})
    } else {
        return Box::new(DefaultAwsClient{
            config: build_mutable_client_config_from_context(ctx)
        })
    }
}

pub async fn get_client_with_profile(ctx: &NovopsContext, profile: &Option<String>) -> Box<dyn AwsClient + Send + Sync> {
    if ctx.dry_run {
        return Box::new(DryRunAwsClient{})
    } else {
        let mut config = build_mutable_client_config_from_context(ctx);

        if let Some(p) = profile{
            config.profile(p);
        }
        
        return Box::new(DefaultAwsClient{
            config: config
        })
    }
}

pub struct DefaultAwsClient{
    config: AwsClientConfig
}
pub struct DryRunAwsClient{}

#[async_trait]
impl AwsClient for DefaultAwsClient {
    async fn get_secret_value(&self, id: &str, version_id: Option<String>, version_stage: Option<String>) -> Result<GetSecretValueOutput, anyhow::Error>{
        let client = get_secretsmanager_client(&self.config).await?;
        client.get_secret_value()
            .secret_id(id)
            .set_version_id(version_id.clone())
            .set_version_stage(version_stage.clone())
            .send().await
            .with_context(|| format!("Couldn't request secret {:} (version: {:?}, version stage: {:?})",
                &id, &version_id, &version_stage))
    }

    async fn get_ssm_parameter(&self, name: &str, decrypt: Option<bool>) -> Result<GetParameterOutput, anyhow::Error>{
        let client = get_ssm_client(&self.config).await?;
        client.get_parameter()
            .name(name)
            .with_decryption(decrypt.unwrap_or(true))
            .send().await
            .with_context(|| format!("Couldn't request SSM parameter {:} (decrypt: {:?})", name, decrypt))
    }

    async fn assume_role(&self, role_arn: &str, session_name: &str) -> Result<AssumeRoleOutput, anyhow::Error>{
        let client = get_sts_client(&self.config).await?;
        client.assume_role()
            .role_arn(role_arn) 
            .role_session_name(session_name)
            .send().await
            .with_context(|| format!("Couldn't impersonate role {:} (session name: {:?})", role_arn, session_name))
    }
}

#[async_trait]
impl AwsClient for DryRunAwsClient{
    async fn get_secret_value(&self, id: &str, _version_id: Option<String>, _version_stage: Option<String>) -> Result<GetSecretValueOutput, anyhow::Error>{
        let r = GetSecretValueOutput::builder()
            .secret_string(format!("RESULT:{:}", id))
            .build();
        
            Ok(r)
    }

    async fn get_ssm_parameter(&self, name: &str, _decrypt: Option<bool>) -> Result<GetParameterOutput, anyhow::Error>{
        let parameter = Parameter::builder()
            .value(format!("RESULT:{:}", name))
            .build();

        Ok(GetParameterOutput::builder()
            .parameter(parameter)
            .build())
    }

    async fn assume_role(&self, _role_arn: &str, _session_name: &str) -> Result<AssumeRoleOutput, anyhow::Error>{
        let creds = Credentials::builder()
            .access_key_id("AKIADRYRUNDRYUNDRYRUN")
            .secret_access_key("xxx")
            .session_token("xxx")
            .build();

        let result = AssumeRoleOutput::builder()
            .credentials(creds)
            .build();

        Ok(result)
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
 * Create an SdkConfig using optional overrides 
 */
pub async fn get_sdk_config(client_conf: &AwsClientConfig) -> Result<aws_config::SdkConfig, anyhow::Error> {

    let mut aws_config = aws_config::from_env();
 
    match &client_conf.endpoint {
        Some(endpoint) => {
            let ep_uri = Uri::from_str(endpoint)
                .with_context(|| format!("Couldn't create endpoint URI from string '{:}'", endpoint))?;
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

pub async fn get_iam_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_iam::Client, anyhow::Error>{
    let conf = get_sdk_config(novops_aws).await?;
    
    debug!("Creating AWS IAM client with config {:?}", conf);
    return Ok(aws_sdk_iam::Client::new(&conf));
}

pub async fn get_sts_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_sts::Client, anyhow::Error>{
    let conf = get_sdk_config(novops_aws).await?;

    debug!("Creating AWS STS client with config {:?}", conf);
    return Ok(aws_sdk_sts::Client::new(&conf));
}

pub async fn get_ssm_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_ssm::Client, anyhow::Error>{
    let conf = get_sdk_config(novops_aws).await?;

    debug!("Creating AWS SSM client with config {:?}", conf);
    return Ok(aws_sdk_ssm::Client::new(&conf));
}

pub async fn get_secretsmanager_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_secretsmanager::Client, anyhow::Error>{
    let conf = get_sdk_config(novops_aws).await?;

    debug!("Creating AWS Secrets Manager client with config {:?}", conf);
    return Ok(aws_sdk_secretsmanager::Client::new(&conf));
}