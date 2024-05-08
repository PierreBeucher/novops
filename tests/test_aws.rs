pub mod test_lib;

use std::io::Write;

use aws_sdk_s3::{primitives::ByteStream, types::{BucketLocationConstraint, CreateBucketConfiguration}};
use novops::modules::aws::client::{get_s3_client, get_secretsmanager_client, get_ssm_client};
use aws_sdk_ssm::types::ParameterType;
use aws_smithy_types::Blob;
use tempfile::NamedTempFile;
use test_lib::{load_env_for, test_setup, aws_ensure_role_exists, aws_test_config};
use log::info;
use base64::prelude::*;

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

    // String
    let pstring_value = "novops-string-test";
    ensure_test_ssm_param_exists("novops-test-ssm-param-string", pstring_value, ParameterType::String).await?;

    // SecureString
    let psecurestring_value = "novops-string-test-secure";
    ensure_test_ssm_param_exists("novops-test-ssm-param-secureString", psecurestring_value, ParameterType::SecureString).await?;

    let outputs = load_env_for("aws_ssm", "dev").await?;

    info!("test_ssmparam: Found variables: {:?}", outputs.variables);

    assert_eq!(outputs.variables.get("SSM_PARAM_STORE_TEST_STRING").unwrap().value, pstring_value);
    assert_eq!(outputs.variables.get("SSM_PARAM_STORE_TEST_SECURE_STRING").unwrap().value, psecurestring_value);
    assert_ne!(outputs.variables.get("SSM_PARAM_STORE_TEST_SECURE_STRING_NO_DECRYPT").unwrap().value, psecurestring_value);

    Ok(())

}

#[tokio::test]
async fn test_secretsmanager() -> Result<(), anyhow::Error> {

    // Prepare env and dummy secret
    test_setup().await?;

    let expect_string = "Some-String-data?1548a~#{[[".to_string();
    let expect_binary = vec![240, 159, 146, 150]; // 💖
    ensure_test_secret_exists("novops-test-secretsmanager-string", Some(expect_string.clone()), None).await?;
    ensure_test_secret_exists("novops-test-secretsmanager-binary", None, Some(expect_binary.clone())).await?;
    
    let outputs = load_env_for("aws_secretsmanager", "dev").await?;

    info!("test_secretsmanager: Found variables: {:?}", outputs.variables);
    info!("test_secretsmanager: Found files: {:?}", outputs.files);

    let binary_var_value = BASE64_STANDARD.decode(outputs.variables.get("SECRETSMANAGER_VAR_BINARY").unwrap().value.clone())?;
    let binary_file_content = BASE64_STANDARD.decode(outputs.files.get("/tmp/SECRETSMANAGER_FILE_BINARY").unwrap().content.clone())?;

    assert_eq!(outputs.variables.get("SECRETSMANAGER_VAR_STRING").unwrap().value, expect_string);
    assert_eq!(binary_var_value, expect_binary);
    assert_eq!(outputs.files.get("/tmp/SECRETSMANAGER_FILE_STRING").unwrap().content, expect_string.as_bytes());
    assert_eq!(binary_file_content, expect_binary);

    Ok(())
}

async fn ensure_test_ssm_param_exists(pname: &str, pvalue: &str, ptype: ParameterType) -> Result<(), anyhow::Error> {
    let client = get_ssm_client(&aws_test_config()).await?;

    info!("PUT SSM param: {}", pname);

    let r = client.put_parameter()
        .name(pname)
        .overwrite(true)
        .value(pvalue)
        .r#type(ptype)
        .send().await?;

    info!("SSM param PUT {} response: {:?}", pname, r);

    Ok(())
}

async fn ensure_test_secret_exists(sname: &str, string_value: Option<String>, binary_value: Option<Vec<u8>>) 
        -> Result<(), anyhow::Error> {
    let client = get_secretsmanager_client(&aws_test_config()).await?;

    if let Ok(r) = client.describe_secret().secret_id(sname).send().await {
        info!("Secret {} already exists, deleting...", sname);

        client.delete_secret()
            .secret_id(r.arn().unwrap())
            .force_delete_without_recovery(true)
            .send().await?;
    }

    let r = client.create_secret()
        .name(sname)
        .set_secret_string(string_value)
        .set_secret_binary(binary_value.map(Blob::new))
        .send().await?;

    info!("Create AWS secret {} response: {:?}", sname, r);

    Ok(())
}

#[tokio::test]
async fn test_s3_object() -> Result<(), anyhow::Error> {

    // Prepare env and dummy secret
    test_setup().await?;

    ensure_test_s3_object_exists("novops-test-bucket", "path/to/var", "variable-content".as_bytes()).await?;
    ensure_test_s3_object_exists("novops-test-bucket", "path/to/file", "file-content".as_bytes()).await?;

    let outputs = load_env_for("aws_s3_object", "dev").await?;
    assert_eq!(outputs.variables.get("S3_OBJECT_AS_VAR").unwrap().value, "variable-content");
    assert_eq!(outputs.files.get("/tmp/S3_OBJECT_AS_FILE").unwrap().content, "file-content".as_bytes());

    Ok(())
}

async fn ensure_test_s3_object_exists(bucket_name: &str, object_key: &str, content: &[u8]) -> Result<(), anyhow::Error> {

    let client = get_s3_client(&aws_test_config()).await?;
    
    let bucket_exists = match client.head_bucket().bucket(bucket_name).send().await {
        Ok(_) => true,
        Err(_) => false,
    };

    if !bucket_exists {
        info!("creating bucket {:}", bucket_name);

        let constraint = BucketLocationConstraint::from("eu-west-3");

        let cfg = CreateBucketConfiguration::builder()
            .location_constraint(constraint)
            .build();

        client.create_bucket()
            .bucket(bucket_name)
            .create_bucket_configuration(cfg)
            .send().await?;

    } else {
        info!("Bucket {:} already exists", bucket_name);
    }

    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(content)?;

    let body = ByteStream::from_path(temp_file.path()).await?;

    info!("Putting object {:} in {:}", object_key, bucket_name);

    client.put_object()
        .bucket(bucket_name)
        .key(object_key)
        .body(body)
        .send()
        .await?;

    Ok(())
}