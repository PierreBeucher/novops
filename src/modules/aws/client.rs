use crate::core::NovopsContext;
use super::config::AwsClientConfig;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_secretsmanager::operation::get_secret_value::GetSecretValueOutput;
use aws_sdk_sts::{operation::assume_role::AssumeRoleOutput, types::builders::CredentialsBuilder};
use aws_sdk_ssm::{operation::get_parameter::GetParameterOutput, types::builders::ParameterBuilder};
use aws_sdk_s3::{operation::get_object::GetObjectOutput, primitives::ByteStream};
use anyhow::Context;
use aws_smithy_types::DateTime;
use log::debug;
use async_trait::async_trait;

/**
 * SIngle wrapper around various AWS clients
 */
#[async_trait]
pub trait AwsClient {
    async fn get_secret_value(&self, id: &str, version_id: Option<String>, version_stage: Option<String>) -> Result<GetSecretValueOutput, anyhow::Error>;

    async fn get_ssm_parameter(&self, name: &str, decrypt: Option<bool>) -> Result<GetParameterOutput, anyhow::Error>;

    async fn assume_role(&self, role_arn: &str, session_name: &str) -> Result<AssumeRoleOutput, anyhow::Error>;

    async fn get_s3_object(&self, bucket: &str, key: &str, region: &Option<String>) -> Result<GetObjectOutput, anyhow::Error>;
}

pub async fn get_client(ctx: &NovopsContext) -> Box<dyn AwsClient + Send + Sync> {
    if ctx.dry_run {
        Box::new(DryRunAwsClient{})
    } else {
        Box::new(DefaultAwsClient{
            config: build_mutable_client_config_from_context(ctx)
        })
    }
}

pub async fn get_client_with_profile(ctx: &NovopsContext, profile: &Option<String>) -> Box<dyn AwsClient + Send + Sync> {
    if ctx.dry_run {
        Box::new(DryRunAwsClient{})
    } else {
        let mut config = build_mutable_client_config_from_context(ctx);

        if let Some(p) = profile{
            config.profile(p);
        }
        
        Box::new(DefaultAwsClient{
            config
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
            .duration_seconds(3600) // TODO as config
            .send().await
            .with_context(|| format!("Couldn't impersonate role {:} (session name: {:?})", role_arn, session_name))
    }

    async fn get_s3_object(&self, bucket: &str, key: &str, region: &Option<String>) -> Result<GetObjectOutput, anyhow::Error> {
        let client = get_s3_client(&self.config, region).await?;
        client.get_object()
            .bucket(bucket)
            .key(key)
            .send().await
            .with_context(|| format!("Couldn't get S3 object '{}/{}'", bucket, key))
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
        let param = ParameterBuilder::default()
            .name(String::from(name))
            .value(format!("RESULT:{:}", name))
            .build();

        Ok(GetParameterOutput::builder()
            .parameter(param)
            .build())
    }

    async fn assume_role(&self, _role_arn: &str, _session_name: &str) -> Result<AssumeRoleOutput, anyhow::Error>{
        let exp = DateTime::from_str("2999-01-01T00:00:00Z", aws_smithy_types::date_time::Format::DateTime)?;
        let creds = CredentialsBuilder::default()
            .access_key_id("AKIADRYRUNDRYUNDRYRUN")
            .secret_access_key("xxx")
            .session_token("xxx")
            .expiration(exp)
            .build()?;

        let result = AssumeRoleOutput::builder()
            .credentials(creds)
            .build();

        Ok(result)
    }

    async fn get_s3_object(&self, _: &str, _: &str, _: &Option<String>) -> Result<GetObjectOutput, anyhow::Error> {
        Ok(GetObjectOutput::builder()
            .body(ByteStream::from_static(b"dummy"))
            .build())
    }
}

pub fn build_mutable_client_config_from_context(ctx: &NovopsContext) -> AwsClientConfig {
    

    match &ctx.config_file_data.config {
        Some(config) => {
            match &config.aws {
                Some(aws) => {
                    AwsClientConfig::from(aws)
                },
                None => AwsClientConfig::default(),
            }
        },
        None => AwsClientConfig::default()
    }
}

/**
 * Create an SdkConfig using optional overrides 
 */
pub async fn get_sdk_config(client_conf: &AwsClientConfig) -> Result<aws_config::SdkConfig, anyhow::Error> {
    
    let mut shared_config = aws_config::defaults(BehaviorVersion::v2024_03_28());

    if let Some(endpoint) = &client_conf.endpoint {
        shared_config = shared_config.endpoint_url(endpoint);
    }

    if let Some(profile) = &client_conf.profile {
        shared_config = shared_config.profile_name(profile);
    }

    if let Some(region) = &client_conf.region {
        shared_config = shared_config.region(Region::new(region.clone()));
    }

    Ok(shared_config.load().await)

}

pub async fn get_iam_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_iam::Client, anyhow::Error>{
    let conf = get_sdk_config(novops_aws).await?;
    
    debug!("Creating AWS IAM client with config {:?}", conf);
    Ok(aws_sdk_iam::Client::new(&conf))
}

pub async fn get_sts_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_sts::Client, anyhow::Error>{
    let conf = get_sdk_config(novops_aws).await?;

    debug!("Creating AWS STS client with config {:?}", conf);
    Ok(aws_sdk_sts::Client::new(&conf))
}

pub async fn get_ssm_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_ssm::Client, anyhow::Error>{
    let conf = get_sdk_config(novops_aws).await?;

    debug!("Creating AWS SSM client with config {:?}", conf);
    Ok(aws_sdk_ssm::Client::new(&conf))
}

pub async fn get_secretsmanager_client(novops_aws: &AwsClientConfig) -> Result<aws_sdk_secretsmanager::Client, anyhow::Error>{
    let conf = get_sdk_config(novops_aws).await?;

    debug!("Creating AWS Secrets Manager client with config {:?}", conf);
    Ok(aws_sdk_secretsmanager::Client::new(&conf))
}

pub async fn get_s3_client(novops_aws: &AwsClientConfig, region: &Option<String>) -> Result<aws_sdk_s3::Client, anyhow::Error> {
    let conf = get_sdk_config(novops_aws).await?;

    debug!("Creating AWS S3 client with config {:?}", conf);

    let mut s3_conf = aws_sdk_s3::config::Builder::from(&conf)
        .force_path_style(true);

    if let Some(r) = region.clone() {
        s3_conf = s3_conf.region(Region::new(r));
    };
    
    Ok(aws_sdk_s3::Client::from_conf(s3_conf.build()))
}
