use anyhow::Context;
use aws_sdk_secretsmanager::output::GetSecretValueOutput;
use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;
use std::default::Default;

use crate::core::{ResolveTo, NovopsContext};
use crate::modules::aws::config::{build_mutable_client_config_from_context, get_secretsmanager_client};

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsSecretsManagerSecretInput {
    
    pub aws_secret: AwsSecretsManagerSecret
}

/**
 * Structure to request a Secrets Manager secret
 * 
 * Secret Manager content is either String or Binary so this Input implements 
 * ResolveTo<String> and ResolveTo<Vec<u8>> to represent both situations
 * 
 * Depending on usage, resolving behavior is:
 * - String for VariableInput: String is used for export
 * - Binary for VariableInput: **Vec<u8> is encoded in UTF-8 for export**
 * - String for FileInput: String's underlying Vec<u8> is written to file
 * - Binary for FileInput: Vec<u8> data is written to file as-is
 * 
 * See https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_GetSecretValue.html
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct AwsSecretsManagerSecret {
    
    /**
     * Secret ID
     */
    pub id: String,

    /**
     * The unique identifier of the version of the secret to retrieve. 
     */
    pub version_id: Option<String>,

    /**
     * The staging label of the version of the secret to retrieve.
     */
    pub version_stage: Option<String>,
}

#[async_trait]
impl ResolveTo<Vec<u8>> for AwsSecretsManagerSecretInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<u8>, anyhow::Error> {

        let output = retrieve_secret(ctx, self).await?;

        if output.secret_string().is_some(){
            return Ok(output.secret_string().unwrap().to_string().into_bytes());
        }

        if output.secret_binary().is_some(){
            return Ok(output.secret_binary().unwrap().clone().into_inner());
        }

        return Err(anyhow::format_err!("Secret value was neither string nor binary, got response: {:?}", output));
        
    }
}

#[async_trait]
impl ResolveTo<String> for AwsSecretsManagerSecretInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

        let output = retrieve_secret(ctx, self).await?;

        if output.secret_string().is_some(){
            return Ok(output.secret_string().unwrap().to_string());
        }

        if output.secret_binary().is_some(){
            let binary = output.secret_binary().unwrap().clone().into_inner();
            let result = String::from_utf8(binary)
                .with_context(|| format!("Couldn't convert bytes from Secrets Manager secret '{}' to UTF-8 String. \
                Non-UTF-8 binary data can't be used as Variable input yet. Either use File input for binary data or make sure it's a valid UTF-8 string.", self.aws_secret.id))?;
            return Ok(result);
        }

        return Err(anyhow::format_err!("Secret value was neither string nor binary, got response: {:?}", output));        
        
    }
}

async fn retrieve_secret(ctx: &NovopsContext, input: &AwsSecretsManagerSecretInput) -> Result<GetSecretValueOutput, anyhow::Error>{
    let client_conf = build_mutable_client_config_from_context(ctx);
    let ssm_client = get_secretsmanager_client(&client_conf).await?;

    let output = ssm_client.get_secret_value()
        .secret_id(input.aws_secret.id.clone())
        .set_version_id(input.aws_secret.version_id.clone())
        .set_version_stage(input.aws_secret.version_stage.clone())
        .send().await?;

    return Ok(output);
}
