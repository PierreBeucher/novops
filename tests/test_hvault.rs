mod test_utils;

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
    use vaultrs::{kv2, kv1};
    use log::info;
    use std::collections::HashMap;
    use crate::test_utils::{load_env_for, test_setup};

    #[tokio::test]
    async fn test_hashivault_kv2() -> Result<(), anyhow::Error> {
        test_setup();
        let client = hashivault_test_client();

        // enable kv2 engine
        let opts = HashMap::from([("version".to_string(), "2".to_string())]);
        enable_engine(&client, "kv2", "kv", Some(opts)).await?;

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
    async fn test_hashivault_kv1() -> Result<(), anyhow::Error> {
        test_setup();
        
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

    async fn enable_engine(client: &VaultClient, path: &str, engine_type: &str, opts: Option<HashMap<String, String>>) -> Result<(), anyhow::Error> {
        let mounts = vaultrs::sys::mount::list(client).await
            .with_context(|| "Couldn't list secret engines")?;
        
        if ! mounts.contains_key(format!("{:}/", path).as_str()) {

            let mut options = vaultrs::api::sys::requests::EnableEngineRequest::builder();
            if opts.is_some(){
                options.options(opts.unwrap());
            };

            vaultrs::sys::mount::enable(client, path, engine_type, Some(&mut options)).await
                .with_context(|| format!("Couldn!'t enable engine {:} at path {:}", engine_type, path))?;    
        } else {
            info!("Secret engine {:} already enabled at {:}", engine_type, path)
        }
        
        Ok(())
    }

}