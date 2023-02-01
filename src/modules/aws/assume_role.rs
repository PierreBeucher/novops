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

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsAssumeRoleInput {
    pub role_arn: String,
    pub source_profile: Option<String>
}

#[async_trait]
impl ResolveTo<Vec<VariableOutput>> for AwsAssumeRoleInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<VariableOutput>, anyhow::Error> {
        
        let client = get_client_with_profile(ctx, &self.source_profile).await;

        let session_random_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
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

        let assumed_role = client.assume_role(&self.role_arn, &role_session_name).await?;
    
        let creds = &assumed_role.credentials.clone()
            .with_context(|| format!("Can't assume role: returned Credentials Option was None for {:?}", &assumed_role))?;

        let access_key = creds.access_key_id.as_ref()
            .with_context(|| format!("Can't assume role: Returned access key Option was None for {:?}", &assumed_role))?;

        let secret_key = creds.secret_access_key.as_ref()
            .with_context(|| format!("Can't assume role: returned secret key Option was None for {:?}", &assumed_role))?;

        let session_token = creds.session_token.as_ref()
            .with_context(|| format!("Can't assume role: returned access key Option was None for {:?}", &assumed_role))?;
        
        return Ok(
            vec![
                VariableOutput{name: "AWS_ACCESS_KEY_ID".into(), value: access_key.clone()},
                VariableOutput{name: "AWS_SECRET_ACCESS_KEY".into(), value: secret_key.clone()},
                VariableOutput{name: "AWS_SESSION_TOKEN".into(), value: session_token.clone()} 
            ]
        )
    }
}