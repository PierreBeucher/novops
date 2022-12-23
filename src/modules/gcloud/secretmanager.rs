use anyhow::Context;
use serde::Deserialize;
use async_trait::async_trait;
use schemars::JsonSchema;
use std::default::Default;

use crate::core::{ResolveTo, NovopsContext};

use base64;
use crc32c;
use log::debug;

use google_secretmanager1::{SecretManager, oauth2, hyper, hyper_rustls, api::SecretPayload};

#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct GCloudSecretManagerSecretInput {
    
    pub gcloud_secret: GCloudSecretManagerSecret
}

/**
 * Structure to request a GCloud Secret Manager secret
 * 
 * See https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_GetSecretValue.html
 */
#[derive(Debug, Deserialize, Clone, PartialEq, JsonSchema, Default)]
pub struct GCloudSecretManagerSecret {
    
    /**
     * Name of the secret in the format projects/\*\/secrets/\*\/versions/\*
     * Such as `projects/my-org-project/secrets/my-secret/latest`
     * Or `projects/my-org-project/secrets/my-secret/42` for a specific version
     */
    pub name: String,

    /**
     * Whether to validate crc32c checksum provided with secret (default: true)
     */
    pub validate_crc32c: Option<bool>
}

#[async_trait]
impl ResolveTo<String> for GCloudSecretManagerSecretInput {
    async fn resolve(&self, _: &NovopsContext) -> Result<String, anyhow::Error> {

        let value = retrieve_secret_bytes_for(&self.gcloud_secret).await?;

        let result = String::from_utf8(value)
            .with_context(|| format!("Couldn't convert secret {:} bytes into String", &self.gcloud_secret.name))
            .unwrap();

        return Ok(result);

    }
}

#[async_trait]
impl ResolveTo<Vec<u8>> for GCloudSecretManagerSecretInput {
    async fn resolve(&self, _: &NovopsContext) -> Result<Vec<u8>, anyhow::Error> {
        return retrieve_secret_bytes_for(&self.gcloud_secret).await;
    }
}

/**
 * Return bytes value or a secret after validating CRC32
 */
async fn retrieve_secret_bytes_for(secret: &GCloudSecretManagerSecret) -> Result<Vec<u8>, anyhow::Error> {
    let payload = retrieve_secret_payload(&secret.name).await?;

    // decode b64
    let bytes_val = base64::decode(payload.data.unwrap()).unwrap();

    if secret.validate_crc32c.unwrap_or(true) {
        let expected_checksum = payload.data_crc32c.unwrap();
        let calculated_checksum = crc32c::crc32c(&bytes_val).to_string();

        debug!("Secret {:} - expected checksum: {:} - calculated checksum: {:}", 
            &secret.name, &expected_checksum, &calculated_checksum);

        anyhow::ensure!(expected_checksum == calculated_checksum, 
            format!("Couldn't validate checksum for {:}: expected '{:}' got '{:}'", 
            &secret.name, expected_checksum, calculated_checksum));
    }
    
    return Ok(bytes_val);
}

async fn retrieve_secret_payload(name: &str) -> Result<SecretPayload, anyhow::Error> {

    debug!("Retrieving secret: {:}", &name);

    let opts = oauth2::ApplicationDefaultCredentialsFlowOpts::default();
        let authenticator = match oauth2::ApplicationDefaultCredentialsAuthenticator::builder(opts).await {
            oauth2::authenticator::ApplicationDefaultCredentialsTypes::InstanceMetadata(auth) => auth
                    .build()
                    .await
                    .expect("Unable to create instance metadata authenticator"),
                oauth2::authenticator::ApplicationDefaultCredentialsTypes::ServiceAccount(auth) => auth
                    .build()
                    .await
                    .expect("Unable to create service account authenticator"),
        };

        let hub = SecretManager::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()
            ), 
            authenticator
        );

        let (_, secret) = hub.projects()
            .secrets_versions_access(name)
            .doit().await?;

        return Ok(secret.payload.unwrap());

}



// #[async_trait]
// impl ResolveTo<String> for AwsSecretsManagerSecretInput {
//     async fn resolve(&self, ctx: &NovopsContext) -> Result<String, anyhow::Error> {

//         let output = retrieve_secret(ctx, self).await?;

//         if output.secret_string().is_some(){
//             return Ok(output.secret_string().unwrap().to_string());
//         }

//         if output.secret_binary().is_some(){
//             let binary = output.secret_binary().unwrap().clone().into_inner();
//             let result = String::from_utf8(binary)
//                 .with_context(|| format!("Couldn't convert bytes from Secrets Manager secret '{}' to UTF-8 String. \
//                 Non-UTF-8 binary data can't be used as Variable input yet. Either use File input for binary data or make sure it's a valid UTF-8 string.", self.aws_secret.id))?;
//             return Ok(result);
//         }

//         return Err(anyhow::format_err!("Secret value was neither string nor binary, got response: {:?}", output));        
        
//     }
// }

// async fn retrieve_secret(ctx: &NovopsContext, input: &AwsSecretsManagerSecretInput) -> Result<GetSecretValueOutput, anyhow::Error>{
//     let client_conf = build_mutable_client_config_from_context(ctx);
//     let ssm_client = get_secretsmanager_client(&client_conf).await?;

//     let output = ssm_client.get_secret_value()
//         .secret_id(input.aws_secret.id.clone())
//         .set_version_id(input.aws_secret.version_id.clone())
//         .set_version_stage(input.aws_secret.version_stage.clone())
//         .send().await?;

//     return Ok(output);
// }
