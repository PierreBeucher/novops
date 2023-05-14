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

  /// Token used to connect
  pub token: Option<String>,

  /// TLS verify
  pub verify: Option<bool>
}

impl Default for HashivaultConfig {
  fn default() -> HashivaultConfig {
    HashivaultConfig{
      address: None,
      token: None,
      verify: None
    }
  }
}
