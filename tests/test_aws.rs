mod test_utils;


#[cfg(test)]
mod tests {
    use novops::{load_context_and_resolve, NovopsArgs};
    use novops::modules::aws::config::{get_iam_client, AwsClientConfig};

    #[tokio::test]
    async fn test_assume_role() -> Result<(), anyhow::Error> {

        setup_test_env().await?;
        ensure_test_role_exists("NovopsTestAwsAssumeRole").await?;        

        // Load config and expect temporary credentials
        let args = NovopsArgs { 
            config: "tests/.novops.aws.yml".to_string(), 
            env: Some("dev".to_string()), 
            working_directory: None,
            symlink: None
        };

        let outputs = load_context_and_resolve(&args).await?;

        assert_eq!(outputs.variables[0].name, "AWS_ACCESS_KEY_ID");
        assert_eq!(outputs.variables[1].name, "AWS_SECRET_ACCESS_KEY");
        assert_eq!(outputs.variables[2].name, "AWS_SESSION_TOKEN");

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

    async fn setup_test_env() -> Result<(), anyhow::Error> {
        // use known AWS config
        let aws_config = std::env::current_dir()?.join("tests/aws/config");
        let aws_creds = std::env::current_dir()?.join("tests/aws/credentials");

        std::env::set_var("AWS_CONFIG_FILE", aws_config.to_str().unwrap());
        std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", &aws_creds.to_str().unwrap());

        Ok(())
    }

}