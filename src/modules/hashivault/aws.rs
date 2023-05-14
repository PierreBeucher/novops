use crate::core::{ResolveTo, NovopsContext};
use super::client::get_client;
use crate::modules::variables::VariableOutput;

use anyhow::Context;
use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultAWSInput {
    pub mount: Option<String>,
    pub name: String,
    pub role_arn: Option<String>,
    pub role_session_name: Option<String>,
    pub ttl: Option<String>
}

#[async_trait]
impl ResolveTo<Vec<VariableOutput>> for HashiVaultAWSInput {
  async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<VariableOutput>, anyhow::Error> {
    
    let client = get_client(ctx)?;

    let creds = client.aws_creds(
      &Some(self.mount.clone().unwrap_or("aws".to_string())), 
      &self.name,
      &self.role_arn,
      &self.role_session_name,
      &self.ttl
    )
    .await.with_context(|| format!("Couldn't generate Hashivault AWS credentials for {:}", self.name))?;

    let mut result = vec![
      VariableOutput{ name: "AWS_ACCESS_KEY_ID".to_string(), value: creds.access_key },
      VariableOutput{ name: "AWS_SECRET_ACCESS_KEY".to_string(), value: creds.secret_key }
    ];

    if creds.security_token.is_some() {
      result.push(VariableOutput{ name: "AWS_SESSION_TOKEN".to_string(), value: creds.security_token.unwrap() })
    }

    Ok(result)
  }
}
