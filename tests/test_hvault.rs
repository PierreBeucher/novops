mod test_utils;

#[cfg(test)]
mod tests {
    use novops::{load_context_and_resolve, NovopsArgs};
    use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
    use vaultrs::kv2;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_hashivault() -> Result<(), anyhow::Error> {
        let client = hashivault_test_client();
        kv2::set(
            &client,
            "secret",
            "test_hashivault",
            &HashMap::from([("novops_secret", "s3cret")])
        ).await?;

        let args = NovopsArgs { 
            config: "tests/.novops.hashivault.yml".to_string(), 
            env: Some("dev".to_string()), 
            working_directory: None,
            symlink: None
        };

        let outputs = load_context_and_resolve(&args).await?;

        assert_eq!(outputs.variables[0].name, "HASHIVAULT_KV_V2_TEST");
        assert_eq!(outputs.variables[0].value, "s3cret");

        Ok(())
    }

    /**
     * Test client used to prepare Hashivault with a few secrets
     */
    fn hashivault_test_client() -> VaultClient {
        return VaultClient::new(
            VaultClientSettingsBuilder::default()
                .token("novops")
                .build()
                .unwrap()
        ).unwrap();
    }

}