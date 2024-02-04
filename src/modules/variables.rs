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
    pub value: StringResolvableInput,

    // /// Whether to export variable.
    // /// By default, variable output are exported with 'export VAR=value',
    // /// setting false will generate output without 'export' keywork
    // pub export: Option<bool>,

    /// Method used to quote variables using either single quote (`'`) or double quotes (`"`).
    /// 
    /// Variables are wrapped between single quotes by default
    /// to prevent any interpolation of special characters. For example
    /// a string with special characters `$"!<>_'` will be exported as 
    /// `export MYVAR='$"!<>'"'"`). Only single quote character `'` 
    /// is escaped as `"'"` so that it's interpreted as a single quote.
    /// 
    /// With double quotes, the same principle applies except double quotes `"`
    /// are exported as `'"'`. While it may be practical in some situations to 
    /// leverage interpolation, it may represent a risk as interpolation can cause unwanted behavior. 
    /// For example string `(rm /tmp/important_file)` would be loaded as 
    /// `export MYVAR="$(rm /tmp/important_file)"`.
    /// 
    /// Only use double quote from trusted source. 
    pub quote_method: Option<String>
}
    
/**
 * Output for VariableInput, with final name and value
 */
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub struct VariableOutput {
    pub name: String,
    pub value: String,
    pub quote_method: Option<String>
}

#[async_trait]
impl ResolveTo<VariableOutput> for VariableInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<VariableOutput, anyhow::Error> {
        let v = match self.value.resolve(ctx).await {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        return Ok(
            VariableOutput { 
                name: self.name.clone(), 
                value: v,
                quote_method: self.quote_method.clone(),
            }
        )
    }
}