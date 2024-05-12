mod test_lib;

use anyhow::Context;
use std::{ fs, path::PathBuf };
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::{
    self, 
    kv2, 
    kv1, 
    aws, 
    api::aws::requests::{ 
        SetConfigurationRequest, CreateUpdateRoleRequest 
    }
};
use log::info;
use std::{ collections::HashMap, thread, time };
use test_lib::{load_env_for, test_setup, aws_ensure_role_exists, create_dummy_context};
use novops::modules::hashivault::{
    client::{load_vault_token, load_vault_address},
    config::HashivaultConfig
};
use novops::core::NovopsContext;


#[tokio::test]
async fn test_hashivault_kv2() -> Result<(), anyhow::Error> {
    test_setup().await?;
    let client = hashivault_test_client();

    // enable kv2 engine
    let opts = HashMap::from([("version".to_string(), "2".to_string())]);
    enable_engine(&client, "kv2", "kv", Some(opts)).await?;

    // sleep a few seconds as creating kv2 secret engine may take a few seconds
    // vault may return a 400 in the meantime
    thread::sleep(time::Duration::from_secs(3));

    kv2::set(
        &client,
        "kv2",
        "test_hashivault_kv2",
        &HashMap::from([("novops_secret", "s3cret_kv2")])
    ).await.with_context(|| "Error when setting test secret for kv2")?;

    let outputs = load_env_for("hvault_kv2", "dev").await?;

    assert_eq!(outputs.variables.get("HASHIVAULT_KV_V2_TEST").unwrap().value, "s3cret_kv2");

    Ok(())
}

#[tokio::test]
async fn test_hashivault_kv1() -> Result<(), anyhow::Error> {
    test_setup().await?;
    
    let client = hashivault_test_client();
    enable_engine(&client, "kv1", "generic", None).await?;

    kv1::set(
        &client,
        "kv1",
        "test_hashivault_kv1",
        &HashMap::from([("novops_secret", "s3cret_kv1")])
    ).await.with_context(|| "Error when setting test secret for kv1")?;

    let outputs = load_env_for("hvault_kv1", "dev").await?;

    assert_eq!(outputs.variables.get("HASHIVAULT_KV_V1_TEST").unwrap().value, "s3cret_kv1");

    Ok(())
}

#[tokio::test]
async fn test_hashivault_aws() -> Result<(), anyhow::Error> {
    test_setup().await?;
    
    // Setup Vault AWS SE for Localstack and create Hashivault role
    let client = hashivault_test_client();
    enable_engine(&client, "test_aws", "aws", None).await?;
    
    aws::config::set(&client, "test_aws", "test_key", "test_secret", Some(SetConfigurationRequest::builder()
        .sts_endpoint("http://localstack:4566/") // Localstack URL reachable from Vault container in Docker Compose stack
        .iam_endpoint("http://localstack:4566/")
    )).await?;

    aws::roles::create_update(&client, "test_aws", "test_role", "assumed_role", Some(CreateUpdateRoleRequest::builder()
        .role_arns(vec!["arn:aws:iam::111122223333:role/test_role".to_string()])
    )).await?;

    // Make sure IAM Role exists on AWS side 
    aws_ensure_role_exists("test_role").await?;

    // Generate credentials
    let outputs = load_env_for("hvault_aws", "dev").await?;

    info!("Hashivault AWS credentials: {:?}", outputs);

    assert!(!outputs.variables.get("AWS_ACCESS_KEY_ID").unwrap().value.is_empty());
    assert!(!outputs.variables.get("AWS_SECRET_ACCESS_KEY").unwrap().value.is_empty());
    assert!(!outputs.variables.get("AWS_SESSION_TOKEN").unwrap().value.is_empty());

    Ok(())
}

/**
 * Check vault token is loaded in various situations in the proper order
 */
