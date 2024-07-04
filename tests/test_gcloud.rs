mod test_lib;

use test_lib::{test_setup, load_env_for};
use log::info;


#[tokio::test]
async fn test_gcloud_secretmanager() -> Result<(), anyhow::Error> {

    test_setup().await?;

    let expect = "very!S3cret";
    let outputs = load_env_for("gcloud_secretmanager", "dev").await?;

    info!("test_gcloud_secretmanager: Found variables: {:?}", outputs.variables);
    info!("test_gcloud_secretmanager: Found files: {:?}", outputs.files);

    assert_eq!(outputs.variables.get("SECRETMANAGER_VAR_STRING").unwrap().value, expect);
    assert_eq!(outputs.files.get("/tmp/gcloud_SECRETMANAGER_VAR_FILE").unwrap().content, expect.as_bytes());

    Ok(())
}