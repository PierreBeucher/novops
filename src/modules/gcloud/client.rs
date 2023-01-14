
use google_secretmanager1::{SecretManager, oauth2, hyper, hyper_rustls, api::SecretPayload};
use log::debug;
use async_trait::async_trait;

use crate::core::NovopsContext;


#[async_trait]
pub trait GCloudClient {
   async fn get_secret_version(&self, name: &str) -> Result<SecretPayload, anyhow::Error>;
}

pub struct DefaultGCloudClient {}
pub struct DryRunGCloudClient {}

#[async_trait]
impl GCloudClient for DefaultGCloudClient{

    async fn get_secret_version(&self, name: &str) -> Result<SecretPayload, anyhow::Error> {
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
}


#[async_trait]
impl GCloudClient for DryRunGCloudClient{

    async fn get_secret_version(&self, name: &str) -> Result<SecretPayload, anyhow::Error> {
        let mut result = "RESULT:".to_string();
        result.push_str(name);
        
        return Ok(SecretPayload{
            data: Some(base64::encode(result.to_string())),
            data_crc32c: Some(crc32c::crc32c(result.as_bytes()).to_string())
        })
        
    }
}

pub async fn get_client(ctx: &NovopsContext) -> Box<dyn GCloudClient + Send + Sync> {
    if ctx.dry_run {
        return Box::new(DryRunGCloudClient{})
    } else {
        return Box::new(DefaultGCloudClient{})
    }
}