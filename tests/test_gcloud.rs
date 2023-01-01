mod test_utils;


#[cfg(test)]
mod tests {
    use crate::test_utils;
    use log::{info};


    #[tokio::test]
    async fn test_gcloud_secretmanager() -> Result<(), anyhow::Error> {

        test_utils::test_setup();

        let expect = "RESULT:projects/pierre-sandbox-372512/secrets/TestSecret/versions/latest";
        let outputs = test_utils::load_env_dryrun_for("gcloud_secretmanager", "dev").await?;

        info!("test_gcloud_secretmanager: Found variables: {:?}", outputs.variables);
        info!("test_gcloud_secretmanager: Found files: {:?}", outputs.files);

        assert_eq!(outputs.variables.get("SECRETMANAGER_VAR_STRING").unwrap().value, expect);
        assert_eq!(outputs.files.get("/tmp/gcloud_SECRETMANAGER_VAR_FILE").unwrap().content, expect.as_bytes());

        Ok(())
    }
}