use crate::core::NovopsContext;

use anyhow::Context;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use url::Url;
use async_trait::async_trait;
use std::collections::HashMap;
use vaultrs::{kv2, kv1};


#[async_trait]
pub trait HashivaultClient {
    async fn kv2_read(&self, mount: &Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error>;
    async fn kv1_read(&self, mount: &Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error>;
}

pub struct DefaultHashivaultClient{
    client: VaultClient
}

pub struct DryRunHashivaultClient{}

#[async_trait]
impl HashivaultClient for DefaultHashivaultClient {
    async fn kv2_read(&self, mount: &Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error>{

        // retrieve secret using "secret" mount by default
        let _mount = mount.clone().unwrap_or("secret".to_string());
        let secret_data: HashMap<String, String> = kv2::read(
            &self.client, 
            &_mount, 
            path
        ).await.with_context(|| format!("Error reading '{:}' mount at path '{:}'", &_mount, &path))?;

        return secret_data.get(key)
            .ok_or_else(|| anyhow::anyhow!("Mount '{:}' secret '{:}' found but key '{:}' did not exist", &_mount, &path, &key))
            .map(|s| s.clone());
    }

    async fn kv1_read(&self, mount: &Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error> {
        let _mount = mount.clone().unwrap_or("secret".to_string());
        let secret_data: HashMap<String, String> = kv1::get(
            &self.client, 
            mount.clone().unwrap_or("secret".to_string()).as_str(), 
            &path
        ).await.with_context(|| format!("Error reading '{:}' mount at path '{:}'", &_mount, &path))?;

        return secret_data.get(key)
            .ok_or_else(|| anyhow::anyhow!("Mount '{:}' secret '{:}', found but key '{:}' did not exist", &_mount, &path, &key))
            .map(|s| s.clone());
    }

}


#[async_trait]
impl HashivaultClient for DryRunHashivaultClient {
    async fn kv2_read(&self, _mount: &Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error>{

        let mut result = "RESULT:".to_string();
        result.push_str(format!("{:}/{:}", path, key).as_str());

        Ok(result)
    }

    async fn kv1_read(&self, _mount: &Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error>{

        let mut result = "RESULT:".to_string();
        result.push_str(format!("{:}/{:}", path, key).as_str());

        Ok(result)
    }
}


pub fn get_client(ctx: &NovopsContext) -> Result<Box<dyn HashivaultClient + Send + Sync>, anyhow::Error> {
    if ctx.dry_run {
        return Ok(Box::new(DryRunHashivaultClient{}))
    } else {
        let client = build_client(ctx)
            .with_context(|| "Couldn't build Hashivault client")?;
        return Ok(Box::new(DefaultHashivaultClient{
            client: client
        }))
    }
    
}

pub fn build_client(ctx: &NovopsContext) -> Result<VaultClient, anyhow::Error> {

    let hv_config = ctx.config_file_data.config.clone()
        .unwrap_or_default()
        .hashivault.unwrap_or_default();

    let default_settings = VaultClientSettingsBuilder::default().build()?;

    let vault_url_string = hv_config.address.unwrap_or(default_settings.address.to_string());
    let vault_url = Url::parse(&vault_url_string)
        .with_context(|| format!("Couldn't parse Vault URL '{:?}'", &vault_url_string))?;

    let settings = VaultClientSettingsBuilder::default()
        .address(vault_url)
        .token(hv_config.token.unwrap_or(default_settings.token))
        .verify(hv_config.verify.unwrap_or(default_settings.verify))
        .build()?;
    
    let client = VaultClient::new(settings)?;
    
    Ok(client)
}