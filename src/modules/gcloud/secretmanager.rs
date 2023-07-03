use anyhow::Context;
use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;
use std::default::Default;
use crc32c;
use log::debug;


use crate::core::{ResolveTo, NovopsContext};
use super::client::get_client;

/// Reference Secret Manager secret
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GCloudSecretManagerSecretInput {
    
    pub gcloud_secret: GCloudSecretManagerSecret
}


/// Structure to request a GCloud Secret Manager secret
/// 
/// See https://cloud.google.com/secret-manager/docs/reference/rest/v1/projects.secrets/get
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct GCloudSecretManagerSecret {
    
    /// Name of the secret in the format projects/\*\/secrets/\*\/versions/\*
    /// Such as `projects/my-org-project/secrets/my-secret/latest`
    /// Or `projects/my-org-project/secrets/my-secret/42` for a specific version
    pub name: String,

    /// Whether to validate crc32c checksum provided with secret (default: true)
    pub validate_crc32c: Option<bool>
}

#[async_trait]
impl ResolveTo<String> for GCloudSecretManagerSecretInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

        let value = retrieve_secret_bytes_for(ctx, &self.gcloud_secret).await?;

        let result = String::from_utf8(value)
            .with_context(|| format!("Couldn't convert secret {:} bytes into String", &self.gcloud_secret.name))?;

        return Ok(result);

    }
}

#[async_trait]
impl ResolveTo<Vec<u8>> for GCloudSecretManagerSecretInput {
    async fn resolve(&self, ctx: &NovopsContext) -> Result<Vec<u8>, anyhow::Error> {
        return retrieve_secret_bytes_for(ctx, &self.gcloud_secret).await;
    }
}


/// Return bytes value or a secret after validating CRC32
async fn retrieve_secret_bytes_for(ctx: &NovopsContext, secret: &GCloudSecretManagerSecret) -> Result<Vec<u8>, anyhow::Error> {
    
    let client = get_client(ctx).await;

    let payload = client.get_secret_version(&secret.name).await?;

    let payload_data = payload.data
        .ok_or(anyhow::anyhow!("No payload data obtained for {}", &secret.name))?;

    if secret.validate_crc32c.unwrap_or(true) {
        let expected_checksum = payload.data_crc32c
            .ok_or(anyhow::anyhow!("No CRC32C found for {:}. If the secret has no CRC32C, set 'validate_crc32c: false' on input", &secret.name))?;

        let calculated_checksum = i64::from(crc32c::crc32c(&payload_data));

        debug!("Secret {:} - expected checksum: {:} - calculated checksum: {:}", 
            &secret.name, &expected_checksum, &calculated_checksum);

        anyhow::ensure!(expected_checksum == calculated_checksum, 
            format!("Couldn't validate checksum for {:}: expected '{:}' got '{:}'", 
            &secret.name, expected_checksum, calculated_checksum));
    }
    
    return Ok(payload_data);
}
