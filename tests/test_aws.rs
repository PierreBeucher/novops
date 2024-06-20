pub mod test_lib;

use test_lib::{load_env_for, test_setup, aws_ensure_role_exists};
use log::info;

#[tokio::test]
async fn test_assume_role() -> Result<(), anyhow::Error> {

    test_setup().await?;
    aws_ensure_role_exists("NovopsTestAwsAssumeRole").await?;        

    let outputs = load_env_for("aws_assumerole", "dev").await?;

    info!("test_assume_role: Found variables: {:?}", outputs.variables);

    assert!(!outputs.variables.get("AWS_ACCESS_KEY_ID").unwrap().value.is_empty());
    assert!(!outputs.variables.get("AWS_SECRET_ACCESS_KEY").unwrap().value.is_empty());
    assert!(!outputs.variables.get("AWS_SESSION_TOKEN").unwrap().value.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_ssm_param() -> Result<(), anyhow::Error> {

    test_setup().await?;

    let pstring_value = "novops-string-test";
    let psecurestring_value = "novops-string-test-secure";

    let outputs = load_env_for("aws_ssm", "dev").await?;

    info!("test_ssmparam: Found variables: {:?}", outputs.variables);

    assert_eq!(outputs.variables.get("SSM_PARAM_STORE_TEST_STRING").unwrap().value, pstring_value);
    assert_eq!(outputs.variables.get("SSM_PARAM_STORE_TEST_SECURE_STRING").unwrap().value, psecurestring_value);
    assert_ne!(outputs.variables.get("SSM_PARAM_STORE_TEST_SECURE_STRING_NO_DECRYPT").unwrap().value, psecurestring_value);

    Ok(())

}

#[tokio::test]
async fn test_secretsmanager() -> Result<(), anyhow::Error> {

    test_setup().await?;

    let expect_string = "Some-String-data?1548a~#{[[".to_string();
    let expect_binary = vec![240, 159, 146, 150]; // 💖
    
    let outputs = load_env_for("aws_secretsmanager", "dev").await?;

    info!("test_secretsmanager: Found variables: {:?}", outputs.variables);
    info!("test_secretsmanager: Found files: {:?}", outputs.files);

    assert_eq!(outputs.variables.get("SECRETSMANAGER_VAR_STRING").unwrap().value, expect_string, "Diff {:?} - {:?}", outputs.variables.get("SECRETSMANAGER_VAR_STRING").unwrap().value, expect_string);
    assert_eq!(outputs.variables.get("SECRETSMANAGER_VAR_BINARY").unwrap().value.as_bytes(), expect_binary);
    assert_eq!(outputs.files.get("/tmp/SECRETSMANAGER_FILE_STRING").unwrap().content, expect_string.as_bytes(), "Diff {:?} - {:?}", outputs.files.get("/tmp/SECRETSMANAGER_FILE_STRING").unwrap().content, expect_string.as_bytes());
    assert_eq!(outputs.files.get("/tmp/SECRETSMANAGER_FILE_BINARY").unwrap().content, expect_binary);

    Ok(())
}

#[tokio::test]
async fn test_s3_object() -> Result<(), anyhow::Error> {

    test_setup().await?;

    let outputs = load_env_for("aws_s3_object", "dev").await?;
    assert_eq!(outputs.variables.get("S3_OBJECT_AS_VAR").unwrap().value, "variable-content");
    assert_eq!(outputs.files.get("/tmp/S3_OBJECT_AS_FILE").unwrap().content, "file-content".as_bytes());

    Ok(())
}
