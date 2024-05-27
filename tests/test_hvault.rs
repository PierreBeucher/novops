mod test_lib;

use anyhow::{Context, Error};
use std::{env, fs, path::PathBuf};
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
use std::env::temp_dir;
use std::time::{Duration};

use test_lib::{load_env_for, test_setup, aws_ensure_role_exists, create_dummy_context};
use novops::modules::hashivault::{
    client::{load_vault_token, load_vault_address, build_client},
    config::{HashivaultConfig, HashiVaultAuth}
};
use jwt_simple::prelude::*;
use k8s_openapi::api::core::v1::{Secret, ServiceAccount};
use k8s_openapi::api::authentication::v1::TokenRequest;
use k8s_openapi::api::rbac::v1::ClusterRoleBinding;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::Config;
use serde_json::json;
use vaultrs::api::auth::kubernetes::requests::CreateKubernetesRoleRequest;
use novops::core::NovopsContext;


// Kind Control Plane API URL to access inside docker compose network
const KUBERNETES_KIND_API_URL: &str = "https://novops-auth-test-control-plane:6443";


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
async fn test_hashivault_kv1() -> Result<(), Error> {
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
async fn test_hashivault_aws() -> Result<(), Error> {
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


#[tokio::test]
async fn test_hashivault_auth_approle() -> Result<(), Error> {
    test_setup().await?;

    let approle_mount = "approle";
    let role_without_secret = "without-secret";

    let client = hashivault_test_client();
    enable_auth(&client, approle_mount, "approle").await?;

    vault_set_app_role(&client, approle_mount, role_without_secret, false).await?;
    let role_id = vaultrs::auth::approle::role::read_id(&client, approle_mount, role_without_secret).await?;

    let auth = HashiVaultAuth::AppRole {
        mount_path: approle_mount.to_string().into(),
        role_id: role_id.role_id.to_string().into(),
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

    let approle_mount = "approle-secret-path";
    let role_with_secret = "with-secret";

    let client = hashivault_test_client();
    enable_auth(&client, approle_mount, "approle").await?;

    vault_set_app_role(&client, approle_mount, role_with_secret, true).await?;
    let role_id = vaultrs::auth::approle::role::read_id(&client, approle_mount, role_with_secret).await?;
    let secret = vaultrs::auth::approle::role::secret::generate(&client, approle_mount, role_with_secret, None).await?;

    env::set_var("VAULT_AUTH_SECRET_ID", secret.secret_id);

    let auth = HashiVaultAuth::AppRole {
        mount_path: approle_mount.to_string().into(),
        role_id: role_id.role_id.to_string().into(),
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

    let approle_mount = "approle-secret";
    let role_with_secret = "with-secret";

    let client = hashivault_test_client();
    enable_auth(&client, approle_mount, "approle").await?;

    vault_set_app_role(&client, approle_mount, role_with_secret, true).await?;
    let role_id = vaultrs::auth::approle::role::read_id(&client, approle_mount, role_with_secret).await?;
    let secret = vaultrs::auth::approle::role::secret::generate(&client, approle_mount, role_with_secret, None).await?;

    let secret_id_path = env::temp_dir().join("secret-id");
    fs::write(secret_id_path.clone(), secret.secret_id.to_string())?;

    let auth = HashiVaultAuth::AppRole {
        mount_path: approle_mount.to_string().into(),
        role_id: role_id.role_id.to_string().into(),
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

    let jwt_mount = "jwt-env";
    let role = "test-env";

    let client = hashivault_test_client();
    let key_pair = vault_configure_jwt(&client, jwt_mount).await?;
    let token_claims = create_jwt_claims();

    vault_configure_jwt_role(&client, jwt_mount, role, &token_claims).await?;
    
    let jwt_token = key_pair.sign(token_claims)?;

    env::set_var("VAULT_AUTH_JWT_TOKEN", jwt_token);

    let auth = HashiVaultAuth::JWT {
        mount_path: jwt_mount.to_string().into(),
        token_path: None,
        role: role.to_string().into()
    };

    let vault_context = create_dummy_auth_context(None, auth, None, None);
    let actual_client = build_client(&vault_context).await.expect("Should login with no errors");

    assert!(!actual_client.settings.token.is_empty());

    Ok(())
}


#[tokio::test]
async fn test_hashivault_auth_jwt_file() -> Result<(), Error> {
    test_setup().await?;

    let jwt_mount = "jwt-file";
    let role = "test-file";

    let client = hashivault_test_client();
    let key_pair = vault_configure_jwt(&client, jwt_mount).await?;
    let token_claims = create_jwt_claims();

    vault_configure_jwt_role(&client, jwt_mount, role, &token_claims).await?;

    let jwt_token = key_pair.sign(token_claims)?;
    let jwt_token_file = env::temp_dir().join("novops-jwt-token");

    fs::write(jwt_token_file.clone(), jwt_token)?;

    let auth = HashiVaultAuth::JWT {
        mount_path: jwt_mount.to_string().into(),
        token_path: Some(jwt_token_file.to_string_lossy().to_string()),
        role: role.to_string().into()
    };

    let vault_context = create_dummy_auth_context(None, auth, None, None);
    let actual_client = build_client(&vault_context).await.expect("Should login with no errors");

    assert!(!actual_client.settings.token.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_hashivault_auth_kubernetes() -> Result<(), Error> {
    test_setup().await?;

    let kubernetes_mount = "kubernetes";
    let role = "test";
    let sa_name = "novops-test-auth";

    let client = hashivault_test_client();
    let kube_config = Config::infer().await?;

    let kube_ca_cert = pem::encode_many(
        &kube_config.root_cert.clone().unwrap()
        .into_iter()
        .map(|v| pem::Pem::new("CERTIFICATE", v))
        .collect::<Vec<_>>()
    );

    let kube_client = kube::Client::try_from(kube_config.clone())?;

    let sa_token = kube_create_sa_token(kube_client, sa_name).await?;

    vault_configure_kubernetes(&client, kubernetes_mount, Some(kube_ca_cert), KUBERNETES_KIND_API_URL).await?;
    vault_configure_kube_role(&client, kubernetes_mount, role).await?;

    let sa_token_file = temp_dir().join("novops-test-auth-sa-token");
    fs::write(sa_token_file.clone(), sa_token)?;

    env::set_var("KUBERNETES_SA_JWT_PATH", sa_token_file.to_string_lossy().to_string());

    let auth = HashiVaultAuth::Kubernetes {
        mount_path: kubernetes_mount.to_string().into(),
        role: role.to_string().into()
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
        subject: "user1".to_string().into(),
        issuer: "test.local".to_string().into(),
        ..Claims::create(Duration::from_secs(30).into())
    }
}

async fn vault_configure_jwt_role(client: &VaultClient, jwt_mount: &str, role: &str, token_claims: &JWTClaims<NoCustomClaims>) -> Result<(), Error> {
    let user_claim = "sub";
    
    let mut set_role = vaultrs::api::auth::oidc::requests::SetRoleRequest::builder();
    set_role.role_type("jwt".to_string())
        .bound_subject(token_claims.subject.clone().unwrap())
        .user_claim(user_claim);

    vaultrs::auth::oidc::role::set(client, jwt_mount, role, user_claim, vec![], Some(&mut set_role)).await?;

    Ok(())
}

async fn vault_set_app_role(client: &VaultClient, mount_path: &str, role_name: &str, bind_secret_id: bool) -> Result<(), Error> {
    let mut set_approle = vaultrs::api::auth::approle::requests::SetAppRoleRequest::builder();
    set_approle
        .bind_secret_id(bind_secret_id)
        .role_name(role_name)
        .token_bound_cidrs(vec!["0.0.0.0/0".to_string()]);

    vaultrs::auth::approle::role::set(client, mount_path, role_name, Some(&mut set_approle)).await?;
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

async fn enable_engine(client: &VaultClient, path: &str, engine_type: &str, opts: Option<HashMap<String, String>>) -> Result<(), Error> {
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

async fn enable_auth(client: &VaultClient, path: &str, engine_type: &str) -> Result<(), Error> {
    let mounts = vaultrs::sys::auth::list(client).await.with_context(|| "Couldn't list authentication backend")?;

    if ! mounts.contains_key(format!("{:}/", path).as_str()) {
        let mut options = vaultrs::api::sys::requests::EnableAuthRequest::builder();

        vaultrs::sys::auth::enable(client, path, engine_type, Some(&mut options)).await
            .with_context(|| format!("Couldn't enable auth backend {:} at path {:}", engine_type, path))?;
    } else {
        info!("Auth backend {:} already enabled at {:}", engine_type, path)
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

async fn vault_configure_jwt(client: &VaultClient, jwt_mount: &str) -> Result<ES256KeyPair, Error> {
    let key_pair = ES256KeyPair::generate();
    let public_key = key_pair.public_key();

    enable_auth(client, jwt_mount, "jwt").await?;

    let mut jwt_auth_config = vaultrs::api::auth::oidc::requests::SetConfigurationRequest::builder();
    jwt_auth_config.jwt_validation_pubkeys(vec![public_key.to_pem()?]);

    vaultrs::auth::oidc::config::set(client, jwt_mount, Some(&mut jwt_auth_config)).await?;
    
    Ok(key_pair)
}

async fn vault_configure_kubernetes(client: &VaultClient, mount_path: &str, kubernetes_ca: Option<String>, kubernetes_host: &str) -> Result<(), Error> {
    enable_auth(client, mount_path, "kubernetes").await?;

    let mut kube_auth_config = vaultrs::api::auth::kubernetes::requests::ConfigureKubernetesAuthRequest::builder();
    kube_auth_config
        .disable_iss_validation(true)
        .kubernetes_host(kubernetes_host);

    if let Some(ca) = kubernetes_ca {
        kube_auth_config.kubernetes_ca_cert(ca.as_str());
    }

    vaultrs::auth::kubernetes::configure(client, mount_path, kubernetes_host, Some(&mut kube_auth_config)).await?;

    Ok(())
}

/// Create the service account on kubernetes with permission auth delegator to allow Vault to validate
/// the service account token on Kubernetes.
/// Returns the Service Account generated token.
async fn kube_create_sa_token(kube_client: kube::Client, sa_name: &str) -> Result<String, Error> {
    let sas: kube::Api<ServiceAccount> = kube::Api::default_namespaced(kube_client.clone());
    let secrets: kube::Api<Secret> = kube::Api::default_namespaced(kube_client.clone());
    let cluster_role_bindings: kube::Api<ClusterRoleBinding> = kube::Api::all(kube_client);

    let sa_metadata = ObjectMeta {
        name: Some(sa_name.to_string()),
        ..ObjectMeta::default()
    };

    let post_params = kube::api::PostParams::default();
    let sa = ServiceAccount {
        automount_service_account_token: None,
        image_pull_secrets: None,
        metadata: sa_metadata,
        secrets: None,
    };

    ignore_already_exists(sas.create(&post_params, &sa).await)?;

    let secret_name = format!("{}-token", sa_name);

    let secret: Secret = serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
           "name": secret_name,
           "annotations": {
              "kubernetes.io/service-account.name": sa_name,
              "type": "kubernetes.io/service-account-token"
           }
        }
    }))?;

    ignore_already_exists(secrets.create(&post_params, &secret).await)?;

    let token = sas.create_token_request(sa_name, &post_params, &TokenRequest::default()).await?;
    let token = token.status.expect("kubernetes token request status is empty").token;

    // Create a role binding to allow the service account token to be used by vault during login
    let role_binding = serde_json::from_value::<ClusterRoleBinding>(json!({
      "apiVersion": "rbac.authorization.k8s.io/v1",
      "kind": "ClusterRoleBinding",
      "metadata": {
        "name": format!("{}-tokenreview-binding", sa_name),
        "namespace": "default"
      },
      "roleRef": {
        "apiGroup": "rbac.authorization.k8s.io",
        "kind": "ClusterRole",
        "name": "system:auth-delegator"
      },
      "subjects": [
        {
          "kind": "ServiceAccount",
          "name": sa_name,
          "namespace": "default"
        }
      ]
    }))?;

    ignore_already_exists(cluster_role_bindings.create(&post_params, &role_binding).await)?;

    Ok(token)
}

/// Configure the kubernetes role on Vault allowing it to login in any namespace and service account
async fn vault_configure_kube_role(client: &VaultClient, mount_path: &str, role: &str) -> Result<(), Error> {
    let mut role_request = CreateKubernetesRoleRequest::builder();
    role_request
        .bound_service_account_names(vec!["*".to_string()])
        .bound_service_account_namespaces(vec!["*".to_string()]);

    vaultrs::auth::kubernetes::role::create(client, mount_path, role, Some(&mut role_request)).await?;

    Ok(())
}

/// Ignore kubernetes errors related to 429 when the resource already exists
fn ignore_already_exists<T>(result: Result<T, kube::error::Error>) -> Result<(), Error> {
    match result {
        Err(kube::error::Error::Api(e)) => {
            if e.code != 409 {
                return Err(e)?
            }
            Ok(())
        },
        Err(e) => Err(e)?,
        Ok(_) => Ok(())
    }
}
