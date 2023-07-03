use crate::core::{ResolveTo, NovopsContext};

use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;
use super::client::get_client;

/// Reference a Key Value V1 secret
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultKeyValueV1Input {
  hvault_kv1: HashiVaultKeyValueV1
}

/// Reference a Key Value V1 secret
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultKeyValueV1 {
    /// KV v1 mount point
    /// 
    /// default to "kv/"
    pub mount: Option<String>,

    /// Path to secret
    pub path: String,

    /// Secret key to retrieve
    pub key: String
}

#[async_trait]
impl ResolveTo<String> for HashiVaultKeyValueV1Input {
  async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {
    
    let client = get_client(ctx)?;
    let kv1 = &self.hvault_kv1;

    // retrieve secret using "secret" mount by default
    let result = client.kv1_read(&kv1.mount, &kv1.path, &kv1.key).await?;
    Ok(result)
  }
}