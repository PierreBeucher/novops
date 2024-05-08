use crate::core::NovopsContext;

use anyhow::Context;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder };
use url::Url;
use async_trait::async_trait;
use std::{ collections::HashMap, env, fs, path::{Path, PathBuf} };
use vaultrs::{kv2, kv1, aws, api::aws::requests::GenerateCredentialsRequest};
use log::debug;
use home;

#[async_trait]
pub trait HashivaultClient {
    async fn kv2_read(&self, 
        mount: &Option<String>, 
        path: &str, 
        key: &str
    ) -> Result<String, anyhow::Error>;
    
    async fn kv1_read(&self, 
        mount: &Option<String>, 
        path: &str, 
        key: &str
    ) -> Result<String, anyhow::Error>;
    
    async fn aws_creds(&self, 
        mount: &Option<String>, 
        role: &str, 
        role_arn: &Option<String>,
        role_session_name: &Option<String>,
        ttl: &Option<String>
    ) -> Result<Creds, anyhow::Error>;
}

pub struct DefaultHashivaultClient{
    client: VaultClient
}

pub struct DryRunHashivaultClient{}

pub struct Creds{
    pub access_key: String,
    pub secret_key: String,
    pub security_token: Option<String>,
    pub arn: String
}

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
            .cloned()
    }

    async fn kv1_read(&self, mount: &Option<String>, path: &str, key: &str) -> Result<String, anyhow::Error> {
        let _mount = mount.clone().unwrap_or("secret".to_string());
        let secret_data: HashMap<String, String> = kv1::get(
            &self.client, 
            mount.clone().unwrap_or("secret".to_string()).as_str(), 
            path
        ).await.with_context(|| format!("Error reading '{:}' mount at path '{:}'", &_mount, &path))?;

        return secret_data.get(key)
            .ok_or_else(|| anyhow::anyhow!("Mount '{:}' secret '{:}', found but key '{:}' did not exist", &_mount, &path, &key))
            .cloned()
    }

    async fn aws_creds (&self, mount: &Option<String>, role: &str, 
        role_arn: &Option<String>, role_session_name: &Option<String>, ttl: &Option<String>
    ) -> Result<Creds, anyhow::Error>{

        let mut opts = GenerateCredentialsRequest::builder();

        if role_arn.is_some() {
            opts.role_arn(role_arn.clone().unwrap().to_string());
        }

        if role_session_name.is_some() {
            opts.role_session_name(role_session_name.clone().unwrap().to_string());
        }

        if ttl.is_some() {
            opts.ttl(ttl.clone().unwrap().to_string());
        }

        let result = aws::roles::credentials(&self.client, 
            &mount.clone().unwrap_or("aws".to_string()), 
            role, 
            Some(&mut opts)
        ).await.with_context(|| format!("Couldn't generate Hashivault AWS creds for {:}", role))?;

        Ok(Creds {
            access_key: result.access_key,
            secret_key: result.secret_key,
            security_token: result.security_token,
            arn: result.arn
        })

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

    async fn aws_creds (&self, _mount: &Option<String>, role: &str, 
        _role_arn: &Option<String>, role_session_name: &Option<String>, _ttl: &Option<String>
    ) -> Result<Creds, anyhow::Error>{

        // use role to generate dummy role-arn is not passed
        let session_arn = role_session_name.clone().map_or(
            format!("arn:aws:sts::123456789012:assumed-role/{:}/{:}", role, "dummy-session"), 
            |r| format!("arn:aws:sts::123456789012:assumed-role/{:}/{:}", role, r), );
        
        let result = Creds {
            access_key: "AKIADRYRUNACCESSKEY".to_string(),
            secret_key: "s3cret".to_string(),
            security_token: Some("securityToken".to_string()),
            arn: session_arn
        };

        Ok(result)
    }
}


pub fn get_client(ctx: &NovopsContext) -> Result<Box<dyn HashivaultClient + Send + Sync>, anyhow::Error> {
    if ctx.dry_run {
        Ok(Box::new(DryRunHashivaultClient{}))
    } else {
        let client = build_client(ctx)
            .with_context(|| "Couldn't build Hashivault client")?;
        Ok(Box::new(DefaultHashivaultClient{
            client
        }))
    }
    
}

