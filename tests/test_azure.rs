mod test_lib;

use test_lib::{load_env_for, test_setup};
use log::info;

#[tokio::test]
async fn test_azure_keyvault() -> Result<(), anyhow::Error> {

    test_setup().await?;

    let expect_values = "v3rySecret!";

    let outputs = load_env_for("azure_keyvault", "dev").await?;

    info!("test_azure_keyvault: Found variables: {:?}", outputs.variables);
    info!("test_azure_keyvault: Found files: {:?}", outputs.files);

    assert_eq!(outputs.variables.get("AZ_KEYVAULT_SECRET_VAR").unwrap().value, expect_values);
    assert_eq!(outputs.files.get("/tmp/AZ_KEYVAULT_SECRET_FILE").unwrap().content, expect_values.as_bytes());
    Ok(())
}
