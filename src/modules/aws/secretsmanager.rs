use anyhow::Context;
use aws_sdk_secretsmanager::output::GetSecretValueOutput;
use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;
use std::default::Default;

use crate::core::{ResolveTo, NovopsContext};
use crate::modules::aws::client::get_client;

/// Reference an AWS Secret Manager secret
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsSecretsManagerSecretInput {
    
    pub aws_secret: AwsSecretsManagerSecret
}


/// Structure to request a Secrets Manager secret
/// 
/// Maps directly to GetSecretValue API. See https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_GetSecretValue.html
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct AwsSecretsManagerSecret {
    
    /// Secret ID
    pub id: String,

    
    /// The unique identifier of the version of the secret to retrieve. 
    pub version_id: Option<String>,

    
    /// The staging label of the version of the secret to retrieve.
    pub version_stage: Option<String>,
}

#[async_trait]
impl ResolveTo<Vec<u8>> for AwsSecretsManagerSecretInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<u8>, anyhow::Error> {

        let output = retrieve_secret(ctx, self).await
            .with_context(|| format!("Couldn't retrieve secret {:}", &self.aws_secret.id))?;

        return if let Some(s) =  output.secret_string() {
            Ok(s.to_string().into_bytes())
        } else if let Some(s) = output.secret_binary() {
            Ok(s.clone().into_inner())
        } else {
            Err(anyhow::format_err!("Secret value was neither string nor binary, got response: {:?}", output))
        }
    }
}

#[async_trait]
impl ResolveTo<String> for AwsSecretsManagerSecretInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

        let output = retrieve_secret(ctx, self).await?;

        return if let Some(s) = output.secret_string() {
            Ok(s.to_string())
        } else if let Some (s) = output.secret_binary() {
            let binary = s.clone().into_inner();
            let result = String::from_utf8(binary)
                .with_context(|| format!("Couldn't convert bytes from Secrets Manager secret '{}' to UTF-8 String. \
                Non-UTF-8 binary data can't be used as Variable input yet. Either use File input for binary data or make sure it's a valid UTF-8 string.", self.aws_secret.id))?;
            Ok(result)
        } else {
            Err(anyhow::format_err!("Secret value was neither string nor binary, got response: {:?}", output))
        };
        
    }
}

async fn retrieve_secret(ctx: &NovopsContext, input: &AwsSecretsManagerSecretInput) -> Result<GetSecretValueOutput, anyhow::Error>{
    let client = get_client(ctx).await;

    let output = client.get_secret_value(
        input.aws_secret.id.as_str(), 
        input.aws_secret.version_id.clone(), 
        input.aws_secret.version_stage.clone()
    ).await?;
    
    Ok(output)
}
