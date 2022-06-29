use serde::Deserialize;
use async_trait::async_trait;
use crate::novops::{ResolveTo, AnyStringInput, NovopsContext};

/**
 * An environment variable (key / value)
 */
#[derive(Debug, Deserialize, Clone)]
pub struct VariableInput {
    name: String,
    value: AnyStringInput
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
    async fn resolve(&self, ctx: &NovopsContext) -> VariableOutput {
        return VariableOutput { 
            name: self.name.clone(), 
            value: self.value.resolve(ctx).await
        }
    }
}