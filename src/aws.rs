use aws_sdk_sts::Client as StsClient;
use serde::Deserialize;
use async_trait::async_trait;
use anyhow::{self, Context};
use rand::{distributions::Alphanumeric, Rng};

use crate::novops::{ResolveTo, NovopsContext};
use crate::variables::VariableOutput;

#[derive(Debug, Deserialize, Clone)]
pub struct AwsInput {
    pub assume_role: AwsAssumeRoleInput
}

#[derive(Debug, Deserialize, Clone)]
pub struct AwsAssumeRoleInput {
    pub role_arn: String,
    pub source_profile: String
}

#[async_trait]
impl ResolveTo<Vec<VariableOutput>> for AwsAssumeRoleInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<VariableOutput>, anyhow::Error> {
        let config = aws_config::from_env()
        .credentials_provider(
            aws_config::profile::ProfileFileCredentialsProvider::builder()
            .profile_name(&self.source_profile)
            .build()
        )
        .load()
        .await;
    
        let sts_client = StsClient::new(&config);

        let session_random_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // session name is max 64 characters length
        // by default use full app and env name with random suffix
        // truncate if name > 64 chars to avoid error but print warning
        // see https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html
        let mut role_session_name = format!("novops-{:}-{:}-{:}", &ctx.app_name, &ctx.env_name, &session_random_suffix);
        
        if role_session_name.len() > 64 {
           println!("WARNING: Computed AWS STS Role session name length > 64 characters and will be truncated. \
           Consider using shorter application or environment name to avoid losing information with truncation. \
           Role session name before truncate: '{:}'", 
           role_session_name);
        }

        role_session_name.truncate(64);

        let assumed_role = sts_client.assume_role()
            .role_arn(&self.role_arn)
            .role_session_name(role_session_name)
            .send().await;
        
        let aso = match assumed_role {
            Ok(c) => {c},
            Err(e) => return Err(e.into())
        };

        let creds = &aso.credentials.clone()
            .with_context(|| format!("Can't assume role: returned Credentials Option was None for {:?}", &aso))?;

        let access_key = creds.access_key_id.as_ref()
            .with_context(|| format!("Can't assume role: Returned access key Option was None for {:?}", &aso))?;

        let secret_key = creds.secret_access_key.as_ref()
            .with_context(|| format!("Can't assume role: returned secret key Option was None for {:?}", &aso))?;

        let session_token = creds.session_token.as_ref()
            .with_context(|| format!("Can't assume role: returned access key Option was None for {:?}", &aso))?;
        
        return Ok(
            vec![
                VariableOutput{name: "AWS_ACCESS_KEY_ID".into(), value: access_key.clone()},
                VariableOutput{name: "AWS_SECRET_ACCESS_KEY".into(), value: secret_key.clone()},
                VariableOutput{name: "AWS_SESSION_TOKEN".into(), value: session_token.clone()} 
            ]
        )
    }
}
