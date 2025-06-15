use serde::Deserialize;
use async_trait::async_trait;
use anyhow::{self, Context};
use rand::{distributions::Alphanumeric, Rng};
use schemars::JsonSchema;
use log::warn;

use crate::core::{ResolveTo, NovopsContext};
use crate::modules::variables::VariableOutput;
use crate::modules::aws::client::get_client_with_profile;

const STS_ROLE_SESSION_NAME_MAX_LENGTH: usize = 64;

/// Assume an IAM Role 
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsAssumeRoleInput {

    /// Full IAM Role ARN
    pub role_arn: String,

    /// Source profile. Must exist in config. 
    pub source_profile: Option<String>,

    /// Duration of the role session (seconds). 
    /// Can range from 900 seconds up to the maximum session duration set for the role.
    /// Default to 1h (3600).
    pub duration_seconds: Option<i32>
}

#[async_trait]
impl ResolveTo<Vec<VariableOutput>> for AwsAssumeRoleInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<VariableOutput>, anyhow::Error> {
        
        let client = get_client_with_profile(ctx, &self.source_profile).await;

        let session_random_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .collect();

        let mut role_session_name = format!("novops-{:}-{:}-{:}", &ctx.app_name, &ctx.env_name, &session_random_suffix);

        // session name is max 64 characters length
        // by default use full app and env name with random suffix
        // truncate if name > 64 chars to avoid error but print warning
        // see https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html        
        if role_session_name.len() > STS_ROLE_SESSION_NAME_MAX_LENGTH {
           let original_role_session_name = role_session_name.clone();

           // when truncating, truncate based on app and env name but keep random identifier
           let mut truncated_rsname = format!("novops-{:}-{:}", &ctx.app_name, &ctx.env_name);
           truncated_rsname.truncate(STS_ROLE_SESSION_NAME_MAX_LENGTH-session_random_suffix.len()-1);

           role_session_name = format!("{:}-{:}", truncated_rsname, &session_random_suffix);

           warn!("WARNING: Role session name {:} truncated to {:} as length > 64 characters. \
           Consider using shorter application or environment name to avoid losing information with truncation.", 
           &original_role_session_name, &role_session_name);
        }

        let duration_seconds = self.duration_seconds.unwrap_or(3600);

        let assumed_role = client.assume_role(&self.role_arn, &role_session_name, duration_seconds).await?;
    
        let creds = &assumed_role.credentials.clone()
            .with_context(|| format!("Can't assume role: returned Credentials Option was None for {:?}", &assumed_role))?;

        return Ok(
            vec![
                VariableOutput{name: "AWS_ACCESS_KEY_ID".into(), value: creds.access_key_id.clone()},
                VariableOutput{name: "AWS_SECRET_ACCESS_KEY".into(), value: creds.secret_access_key.clone()},
                VariableOutput{name: "AWS_SESSION_TOKEN".into(), value: creds.session_token.clone()} ,
                VariableOutput{name: "AWS_SESSION_EXPIRATION".into(), value: creds.expiration.clone().secs().to_string() } 
            ]
        )
    }
}