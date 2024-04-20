
use anyhow::Context;
use google_secretmanager1::{
        SecretManager, 
        oauth2::{
            self, 
            hyper_rustls::HttpsConnector,
            authenticator::{Authenticator, ApplicationDefaultCredentialsTypes}, 
            ApplicationDefaultCredentialsAuthenticator,
            ApplicationDefaultCredentialsFlowOpts
        },
        hyper::{self, client::HttpConnector}, 
        hyper_rustls,
        api::SecretPayload
    };
use log::debug;
use async_trait::async_trait;
use home;

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

        let authenticator = get_authenticator()
            .await.with_context(|| "Couldn't get Google client authenticator")?;
        
        let hub = SecretManager::new(
            hyper::Client::builder().build(
                hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().build()
            ), 
            authenticator
        );

        let (_, secret) = hub.projects()
            .secrets_versions_access(name)
            .doit()
            .await.with_context(|| format!("Couldn't get secret {:?}. Did you setup credentials compatible with Application Default Credentials?", name))?;
        
        let result = secret.payload
            .ok_or(anyhow::anyhow!("No secret value found for '{}'", name))?;
            
        Ok(result)
            
    }
}

/// google_secretmanager1 uses yup_oauth2 which provides a bogus Application Default Credentials workflow
///
/// Google Application Default Credentials is documented as:
/// 1. GOOGLE_APPLICATION_CREDENTIALS environment variable
/// 2. User credentials set up by using the Google Cloud CLI
/// 3. The attached service account, returned by the metadata server
///
/// But google_secretmanager1 only performs 1 and 3
/// Workaround by implementing our own way for 2.
async fn get_authenticator() -> Result<Authenticator<HttpsConnector<HttpConnector>>, anyhow::Error> {

    // Try 1. using google_secretmanager1, expected a ServiceAccountAuthenticator
    let sa_authenticator = try_service_account_authenticator().await;

    debug!("Trying to get Service Account authenticator");
    match sa_authenticator {
        Ok(auth) => Ok(auth),
        Err(e) => {
            debug!("Couldn't generate Service Account authenticator: ${:?}", e);

            // 2. Look for credentials in home directory
            debug!("Trying to get Authorized User authenticator");
            let user_authenticator = try_user_authenticator().await;

            match user_authenticator {
                Ok(auth) => Ok(auth),
                Err(e) => {
                    debug!("Couldn't generate Authorized User Credentials authenticator: ${:?}", e);
                    
                    // 3. Use google_secretmanager1 again to retrieve Metadata instance
                    debug!("Trying to get Instance Metadata authenticator");
                    let metadata_authentiactor = try_metadata_authenticator().await;

                    match metadata_authentiactor {
                        Ok(auth) => Ok(auth),
                        Err(e) => {
                            debug!("Couldn't generate Instance Metadata authenticator: ${:?}", e);

                            Err(anyhow::anyhow!("Couldn't generate Authenticator for Google client. Did you setup credentials compatible with Application Default Credentials? "))
                        },
                    }
                },
            }
        },
    }

    
        
}

async fn try_metadata_authenticator() -> Result<Authenticator<HttpsConnector<HttpConnector>>, anyhow::Error>{
    let opts = ApplicationDefaultCredentialsFlowOpts::default();
    let authenticator = ApplicationDefaultCredentialsAuthenticator::builder(opts).await;

    // authenticator is either ServiceAccount or InstanceMetadata
    // We want ServiceAccount
    let result = match authenticator {
        ApplicationDefaultCredentialsTypes::InstanceMetadata(auth) => auth
            .build()
            .await
            .with_context(|| "Unable to create Instance Metadata authenticator")?,
        ApplicationDefaultCredentialsTypes::ServiceAccount(_) => 
            return Err(anyhow::anyhow!("Expected ServiceAccount authenticator."))
    };
    Ok(result)
}

// Try to generate a Service Account Authenticator using GOOGLE_APPLICATION_CREDENTIALS env var 
async fn try_service_account_authenticator() -> Result<Authenticator<HttpsConnector<HttpConnector>>, anyhow::Error>  {

    let opts = ApplicationDefaultCredentialsFlowOpts::default();
    let authenticator = ApplicationDefaultCredentialsAuthenticator::builder(opts).await;

    // authenticator is either ServiceAccount or InstanceMetadata
    // We want ServiceAccount
    let result = match authenticator {
        ApplicationDefaultCredentialsTypes::ServiceAccount(auth) => auth
            .build()
            .await
            .with_context(|| "Unable to create service account authenticator")?,
        ApplicationDefaultCredentialsTypes::InstanceMetadata(_) => 
            return Err(anyhow::anyhow!("Expected ServiceAccount authenticator."))
    };
    Ok(result)

}

// Try to generate a user authenticator using Authorized User workflow
async fn try_user_authenticator() -> Result<Authenticator<HttpsConnector<HttpConnector>>, anyhow::Error> {
    let home_dir_opt = home::home_dir();
        let home_dir = match home_dir_opt {
            Some(h) => h,
            None => return Err(anyhow::anyhow!("Couldn't find HOME directory.")),
        };

        let user_secret = oauth2::read_authorized_user_secret(home_dir.join(".config/gcloud/application_default_credentials.json"))
            .await.with_context(|| "Couldn't read Google client user secret".to_string())?;

        let user_authenticator = oauth2::AuthorizedUserAuthenticator::builder(user_secret)
            .build()
            .await.with_context(|| "Couldn't build AuthorizedUserAuthenticator for Google client".to_string())?;

        Ok(user_authenticator)
}

#[async_trait]
impl GCloudClient for DryRunGCloudClient{

    async fn get_secret_version(&self, name: &str) -> Result<SecretPayload, anyhow::Error> {
        let mut result = "RESULT:".to_string();
        result.push_str(name);

        return Ok(SecretPayload{
            data: Some(Vec::from(result.as_bytes())),
            data_crc32c: Some(i64::from(crc32c::crc32c(result.as_bytes())))
        })

        
    }
}

pub async fn get_client(ctx: &NovopsContext) -> Box<dyn GCloudClient + Send + Sync> {
    if ctx.dry_run {
        Box::new(DryRunGCloudClient{})
    } else {
        Box::new(DefaultGCloudClient{})
    }
}