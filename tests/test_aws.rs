mod test_utils;


#[cfg(test)]
mod tests {
    use novops::{load_context_and_resolve, NovopsArgs};
    use novops::modules::aws::config::{get_iam_client, get_ssm_client, AwsClientConfig};
    use aws_sdk_ssm::model::ParameterType;
    use crate::test_utils::load_env_for_module;

    use log::info;

    #[tokio::test]
    async fn test_assume_role() -> Result<(), anyhow::Error> {

        setup_test_env().await?;
        ensure_test_role_exists("NovopsTestAwsAssumeRole").await?;        

        let outputs = load_env_for_module("aws", "dev").await?;

        assert_eq!(outputs.variables[3].name, "AWS_ACCESS_KEY_ID");
        assert_eq!(outputs.variables[4].name, "AWS_SECRET_ACCESS_KEY");
        assert_eq!(outputs.variables[5].name, "AWS_SESSION_TOKEN");

        Ok(())
    }

    #[tokio::test]
    async fn test_ssm_param() -> Result<(), anyhow::Error> {

        setup_test_env().await?;

        // String
        let pstring_name = "novops-test-ssm-param-string";
        let pstring_value = "novops-string-test";
        ensure_test_ssm_param_exists(pstring_name, pstring_value, ParameterType::String).await?;

        // SecureString
        let psecurestring_name = "novops-test-ssm-param-secureString";
        let psecurestring_value = "novops-string-test-secure";
        ensure_test_ssm_param_exists(psecurestring_name, psecurestring_value, ParameterType::SecureString).await?;

        let outputs = load_env_for_module("aws", "dev").await?;

        info!("Found variables: {:?}", outputs.variables);

        assert_eq!(outputs.variables[0].name, "SSM_PARAM_STORE_TEST_STRING");
        assert_eq!(outputs.variables[0].value, pstring_value);

        assert_eq!(outputs.variables[1].name, "SSM_PARAM_STORE_TEST_SECURE_STRING");
        assert_eq!(outputs.variables[1].value, psecurestring_value);

        assert_eq!(outputs.variables[2].name, "SSM_PARAM_STORE_TEST_SECURE_STRING_NO_DECRYPT");
        assert_ne!(outputs.variables[2].value, psecurestring_value);
        
        Ok(())

    }

    /**
     * create test IAM role to impersonate, delete it first if already exists
     */
    async fn ensure_test_role_exists(role_name: &str) -> Result<(), anyhow::Error> {
        let mut aws_conf = AwsClientConfig::default();
        aws_conf.endpoint("http://localhost:4566/");
        
        let client = get_iam_client(&aws_conf).await?;

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
        let mut aws_conf = AwsClientConfig::default();
        aws_conf.endpoint("http://localhost:4566/");
        
        let client = get_ssm_client(&aws_conf).await?;

        client.put_parameter()
            .name(pname)
            .overwrite(true)
            .value(pvalue)
            .r#type(ptype)
            .send().await?;

        Ok(())
    }

    async fn setup_test_env() -> Result<(), anyhow::Error> {
        // use known AWS config
        let aws_config = std::env::current_dir()?.join("tests/aws/config");
        let aws_creds = std::env::current_dir()?.join("tests/aws/credentials");

        std::env::set_var("AWS_CONFIG_FILE", aws_config.to_str().unwrap());
        std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", &aws_creds.to_str().unwrap());

        Ok(())
    }

}