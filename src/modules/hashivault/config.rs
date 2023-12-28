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
#[derive(Default)]
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
  pub verify: Option<bool>
}


