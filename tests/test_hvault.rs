mod test_lib;

use anyhow::{Context, Error};
use kube::config::KubeConfigOptions;
use std::{env, fs, path::PathBuf};
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use log::info;
use std::env::temp_dir;
use std::time::Duration;

use test_lib::{load_env_for, test_setup, create_dummy_context};
use novops::modules::hashivault::{
    client::{load_vault_token, load_vault_address, build_client},
    config::{HashivaultConfig, HashiVaultAuth}
};
use jwt_simple::prelude::*;
use k8s_openapi::api::core::v1::Secret;
use kube;
use novops::core::NovopsContext;

#[tokio::test]
async fn test_hashivault_kv2() -> Result<(), anyhow::Error> {
    test_setup().await?;

    // let client = hashivault_test_client();
    // // enable kv2 engine
    // let opts = HashMap::from([("version".to_string(), "2".to_string())]);
    // enable_engine(&client, "kv2", "kv", Some(opts)).await?;

    // // sleep a few seconds as creating kv2 secret engine may take a few seconds
    // // vault may return a 400 in the meantime
    // thread::sleep(time::Duration::from_secs(3));

    // kv2::set(
    //     &client,
    //     "kv2",
    //     "test_hashivault_kv2",
    //     &HashMap::from([("novops_secret", "s3cret_kv2")])
    // ).await.with_context(|| "Error when setting test secret for kv2")?;

    let outputs = load_env_for("hvault_kv2", "dev").await?;

    assert_eq!(outputs.variables.get("HASHIVAULT_KV_V2_TEST").unwrap().value, "s3cret_kv2");

    Ok(())
}

#[tokio::test]
async fn test_hashivault_kv1() -> Result<(), Error> {
    test_setup().await?;

    // let client = hashivault_test_client();
    // enable_engine(&client, "kv1", "generic", None).await?;

    // kv1::set(
    //     &client,
    //     "kv1",
    //     "test_hashivault_kv1",
    //     &HashMap::from([("novops_secret", "s3cret_kv1")])
    // ).await.with_context(|| "Error when setting test secret for kv1")?;

    let outputs = load_env_for("hvault_kv1", "dev").await?;

    assert_eq!(outputs.variables.get("HASHIVAULT_KV_V1_TEST").unwrap().value, "s3cret_kv1");

    Ok(())
}

