use crate::core::{ResolveTo, NovopsContext};
use super::client::get_client;

use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultKeyValueV2Input {
  hvault_kv2: HashiVaultKeyValueV2
}

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultKeyValueV2 {
    /// KV v2 mount point
    /// default to "secret/"
    pub mount: Option<String>,

    /// Path to secret
    pub path: String,

    /// Secret key to retrieve
    pub key: String
}

#[async_trait]
impl ResolveTo<String> for HashiVaultKeyValueV2Input {
  async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {
    
    let client = get_client(ctx)?;
    let result = client.kv2_read(
        &self.hvault_kv2.mount, 
        &self.hvault_kv2.path, 
        &self.hvault_kv2.key
    ).await?;

    Ok(result)
  }
}
