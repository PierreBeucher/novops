use anyhow::Context;
use async_trait::async_trait;

use crate::core::NovopsContext;
use azure_security_keyvault::prelude::{KeyVaultGetSecretResponse, KeyVaultGetSecretResponseAttributes};
use azure_identity::DefaultAzureCredential;
use azure_security_keyvault::KeyvaultClient;
use time::OffsetDateTime;

#[async_trait]
pub trait AzureClient {
   async fn get_keyvault_secret(&self, vault: &str, name: &str, version: &Option<String>) -> Result<KeyVaultGetSecretResponse, anyhow::Error>;
}

pub struct DefaultAzureClient {}
pub struct DryRunAzureClient {}

#[async_trait]
impl AzureClient for DefaultAzureClient{


    async fn get_keyvault_secret(&self, vault: &str, name: &str, version: &Option<String>) -> Result<KeyVaultGetSecretResponse, anyhow::Error> {

        let credential = DefaultAzureCredential::default();
        let url = &format!("https://{}.vault.azure.net", vault);
        let client = KeyvaultClient::new(&url, std::sync::Arc::new(credential))
            .with_context(|| format!("Couldn't create Azure Vault client for {:}", url))?
            .secret_client();

        let secret = client.get(name).version(version.clone().unwrap_or_default()).await?;
        Ok(secret)
    }
}

#[async_trait]
impl AzureClient for DryRunAzureClient{


    async fn get_keyvault_secret(&self, vault: &str, name: &str, version: &Option<String>) -> Result<KeyVaultGetSecretResponse, anyhow::Error> {
        let dummy_value = format!("RESULT:{}/{}/{}", vault, name, version.clone().unwrap_or_default());

        let result = KeyVaultGetSecretResponse {
            value: dummy_value,
            id: String::default(),
            attributes: KeyVaultGetSecretResponseAttributes {
                enabled: bool::default(),
                expires_on: None,
                created_on: OffsetDateTime::now_utc(),
                updated_on: OffsetDateTime::now_utc(),
                recovery_level: String::default()

            }
        };
        Ok(result)
    }
}

pub fn get_client(ctx: &NovopsContext) -> Box<dyn AzureClient + Send + Sync> {
    if ctx.dry_run {
        return Box::new(DryRunAzureClient{})
    } else {
        return Box::new(DefaultAzureClient{})
    }
}