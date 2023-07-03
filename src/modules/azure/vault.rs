use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;
use std::default::Default;

use crate::core::{ResolveTo, NovopsContext};

use super::client::get_client;

/// Reference an Azure Keyvault secret
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct AzureKeyvaultSecretInput {
    
    pub azure_keyvault_secret: AzureKeyvaultSecret
}


/// Maps directly to Keyvault Get Secret API
/// 
/// See https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secret/get-secret?tabs=HTTP
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct AzureKeyvaultSecret {
    
    /// Secret's vault name
    pub vault: String,
    
    /// Secret name
    pub name: String,

    /// Secret's version (default: latest)
    pub version: Option<String>,
}

#[async_trait]
impl ResolveTo<String> for AzureKeyvaultSecretInput {
    
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

        let s = self.azure_keyvault_secret.clone();
        let client = get_client(ctx);
        
        let result = client.get_keyvault_secret(&s.vault, &s.name, &s.version).await?;
        Ok(result.value)
    }

}

