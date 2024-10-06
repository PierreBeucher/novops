pub mod test_lib;

use chrono::Utc;
use test_lib::{load_env_for, test_setup};
use log::info;

#[tokio::test]
async fn test_assume_role() -> Result<(), anyhow::Error> {

    test_setup().await?;

    let outputs = load_env_for("aws_assumerole", "dev").await?;

    info!("test_assume_role: Found variables: {:?}", outputs.variables);

    assert!(!outputs.variables.get("AWS_ACCESS_KEY_ID").unwrap().value.is_empty());
    assert!(!outputs.variables.get("AWS_SECRET_ACCESS_KEY").unwrap().value.is_empty());
    assert!(!outputs.variables.get("AWS_SESSION_TOKEN").unwrap().value.is_empty());
    assert!(!outputs.variables.get("AWS_SESSION_EXPIRATION").unwrap().value.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_assume_role_duration() -> Result<(), anyhow::Error> {

    test_setup().await?;

    let now = Utc::now().timestamp();
    let outputs = load_env_for("aws_assumerole", "integ").await?;

    info!("test_assume_role_duration: Found variables: {:?}", outputs.variables);

    // Check creds duration
    let access_key = outputs.variables.get("AWS_ACCESS_KEY_ID").unwrap().clone().value;
    let secret_key = outputs.variables.get("AWS_SECRET_ACCESS_KEY").unwrap().clone().value;
    let session_token = outputs.variables.get("AWS_SESSION_TOKEN").unwrap().clone().value;
    let session_token_exp = outputs.variables.get("AWS_SESSION_EXPIRATION").unwrap().clone().value;

    assert!(!access_key.is_empty());
    assert!(!secret_key.is_empty());
    assert!(!session_token.is_empty());
    assert!(!session_token_exp.is_empty());

    // Parse the session expiration timestamp
    // Expect actual expiration timestamp to be 15 min from now
    // allow a few seconds diff as it may not be perfect depending on real generation time
    let session_token_exp_number:i64 = session_token_exp.parse().unwrap();
    let expected_min_expiration = now + 900 - 2;
    let expected_max_expiration = now + 900 + 5;
    
    assert!(session_token_exp_number > expected_min_expiration && session_token_exp_number < expected_max_expiration, 
        "Session token expiration should be in around 15 minutes from now, got significan diff. 
        Expected expiration to ~900s from now ({:}) be between {:} and {:}, got {:}", now, expected_min_expiration, expected_max_expiration, session_token_exp_number);



    Ok(())
}

#[tokio::test]
async fn test_assume_role_identity_cache_load_timeout() -> Result<(), anyhow::Error> {

    test_setup().await?;

    // Takes a few seconds to run but should not timeout
    // No error is sufficient to validate test
    let outputs = load_env_for("aws_assumerole_id_cache_load_timeout", "timeout").await?;

    let access_key = outputs.variables.get("AWS_ACCESS_KEY_ID").unwrap().clone().value;

    assert!(!access_key.is_empty());

    Ok(())
}


#[tokio::test]
async fn test_assume_role_identity_cache_load_timeout_error() -> Result<(), anyhow::Error> {

    test_setup().await?;

    // Loading AWS credentials should cause a timeout
    let outputs = load_env_for("aws_assumerole_id_cache_load_timeout_short", "timeout").await;

    assert!(outputs.is_err(), "Expected timeout to occur as per identity cache load timeout config.");

    let error = outputs.err().unwrap();
    let error_str = format!("{:?}", error); // a bit hard way to print all error messages in a single string

    info!("test_assume_role_identity_cache_load_timeout_error: Got expected error: {}", &error_str);

    assert!(error_str.contains("identity resolver timed out after"), "Error message did not contain the expected 'identity resolver timed out after' string.");

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