#[tokio::test]
async fn test_hashivault_client_token_load() -> Result<(), anyhow::Error> {
    test_setup().await?;

    // Empty config should yield empty token
    let ctx_empty = create_dummy_context_with_hvault(None, None, None);
    
    let result_empty = load_vault_token(&ctx_empty, None, None)?;
    assert!(result_empty.is_empty());

    // Token in home should be used
    // Create dummy token for testing
    // use linefeed and empty space to also test trimming
    let home_token = "hometoken\n   \n";
    let expected_home_token = "hometoken";

    let dummy_home_path = PathBuf::from("/tmp");
    let home_token_path = dummy_home_path.join(".vault-token");
    let home_var = Some(dummy_home_path);
    fs::write(home_token_path, home_token).with_context(|| "Couldn't write test token in /tmp")?;
    
    let ctx_empty = create_dummy_context_with_hvault(None, None, None);
    let result_home_token = load_vault_token(&ctx_empty, home_var.clone(), None)?;

    assert_eq!(result_home_token, expected_home_token);

    // Providing plain token should use it
    let token_plain = "token_plain";
    let ctx_token_path = create_dummy_context_with_hvault(
        None, Some(String::from(token_plain)), None);
    let result_token_plain = load_vault_token(&ctx_token_path, home_var.clone(), None)?;
    assert_eq!(result_token_plain, token_plain);

    // Providing token path should read token path before plain token
    let tmp_token_path = "/tmp/token";
    let token_file_content = "token_in_file";
    fs::write(tmp_token_path, token_file_content)
        .with_context(|| format!("Couldn't write test token to {tmp_token_path}"))?;

    let ctx_token_path = create_dummy_context_with_hvault(
        None, Some(String::from(token_plain)), Some(PathBuf::from(tmp_token_path)));
    let result_token_path = load_vault_token(&ctx_token_path, home_var.clone(), None)?;
    assert_eq!(result_token_path, token_file_content);

    // Providing token en var should use it before anything else
    let env_var_token = String::from("envvartoken");
    let ctx_token_path = create_dummy_context_with_hvault(
        None, Some(String::from(token_plain)), Some(PathBuf::from(tmp_token_path)));
    let result_token_env_var = load_vault_token(&ctx_token_path, home_var.clone(), Some(env_var_token.clone()))?;
    assert_eq!(result_token_env_var, env_var_token);

    Ok(())
}

/**
 * Check vault address is loaded in various situations in the proper order
 */
#[tokio::test]
async fn test_hashivault_client_address_load() -> Result<(), anyhow::Error> {
    test_setup().await?;

    // Empty config should yield empty address
    let ctx_empty = create_dummy_context_with_hvault(None, None, None);
    let result_empty = load_vault_address(&ctx_empty, None)?;
    assert_eq!(result_empty, url::Url::parse("http://127.0.0.1:8200")?);

    // Vault address config should yield configured address
    let addr_config = "https://dummy-vault-config";
    let ctx_addr = create_dummy_context_with_hvault(
        Some(String::from(addr_config)), None, None);

    let result_config = load_vault_address(&ctx_addr, None)?;
    assert_eq!(result_config, url::Url::parse(addr_config)?);

    // Vault address env var should be used first
    let addr_var = String::from("https://env-var-address");
    let ctx_addr = create_dummy_context_with_hvault(
        Some(String::from(addr_config)), None, None);

    let result_env_var = load_vault_address(&ctx_addr, Some(addr_var.clone()))?;
    assert_eq!(result_env_var, url::Url::parse(&addr_var)?);
    
    Ok(())
}

/**
 * Test client used to prepare Hashivault with a few secrets
 * Voluntarily separated from implemented client to make tests independent
 */
fn hashivault_test_client() -> VaultClient {
    return VaultClient::new(
        VaultClientSettingsBuilder::default()
            .token("novops")
            .build()
            .unwrap()
    ).unwrap();
}

async fn enable_engine(client: &VaultClient, path: &str, engine_type: &str, opts: Option<HashMap<String, String>>) -> Result<(), anyhow::Error> {
    let mounts = vaultrs::sys::mount::list(client).await
        .with_context(|| "Couldn't list secret engines")?;
    
    if ! mounts.contains_key(format!("{:}/", path).as_str()) {

        let mut options = vaultrs::api::sys::requests::EnableEngineRequest::builder();
        if let Some(opts) = opts {
            options.options(opts);
        }

        vaultrs::sys::mount::enable(client, path, engine_type, Some(&mut options)).await
            .with_context(|| format!("Couldn!'t enable engine {:} at path {:}", engine_type, path))?;    
    } else {
        info!("Secret engine {:} already enabled at {:}", engine_type, path)
    }
    
    Ok(())
}

fn create_dummy_context_with_hvault(addr: Option<String>, token: Option<String>, token_path: Option<PathBuf>) -> NovopsContext {
    let mut ctx = create_dummy_context();

    let novops_config = novops::core::NovopsConfig { hashivault: Some(HashivaultConfig {
        address: addr,
        token,
        token_path,
        verify: Some(false),
        timeout: None
    }), ..Default::default() };

    ctx.config_file_data.config = Some(novops_config);

    ctx
}