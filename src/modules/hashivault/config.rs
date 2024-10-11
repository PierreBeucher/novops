use std::path::PathBuf;

use serde::Deserialize;
use schemars::JsonSchema;

use super::aws::HashiVaultAWSInput;

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultInput {

    /// Use Vault AWS Secret Engine to generate temporary AWS credentials.
    pub aws: HashiVaultAWSInput
}


#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(tag = "type")]
pub enum HashiVaultAuth {
    Kubernetes {
        mount_path: Option<String>,
        role: Option<String>
    },
    AppRole {
        mount_path: Option<String>,
        role_id: Option<String>,
        secret_id_path: Option<String>
    },
    JWT {
        token_path: Option<String>,
        role: Option<String>,
        mount_path: Option<String>
    }
}


#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct HashivaultConfig {
  /// Address in form http(s)://HOST:PORT
  /// 
  /// Example: https://vault.mycompany.org:8200
  pub address: Option<String>,

  /// Vault token as plain string
  /// 
  /// Use for testing only. DO NOT COMMIT NOVOPS CONFIG WITH THIS SET.
  /// 
  pub token: Option<String>,

  /// Vault token path.
  /// 
  /// Example: /var/secrets/vault-token
  pub token_path: Option<PathBuf>,

  /// Whether to enable TLS verify (true by default)
  pub verify: Option<bool>,

  /// Vault client timeout in seconds. Default to 60s.
  pub timeout: Option<u64>,

  /// Vault authentication to use when a token is not provided
  pub auth: Option<HashiVaultAuth>,

  /// Vault namespace to use
  pub namespace: Option<String>
}


