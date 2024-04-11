use crate::core::NovopsContext;

use anyhow::{Context, Error};
use vaultrs::client::{Client, VaultClient, VaultClientSettingsBuilder};
use url::Url;
use async_trait::async_trait;
use std::{ collections::HashMap, env, fs, path::{Path, PathBuf}, time::Duration };
use std::env::VarError;
use vaultrs::{kv2, kv1, aws, auth, api::aws::requests::GenerateCredentialsRequest};
use log::debug;
use home;
use crate::modules::hashivault::config::{HashiVaultAuth};


const KUBERNETES_SA_JWT_PATH: &str = "/var/run/secrets/kubernetes.io/serviceaccount/token";
const KUBERNETES_SA_JWT_PATH_ENV: &str = "KUBERNETES_SA_JWT_PATH";
const VAULT_AUTH_MOUNT_PATH_ENV: &str = "VAULT_AUTH_MOUNT_PATH";
const VAULT_AUTH_ROLE_ENV: &str = "VAULT_AUTH_ROLE";
const VAULT_AUTH_JWT_TOKEN_ENV: &str = "VAULT_AUTH_JWT_TOKEN";
const VAULT_AUTH_SECRET_ID_ENV: &str = "VAULT_AUTH_SECRET_ID";
const VAULT_AUTH_ROLE_ID_ENV: &str = "VAULT_AUTH_ROLE_ID";


/// Default client timeout (seconds)
const CLIENT_TIMEOUT_DEFAULT : u64 = 60;

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


pub async fn get_client(ctx: &NovopsContext) -> Result<Box<dyn HashivaultClient + Send + Sync>, anyhow::Error> {
    if ctx.dry_run {
        Ok(Box::new(DryRunHashivaultClient{}))
    } else {
        let client = build_client(ctx).await
            .with_context(|| "Couldn't build Hashivault client")?;
        Ok(Box::new(DefaultHashivaultClient{
            client
        }))
    }
    
}

pub async fn build_client(ctx: &NovopsContext) -> Result<VaultClient, anyhow::Error> {
    let default_settings = VaultClientSettingsBuilder::default().build()?;

    let hv_config = ctx.config_file_data.config.clone()
        .unwrap_or_default()
        .hashivault.unwrap_or_default();
    
    let token_var = env::var("VAULT_TOKEN").ok();
    let addr_var = env::var("VAULT_ADDR").ok();
    let home_var = home::home_dir();

    let vault_url = load_vault_address(ctx, addr_var).with_context(|| "Couldn't load Vault address")?;

    let timeout = hv_config.timeout.unwrap_or(CLIENT_TIMEOUT_DEFAULT);

    let settings = VaultClientSettingsBuilder::default()
        .address(vault_url)
        .timeout(Some(Duration::new(timeout, 0)))
        .verify(hv_config.verify.unwrap_or(default_settings.verify))
        .build()?;

    debug!("Using Vault client timeout: {:?}", settings.timeout);

    let mut client = VaultClient::new(settings)?;

    if let Some(auth) = hv_config.auth {
        debug!("Found Vault authentication configuration {:?}", auth);

        vault_login(&mut client, auth).await?;
    } else {
        let token = load_vault_token(ctx, home_var, token_var).with_context(|| "Couldn't load Vault token")?;
        client.set_token(&token);
    }
    
    Ok(client)
}

