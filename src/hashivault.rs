use crate::core::{ResolveTo, NovopsContext};

use serde::Deserialize;
use async_trait::async_trait;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::kv2;
use std::collections::HashMap;
use url::Url;
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

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]  
pub struct HashivaultConfig {
  /// Address in form http(s)://HOST:PORT
  address: Option<String>,

  /// Token used to connect
  token: Option<String>,

  /// TLS verify
  verify: Option<bool>
}

impl Default for HashivaultConfig {
  fn default() -> HashivaultConfig {
    HashivaultConfig{
      address: None,
      token: None,
      verify: None
    }
  }
}

#[async_trait]
impl ResolveTo<String> for HashiVaultKeyValueV2Input {
  async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {
    
    let client = build_vault_client(ctx);
    let kv2 = self.hvault_kv2.clone();

    // retrieve secret using "secret" mount by default
    let secret_data: HashMap<String, String> = kv2::read(
      &client, 
      &kv2.mount.unwrap_or("secret".to_string()), 
      &kv2.path
    ).await?;
    let result = secret_data.get(&kv2.key).unwrap().clone();

    return Ok(result);
  }
}

pub fn build_vault_client(ctx: &NovopsContext) -> VaultClient {

  let hv_config = ctx.config_file_data.config.clone()
    .unwrap_or_default()
    .hashivault.unwrap_or_default();

  let default_settings = VaultClientSettingsBuilder::default().build().unwrap();

  let settings = VaultClientSettingsBuilder::default()
    .address(Url::parse(
      &hv_config.address.unwrap_or(default_settings.address.to_string())
    ).unwrap())
    .token(hv_config.token.unwrap_or(default_settings.token))
    .verify(hv_config.verify.unwrap_or(default_settings.verify))
    .build().unwrap();
  
  return VaultClient::new(settings).unwrap();
}