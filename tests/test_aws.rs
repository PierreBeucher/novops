mod test_utils;


#[cfg(test)]
mod tests {
    use novops::modules::aws::config::{get_iam_client, get_ssm_client, get_secretsmanager_client, AwsClientConfig};
    use aws_sdk_ssm::model::ParameterType;
    use aws_smithy_types::Blob;
    use crate::test_utils::load_env_for;

    use log::{info, debug};

    #[tokio::test]
    async fn test_assume_role() -> Result<(), anyhow::Error> {

        setup_test_env().await?;
        ensure_test_role_exists("NovopsTestAwsAssumeRole").await?;        

        let outputs = load_env_for("aws_assumerole", "dev").await?;

        info!("test_assume_role: Found variables: {:?}", outputs.variables);

        // STS temporary keys starts with ASIA https://docs.aws.amazon.com/STS/latest/APIReference/API_GetAccessKeyInfo.html
        assert!(outputs.variables.get("AWS_ACCESS_KEY_ID").unwrap().value.starts_with("ASIA"));
        assert!(outputs.variables.get("AWS_SECRET_ACCESS_KEY").unwrap().value.len() > 0);
        assert!(outputs.variables.get("AWS_SESSION_TOKEN").unwrap().value.len() > 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_ssm_param() -> Result<(), anyhow::Error> {

        setup_test_env().await?;

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
        setup_test_env().await?;

        let expect_string = "Some-String-data?1548a~#{[[".to_string();
        let expect_binary = vec![240, 159, 146, 150]; // 💖
        ensure_test_secret_exists("novops-test-secretsmanager-string", Some(expect_string.clone()), None).await?;
        ensure_test_secret_exists("novops-test-secretsmanager-binary", None, Some(expect_binary.clone())).await?;
        
        let outputs = load_env_for("aws_secretsmanager", "dev").await?;

        info!("test_secretsmanager: Found variables: {:?}", outputs.variables);
        info!("test_secretsmanager: Found files: {:?}", outputs.files);

        assert_eq!(outputs.variables.get("SECRETSMANAGER_VAR_STRING").unwrap().value, expect_string);
        assert_eq!(outputs.variables.get("SECRETSMANAGER_VAR_BINARY").unwrap().value.as_bytes(), expect_binary);
        assert_eq!(outputs.files.get("/tmp/SECRETSMANAGER_FILE_STRING").unwrap().content, expect_string.as_bytes());
        assert_eq!(outputs.files.get("/tmp/SECRETSMANAGER_FILE_BINARY").unwrap().content, expect_binary);

        Ok(())
    }

    /**
     * Variable input cannot be used for NON-UTF8 data yet. Let's test for proper error message. 
     */
    #[tokio::test]
    async fn test_secretsmanager_non_utf8_variable() -> Result<(), anyhow::Error> {

        // Prepare env and dummy secret
        setup_test_env().await?;

        let non_utf8_binary = vec![0, 159, 146, 150];
        ensure_test_secret_exists("novops-test-secretsmanager-binary-non-utf8", None, Some(non_utf8_binary.clone())).await?;
        
        let outputs = load_env_for("aws_secretsmanager_var_nonutf8", "dev").await;
        outputs.expect_err("Expected non UTF-8 binary data to provoke an error.");

        Ok(())
    }

    /**
     * create test IAM role to impersonate, delete it first if already exists
     */
    async fn ensure_test_role_exists(role_name: &str) -> Result<(), anyhow::Error> {
        let client = get_iam_client(&aws_test_config()).await?;
        let existing_role_result = client.get_role().role_name(role_name).send().await;

        match existing_role_result {
            Ok(_) => {  // role exists, clean before running test
                client.delete_role().role_name(role_name).send().await?;
            }
            Err(_) => {}, // role do not exists, do nothing
        }

        client.create_role()
            .role_name(role_name)
            .assume_role_policy_document(r#"{
                "Version": "2012-10-17",
                "Statement": [
                    {
                        "Effect": "Allow",
                        "Principal": {
                            "AWS": "111122223333"
                        },
                        "Action": "sts:AssumeRole"
                    }
                ]
            }"#)
            .send().await.expect("Valid create role response");
        
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

        match client.describe_secret().secret_id(sname).send().await {
            Ok(r) => {
                info!("Secret {} already exists, deleting...", sname);

                client.delete_secret()
                    .secret_id(r.arn().unwrap())
                    .force_delete_without_recovery(true)
                    .send().await?;
            },
            Err(_) => {}, // secret does not exists, ignore
        }

        let r = client.create_secret()
            .name(sname)
            .set_secret_string(string_value)
            .set_secret_binary(binary_value.map(|b| Blob::new(b)))
            .send().await?;

        info!("Create AWS secret {} response: {:?}", sname, r);

        Ok(())
    }

    async fn setup_test_env() -> Result<(), anyhow::Error> {

        // Allow multiple invocation of logger
        match env_logger::try_init() {
            Ok(_) => {},
            Err(e) => {debug!("env_logger::try_nit() error: {:?}", e)},
        };
        
        // use known AWS config
        let aws_config = std::env::current_dir()?.join("tests/aws/config");
        let aws_creds = std::env::current_dir()?.join("tests/aws/credentials");

        std::env::set_var("AWS_CONFIG_FILE", aws_config.to_str().unwrap());
        std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", &aws_creds.to_str().unwrap());

        Ok(())
    }

    fn aws_test_config() -> AwsClientConfig{
        let mut aws_conf = AwsClientConfig::default();
        aws_conf.endpoint("http://localhost:4566/");
        return aws_conf;
    }

}