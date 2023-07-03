use serde::Deserialize;
use async_trait::async_trait;
use anyhow;
use schemars::JsonSchema;
use std::default::Default;
use crate::core::{ResolveTo, NovopsContext};
use crate::modules::aws::client::get_client;

/// Reference an SSM Parameter config or secret
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsSSMParamStoreInput {
    
    pub aws_ssm_parameter: AwsSSMParameter
}

/// Reference an SSM Parameter config or secret
/// 
/// Maps directly to GetParameter API. See https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_GetParameter.html
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct AwsSSMParameter {
    
    /// Parameter name
    pub name: String,

    /// Return decrypted values for secure string parameters. This flag is ignored for String and StringList parameter types.
    pub with_decryption: Option<bool>
}


#[async_trait]
impl ResolveTo<String> for AwsSSMParamStoreInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

        let client = get_client(ctx).await;

        let result = client.get_ssm_parameter(
            &self.aws_ssm_parameter.name, 
            self.aws_ssm_parameter.with_decryption
        ).await?;
        
        let value = result.parameter().ok_or(anyhow::anyhow!("Couldn't unwrap parameter object"))?
            .value().ok_or(anyhow::anyhow!("Couldn't unwrap parameter value"))?
            .to_string();
        
        Ok(value)
    }
}