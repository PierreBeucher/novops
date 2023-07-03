use serde::Deserialize;
use async_trait::async_trait;
use crate::core::{ResolveTo, StringResolvableInput, NovopsContext};
use anyhow;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct VariableInput {
    /// Environment variable name, such as `NPM_TOKEN`
    pub name: String,

    /// Source of truth for variable 
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