use crate::core::NovopsContext;

use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use url::Url;
use async_trait::async_trait;
use std::collections::HashMap;
use vaultrs::kv2;


#[async_trait]
pub trait HashivaultClient {
    async fn kv2_read(&self, mount: Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error>;
}

pub struct DefaultHashivaultClient{
    client: VaultClient
}

pub struct DryRunHashivaultClient{}

#[async_trait]
impl HashivaultClient for DefaultHashivaultClient {
    async fn kv2_read(&self, mount: Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error>{

        // retrieve secret using "secret" mount by default
        let secret_data: HashMap<String, String> = kv2::read(
            &self.client, 
            mount.unwrap_or("secret".to_string()).as_str(), 
            path
        ).await?;

        Ok(secret_data.get(key).unwrap().clone())
    }
}


#[async_trait]
impl HashivaultClient for DryRunHashivaultClient {
    async fn kv2_read(&self, _mount: Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error>{

        let mut result = "RESULT:".to_string();
        result.push_str(format!("{:}/{:}", path, key).as_str());

        Ok(result)
    }
}


pub fn get_client(ctx: &NovopsContext) -> Box<dyn HashivaultClient + Send + Sync> {
    if ctx.dry_run {
        return Box::new(DryRunHashivaultClient{})
    } else {
        let client = build_client(ctx);
        return Box::new(DefaultHashivaultClient{
            client: client
        })
    }
    
}

pub fn build_client(ctx: &NovopsContext) -> VaultClient {

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