/// Log in on Vault using the provided authentication backend
async fn vault_login(client: &mut VaultClient, auth: HashiVaultAuth) -> Result<(), Error> {
    let auth_info = match auth {
        HashiVaultAuth::Kubernetes { mount_path, role } => {
            let mount_path = unwrap_or_env(mount_path, VAULT_AUTH_MOUNT_PATH_ENV, "Failed to read Vault auth mount path environment variable")?;
            let role = unwrap_or_env(role, VAULT_AUTH_ROLE_ENV, "Failed to read Vault auth role environment variable")?;

            debug!("Logging in on Vault using Kubernetes authentication with mount path '{}' and role '{}'", mount_path, role);

            let jwt_file_path = env_or_default(KUBERNETES_SA_JWT_PATH_ENV, KUBERNETES_SA_JWT_PATH)?;
            let jwt = fs::read_to_string(&jwt_file_path)
                .with_context(|| format!("Could not read Service Account token at path {}", jwt_file_path))?;

            auth::kubernetes::login(client, &mount_path, &role, &jwt).await
                .with_context(|| format!("Failed Vault Kubernetes log in using mount path '{}' and role '{}'", mount_path, role))?
        }
        HashiVaultAuth::AppRole { mount_path, role_id, secret_id_path } => {
            let mount_path = unwrap_or_env(mount_path, VAULT_AUTH_MOUNT_PATH_ENV, "Failed to read Vault auth mount path environment variable")?;
            let role_id = unwrap_or_env(role_id, VAULT_AUTH_ROLE_ID_ENV, "Failed to read Vault auth role id environment variable")?;

            debug!("Logging in on Vault using AppRole authentication with mount path '{}'", mount_path);

            let secret_id = if let Some(path) = secret_id_path {
                fs::read_to_string(path.clone()).with_context(|| format!("Failed to secret id from path '{}'", path))?
            } else {
                match env::var(VAULT_AUTH_SECRET_ID_ENV) {
                    Ok(v) => v,
                    Err(env::VarError::NotPresent) => {
                        debug!("Not informed Secret ID by environment variable '{}'", VAULT_AUTH_SECRET_ID_ENV);
                        String::new()
                    },
                    Err(e) => Err(e).with_context(|| format!("Failed to read secret id from env '{}'", VAULT_AUTH_SECRET_ID_ENV))?
                }
            };

            auth::approle::login(client, &mount_path, &role_id, &secret_id).await
                .with_context(|| format!("Failed Vault AppRole log in using mount path '{}'", mount_path))?
        }
        HashiVaultAuth::JWT { mount_path, token_path, role } => {
            let mount_path = unwrap_or_env(mount_path, VAULT_AUTH_MOUNT_PATH_ENV, "Failed to read Vault auth mount path environment variable")?;
            let role = unwrap_or_env(role, VAULT_AUTH_ROLE_ENV, "Failed to read Vault auth role environment variable")?;
            
            debug!("Logging in on Vault using JWT authentication with mount path '{}' and role '{}'", mount_path, role);

            let token = if let Some(path) = token_path {
                fs::read_to_string(path.clone()).with_context(|| format!("Failed to read JWT token from path '{}'", path))?
            } else {
                env::var(VAULT_AUTH_JWT_TOKEN_ENV).with_context(|| format!("Failed to read JWT token from env '{}'", VAULT_AUTH_JWT_TOKEN_ENV))?
            };

            auth::oidc::login(client, &mount_path, &token, Some(role.clone())).await
                .with_context(|| format!("Failed Vault JWT log in using mount path '{}' and role '{}'", mount_path, role))?
        }
    };

    client.set_token(&auth_info.client_token);

    debug!("Success on Vault logging");

    Ok(())
}

/// Unwrap the variable or try to get the value from a environment variable
fn unwrap_or_env(variable: Option<String>, env: &'static str, error_message: &'static str) -> Result<String, Error> {
    variable.map(Ok)
        .unwrap_or_else(||
            env::var(env)
                .with_context(|| error_message.to_string() + ": " + env)
        )
}

/// Use the environment variable value if present or use the given default value
fn env_or_default(env: &'static str, value: &'static str) -> Result<String, Error> {
    match env::var(env) {
        Ok(v) => Ok(v),
        Err(VarError::NotPresent) => Ok(value.to_string()),
        Err(e) => {
            Err(e).with_context(|| format!("Failed to read value from env '{}'", env))
        }
    }
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

    debug!("No vault token found, returning default token...");

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

    debug!("No vault address found, returning default address...");

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


#[cfg(test)]
mod tests {
    use crate::init_logger;

    use super::*;

    #[tokio::test]
    async fn test_build_client() {
        init_logger();

        let ctx = NovopsContext::default();
        let client = build_client(&ctx).await.unwrap();

        assert_eq!(client.settings.timeout, Some(Duration::new(60, 0)));
    }

}
