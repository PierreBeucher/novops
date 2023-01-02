mod test_utils;

#[cfg(test)]
mod tests {
    use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
    use vaultrs::kv2;
    use std::collections::HashMap;
    use crate::test_utils::load_env_for;

    #[tokio::test]
    async fn test_hashivault() -> Result<(), anyhow::Error> {
        let client = hashivault_test_client();
        kv2::set(
            &client,
            "secret",
            "test_hashivault",
            &HashMap::from([("novops_secret", "s3cret")])
        ).await?;

        let outputs = load_env_for("hashivault", "dev").await?;

        assert_eq!(outputs.variables.get("HASHIVAULT_KV_V2_TEST").unwrap().value, "s3cret");

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