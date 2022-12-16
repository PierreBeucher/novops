use serde::Deserialize;
use async_trait::async_trait;
use anyhow;
use schemars::JsonSchema;
use std::default::Default;

use crate::core::{ResolveTo, NovopsContext};
use crate::modules::aws::config::{build_mutable_client_config_from_context, get_ssm_client};

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AwsSSMParamStoreInput {
    
    pub aws_ssm_parameter: AwsSSMParameter
}

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct AwsSSMParameter {
    
    /**
     * Parameter name
     */
    pub name: String,

    /**
     * Return decrypted values for secure string parameters. This flag is ignored for String and StringList parameter types.
     */
    pub with_decryption: Option<bool>
}


#[async_trait]
impl ResolveTo<String> for AwsSSMParamStoreInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

        let client_conf = build_mutable_client_config_from_context(ctx);
        let ssm_client = get_ssm_client(&client_conf).await?;

        let result = ssm_client.get_parameter()
            .name(&self.aws_ssm_parameter.name)
            .with_decryption(self.aws_ssm_parameter.with_decryption.unwrap_or_default())
            .send().await?;
        
        let value = result.parameter().unwrap().value().unwrap();
        return Ok(value.to_string())
    }
}