#[tokio::test]
async fn test_hashivault_aws() -> Result<(), Error> {
    test_setup().await?;

    // Setup Vault AWS SE for Localstack and create Hashivault role
    // let client = hashivault_test_client();
    // enable_engine(&client, "test_aws", "aws", None).await?;

    // aws::config::set(&client, "test_aws", "test_key", "test_secret", Some(SetConfigurationRequest::builder()
    //     .sts_endpoint("http://localstack:4566/") // Localstack URL reachable from Vault container in Docker Compose stack
    //     .iam_endpoint("http://localstack:4566/")
    // )).await?;

    // aws::roles::create_update(&client, "test_aws", "test_role", "assumed_role", Some(CreateUpdateRoleRequest::builder()
    //     .role_arns(vec!["arn:aws:iam::111122223333:role/test_role".to_string()])
    // )).await?;

    // Make sure IAM Role exists on AWS side
    // aws_ensure_role_exists("test_role").await?;

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


#[tokio::test]
async fn test_hashivault_auth_approle() -> Result<(), Error> {
    test_setup().await?;

    // let client = hashivault_test_client();
    // let role_id = vaultrs::auth::approle::role::read_id(&client, "approle", "without-secret").await?;

    let auth = HashiVaultAuth::AppRole {
        mount_path: Some(String::from("approle")),
        role_id: Some(String::from("role_id_without_secret")),
        secret_id_path: None,
    };

    let vault_context = create_dummy_auth_context(None, auth, None, None);
    let actual_client = build_client(&vault_context).await.expect("Should login with no errors");

    assert!(!actual_client.settings.token.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_hashivault_auth_approle_secret() -> Result<(), anyhow::Error> {
    test_setup().await?;

    let client = hashivault_test_client();
    let secret = vaultrs::auth::approle::role::secret::generate(&client, "approle", "with-secret", None).await?;

    env::set_var("VAULT_AUTH_SECRET_ID", secret.secret_id);

    let auth = HashiVaultAuth::AppRole {
        mount_path: Some(String::from("approle")),
        role_id: Some(String::from("role_id_with_secret")),
        secret_id_path: None,
    };

    let vault_context = create_dummy_auth_context(None, auth, None, None);
    let actual_client = build_client(&vault_context).await.expect("Should login with no errors");

    assert!(!actual_client.settings.token.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_hashivault_auth_approle_secret_path() -> Result<(), anyhow::Error> {
    test_setup().await?;

    let client = hashivault_test_client();
    let secret = vaultrs::auth::approle::role::secret::generate(&client, "approle", "with-secret", None).await?;

    let secret_id_path = env::temp_dir().join("secret-id");
    fs::write(secret_id_path.clone(), secret.secret_id.to_string())?;

    let auth = HashiVaultAuth::AppRole {
        mount_path: Some(String::from("approle")),
        role_id: Some(String::from("role_id_with_secret")),
        secret_id_path: secret_id_path.to_string_lossy().to_string().into(),
    };

    let vault_context = create_dummy_auth_context(None, auth, None, None);
    let actual_client = build_client(&vault_context).await.expect("Should login with no errors");

    assert!(!actual_client.settings.token.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_hashivault_auth_jwt() -> Result<(), Error> {
    test_setup().await?;

    // Create a test JWT token with known private key used by test Vault instance
    let token_claims = create_jwt_claims();
    
    let private_key = fs::read_to_string("tests/setup/pulumi/vault/jwt_private_key.pem").with_context(|| "Couldn't read private key")?;
    let keypair = RS384KeyPair::from_pem(&private_key).with_context(|| "Couldn't load keypair")?;
    
    let jwt_token = keypair.sign(token_claims).with_context(|| "Couldn't sign claims with keypair")?;

    env::set_var("VAULT_AUTH_JWT_TOKEN", jwt_token);

    let auth = HashiVaultAuth::JWT {
        mount_path: Some(String::from("jwt")),
        token_path: None,
        role: Some(String::from("test-role"))
    };

    let vault_context = create_dummy_auth_context(None, auth, None, None);
    let actual_client = build_client(&vault_context).await.expect("Should login with no errors");

    assert!(!actual_client.settings.token.is_empty());

    Ok(())
}


#[tokio::test]
async fn test_hashivault_auth_jwt_file() -> Result<(), Error> {
    test_setup().await?;

    // Create a test JWT token with known private key used by test Vault instance
    let token_claims = create_jwt_claims();
    
    let private_key = fs::read_to_string("tests/setup/pulumi/vault/jwt_private_key.pem").with_context(|| "Couldn't read private key")?;
    let keypair = RS384KeyPair::from_pem(&private_key).with_context(|| "Couldn't load keypair")?;
    
    let jwt_token = keypair.sign(token_claims).with_context(|| "Couldn't sign claims with keypair")?;
    let jwt_token_file = env::temp_dir().join("novops-jwt-token");
    fs::write(jwt_token_file.clone(), jwt_token)?;

    let auth = HashiVaultAuth::JWT {
        mount_path: Some(String::from("jwt")),
        token_path: Some(jwt_token_file.to_string_lossy().to_string()),
        role: Some(String::from("test-role"))
    };

    let vault_context = create_dummy_auth_context(None, auth, None, None);
    let actual_client = build_client(&vault_context).await.expect("Should login with no errors");

    assert!(!actual_client.settings.token.is_empty());

    Ok(())
}

fn create_jwt_claims() -> JWTClaims<NoCustomClaims> {
    JWTClaims {
        audiences: Some(Audiences::AsString("vault".to_string())),
        jwt_id: "c82eeb0c-5c6f-4a33-abf5-4c474b92b558".to_string().into(),
        subject: "novops_test_subject".to_string().into(),
        issuer: "novops.test".to_string().into(),
        ..Claims::create(Duration::from_secs(30).into())
    }
}

#[tokio::test]
async fn test_hashivault_auth_kubernetes() -> Result<(), Error> {
    test_setup().await?;

    let kubeconfig = kube::config::Kubeconfig::read_from("tests/setup/k8s/kubeconfig").with_context(|| "Couldn't read kubeconfig")?;
    let kube_client_config = kube::Config::from_custom_kubeconfig(kubeconfig, &KubeConfigOptions {
        cluster: None,
        context: Some(String::from("kind-novops-auth-test")),
        user: None
    }).await?;

    let kube_client = kube::Client::try_from(kube_client_config)?;

    let secrets: kube::Api<Secret> = kube::Api::default_namespaced(kube_client);
    let sa_secret = secrets.get("vault-jwt-test-sa-token").await?;
    let sa_token = String::from_utf8(sa_secret.data.unwrap().get("token").unwrap().0.clone())?;
    
    let sa_token_file = temp_dir().join("novops-test-auth-sa-token");
    fs::write(sa_token_file.clone(), sa_token)?;

    env::set_var("KUBERNETES_SA_JWT_PATH", sa_token_file.to_string_lossy().to_string());

    let auth = HashiVaultAuth::Kubernetes {
        mount_path: Some(String::from("k8s")),
        role: Some(String::from("test-role"))
    };

    let vault_context = create_dummy_auth_context(None, auth, None, None);
    let actual_client = build_client(&vault_context).await.expect("Should login with no errors");

    assert!(!actual_client.settings.token.is_empty());

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

fn create_dummy_context_with_hvault(addr: Option<String>, token: Option<String>, token_path: Option<PathBuf>) -> NovopsContext {
    let mut ctx = create_dummy_context();

    let novops_config = novops::core::NovopsConfig { hashivault: Some(HashivaultConfig {
        address: addr,
        token,
        token_path,
        verify: Some(false),
        timeout: None,
        auth: None
    }), ..Default::default() };

    ctx.config_file_data.config = Some(novops_config);

    ctx
}

fn create_dummy_auth_context(addr: Option<String>, auth: HashiVaultAuth, token: Option<String>, token_path: Option<PathBuf>) -> NovopsContext {
    let mut context = create_dummy_context_with_hvault(addr, token, token_path);

    let config = context.config_file_data.config.as_mut().unwrap().hashivault.as_mut().unwrap();
    config.auth = Some(auth);

    context
}