pub fn build_client(ctx: &NovopsContext) -> Result<VaultClient, anyhow::Error> {

    let default_settings = VaultClientSettingsBuilder::default().build()?;

    let hv_config = ctx.config_file_data.config.clone()
        .unwrap_or_default()
        .hashivault.unwrap_or_default();
    
    let token_var = env::var("VAULT_TOKEN").ok();
    let addr_var = env::var("VAULT_ADDR").ok();
    let home_var = home::home_dir();

    let vault_token = load_vault_token(ctx, home_var, token_var).with_context(|| "Couldn't load Vault token")?;
    let vault_url = load_vault_address(ctx, addr_var).with_context(|| "Couldn't load Vault address")?;

    let settings = VaultClientSettingsBuilder::default()
        .address(vault_url)
        .token(vault_token)
        .verify(hv_config.verify.unwrap_or(default_settings.verify))
        .build()?;
    
    let client = VaultClient::new(settings)?;
    
    Ok(client)
}

/// Look for token address in this order:
/// - VAULT_TOKEN env var
/// - Novops config (file)
/// - Novops config (plain)
/// - $HOME/.vault-token
/// - default settings
pub fn load_vault_token(ctx: &NovopsContext, home_var: Option<PathBuf>, token_var: Option<String>) -> Result<String, anyhow::Error>  {

    debug!("Looking for a vault token...");
    
    if token_var.is_some() {
        debug!("Found VAULT_TOKEN variable, using it.");
        return Ok(token_var.unwrap())
    }

    let hvault_config = &ctx.clone().config_file_data.config.unwrap_or_default()
        .hashivault.unwrap_or_default();

    let token_path = &hvault_config.token_path;
    if token_path.is_some(){
        debug!("Found token path config, reading file and using value as token...");

        let token = read_vault_token_file(&token_path.clone().unwrap())?;
        return Ok(token)
    }

    let token_plain = &hvault_config.token;

    if token_plain.is_some(){
        debug!("Found plain token in config, using its value as token...");

        return Ok(token_plain.clone().unwrap())
    }

    if home_var.is_some() {
        let home_token_path = Path::new(&home_var.unwrap()).join(".vault-token");

        debug!("Checking vault token file exists at {:?}", home_token_path);

        if home_token_path.exists() {
            debug!("Using vault token file {:?}", home_token_path);

            let token = read_vault_token_file(&home_token_path)
                .with_context(|| format!("Found a token file '{:?}' in HOME but couldn't read", &home_token_path))?;

            return Ok(token)
        }
    }

    debug!("No vault token found, returning empty string...");

    Ok(VaultClientSettingsBuilder::default().build()?.token)

}

/// Look for vault address in this order:
/// - VAULT_ADDR env var
/// - Novops config
/// - default settings
pub fn load_vault_address(ctx: &NovopsContext, addr_var: Option<String>) -> Result<Url, anyhow::Error>  {

    debug!("Looking for a vault address...");

    if addr_var.is_some() {
        debug!("Found VAULT_ADDR variable, using it.");

        return parse_vault_addr_str(addr_var.unwrap());
    }

    let hvault_config = &ctx.clone().config_file_data.config.unwrap_or_default()
        .hashivault.unwrap_or_default();

    let addr_config = &hvault_config.address;
    if addr_config.is_some(){
        debug!("Found vault address config, using it.");

        let addr = parse_vault_addr_str(addr_config.clone().unwrap())
            .with_context(|| format!("Couldn't parse address as URl: '{:?}'", addr_config))?;

        debug!("Returning vault address from config...");

        return Ok(addr);
    }

    debug!("No vault address found, returning empty string...");

    Ok(VaultClientSettingsBuilder::default().build()?.address)

}

fn parse_vault_addr_str(vault_addr_str: String) -> Result<Url, anyhow::Error> {
    let vault_addr = Url::parse(&vault_addr_str)
        .with_context(|| format!("Couldn't parse Vault URL '{:?}'", &vault_addr_str))?;
    Ok(vault_addr)
}

fn read_vault_token_file(token_path: &PathBuf) -> Result<String, anyhow::Error>{
    let token = fs::read_to_string(token_path)
        .with_context(|| format!("Couldn't read token file at {:?}", token_path))?;

    // trim result as empty spaces or linefeed causes vaultrs to panic
    return Ok(String::from(token.trim()))
}