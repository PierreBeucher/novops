use serde::Deserialize;
use async_trait::async_trait;
use crate::novops::{ResolveTo, StringResolvableInput, NovopsContext};
use anyhow;

/**
 * An environment variable (key / value)
 */
#[derive(Debug, Deserialize, Clone)]
pub struct VariableInput {
    name: String,
    value: StringResolvableInput
}
    
/**
 * Output for VariableInput, with final name and value
 */
#[derive(Debug, Clone)]
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