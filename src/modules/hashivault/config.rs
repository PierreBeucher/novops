use std::path::PathBuf;

use serde::Deserialize;
use schemars::JsonSchema;

use super::aws::HashiVaultAWSInput;

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct HashiVaultInput {
    pub aws: HashiVaultAWSInput
}


#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]  
pub struct HashivaultConfig {
  /// Address in form http(s)://HOST:PORT
  pub address: Option<String>,

  /// Hashivault token (plain string, for testing purpose only)
  pub token: Option<String>,

  /// Hashivault token path 
  pub token_path: Option<PathBuf>,

  /// TLS verify
  pub verify: Option<bool>
}

impl Default for HashivaultConfig {
  fn default() -> HashivaultConfig {
    HashivaultConfig{
      address: None,
      token: None,
      token_path: None,
      verify: None
    }
  }
}
