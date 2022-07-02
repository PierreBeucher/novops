use aws_sdk_sts::Client as StsClient;
use uuid::Uuid;
use serde::Deserialize;
use async_trait::async_trait;

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
    async fn resolve(&self, ctx: &NovopsContext) -> Vec<VariableOutput> {
        let config = aws_config::from_env()
        .credentials_provider(
            aws_config::profile::ProfileFileCredentialsProvider::builder()
            .profile_name(&self.source_profile)
            .build()
        )
        .load()
        .await;
    
        // let config = aws_config::from_env().credentials_provider(cred)
        let sts_client = StsClient::new(&config);
        let assumed_role = sts_client.assume_role()
            .role_arn(&self.role_arn)
            .role_session_name(&format!("novops-{:}-{:}-{:}", ctx.env_name, ctx.app_name, Uuid::new_v4()))
            .send().await;
        let aso = match assumed_role {
            Ok(c) => {c},
            Err(e) => panic!("Coudn't assume role {:} (source profile {:}: {:?}", 
            self.role_arn, self.source_profile, e),
        };

        let creds = &aso.credentials.unwrap();
        let access_key = creds.access_key_id.as_ref().unwrap();
        let secret_key = creds.secret_access_key.as_ref().unwrap();
        let session_token = creds.session_token.as_ref().unwrap();

        return vec![
            VariableOutput{name: "AWS_ACCESS_KEY_ID".into(), value: access_key.clone()},
            VariableOutput{name: "AWS_SECRET_ACCESS_KEY".into(), value: secret_key.clone()},
            VariableOutput{name: "AWS_SESSION_TOKEN".into(), value: session_token.clone()} 
        ]
    }
}
