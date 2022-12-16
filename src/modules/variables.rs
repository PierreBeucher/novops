use serde::Deserialize;
use async_trait::async_trait;
use crate::core::{ResolveTo, StringResolvableInput, NovopsContext};
use anyhow;
use schemars::JsonSchema;

/**
 * An environment variable (key / value)
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct VariableInput {
    pub name: String,
    pub value: StringResolvableInput
}
    
/**
 * Output for VariableInput, with final name and value
 */
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub struct VariableOutput {
    pub name: String,
    pub value: String
}

#[async_trait]
impl ResolveTo<VariableOutput> for VariableInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<VariableOutput, anyhow::Error> {
        let value = match self.value.resolve(ctx).await {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        return Ok(
            VariableOutput { 
                name: self.name.clone(), 
                value: value
            }
        )
    }
}