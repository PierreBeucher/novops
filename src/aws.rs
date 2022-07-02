use std::error::Error;
use std::fmt;
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

#[derive(Debug)]
struct AwsError {
    pub message: String
}

impl Error for AwsError {}
impl fmt::Display for AwsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:}", self.message)
    }
}

#[async_trait]
impl ResolveTo<Vec<VariableOutput>> for AwsAssumeRoleInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<VariableOutput>, Box<dyn Error>> {
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
            Err(e) => return Err(e.into())
        };

        let creds = match &aso.credentials {
            Some(c) => c,
            None => return Err( AwsError { 
                message: format!("Can't assume role: returned Credentials Option was None for {:?}", &aso)
            }.into())
        };

        let access_key = match creds.access_key_id.as_ref() {
            Some(k) => k,
            None => return Err( AwsError { 
                message: format!("Can't assume role: Returned access key Option was None for {:?}", &aso)
            }.into())
        };

        let secret_key = match creds.secret_access_key.as_ref() {
            Some(k) => k,
            None => return Err( AwsError { 
                message: format!("Can't assume role: returned secret key Option was None for {:?}", &aso)
            }.into())
        };

        let session_token = match creds.session_token.as_ref() {
            Some(k) => k,
            None => return Err( AwsError { 
                message: format!("Can't assume role: returned access key Option was None for {:?}", &aso)
            }.into())
        };

        return Ok(
            vec![
                VariableOutput{name: "AWS_ACCESS_KEY_ID".into(), value: access_key.clone()},
                VariableOutput{name: "AWS_SECRET_ACCESS_KEY".into(), value: secret_key.clone()},
                VariableOutput{name: "AWS_SESSION_TOKEN".into(), value: session_token.clone()} 
            ]
        )
    }
}
