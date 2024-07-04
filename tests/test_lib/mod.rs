use log::debug;
use novops::core::{NovopsConfig, NovopsConfigDefault, NovopsConfigFile, NovopsContext};
use novops::modules::aws::{client::get_iam_client, config::AwsClientConfig};
use novops::{load_context_and_resolve, NovopsLoadArgs, NovopsOutputs};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[allow(dead_code)]
pub const TEST_DIR: &str = "tests/output";

/**
 * Create a temporary dir to be used for test
 * Mostly used as Novops workdir named after test
 */
#[allow(dead_code)]
pub fn clean_and_setup_test_dir(test_name: &str) -> Result<PathBuf, anyhow::Error> {
    let test_output_dir = env::current_dir()?.join(TEST_DIR).join(test_name);

    if test_output_dir.exists() {
        fs::remove_dir_all(&test_output_dir)?;
    }

    fs::create_dir_all(&test_output_dir)?;
    fs::set_permissions(&test_output_dir, Permissions::from_mode(0o700))?;
    Ok(test_output_dir)
}

/**
 * Load Novops environment for tests/.novops.<conf_name>.yml
 */
#[allow(dead_code)]
pub async fn load_env_for(conf_name: &str, env: &str) -> Result<NovopsOutputs, anyhow::Error> {
    _load_env_for(conf_name, env, false).await
}

#[allow(dead_code)]
pub async fn load_env_dryrun_for(
    conf_name: &str,
    env: &str,
) -> Result<NovopsOutputs, anyhow::Error> {
    _load_env_for(conf_name, env, true).await
}

#[allow(dead_code)]
async fn _load_env_for(
    conf_name: &str,
    env: &str,
    dry_run: bool,
) -> Result<NovopsOutputs, anyhow::Error> {
    let args = NovopsLoadArgs {
        config: Some(format!("tests/.novops.{}.yml", conf_name)),
        env: Some(env.to_string()),
        working_directory: None,
        skip_working_directory_check: Some(false),
        dry_run: Some(dry_run),
    };

    let outputs = load_context_and_resolve(&args).await?;

    Ok(outputs)
}

/**
 * Perform test setup before running tests
 * - Use common logging
 * - Use test AWS config
 */
#[allow(dead_code)]
pub async fn test_setup() -> Result<(), anyhow::Error> {
    // enable logger
    match env_logger::try_init() {
        Ok(_) => {}
        Err(e) => {
            debug!("env_logger::try_init() error: {:?}", e)
        }
    };

    // use known AWS config
    let aws_config = std::env::current_dir()?.join("tests/setup/aws/config");
    let aws_creds = std::env::current_dir()?.join("tests/setup/aws/credentials");

    std::env::set_var("AWS_CONFIG_FILE", aws_config.to_str().unwrap());
    std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", aws_creds.to_str().unwrap());

    // known age keys
    std::env::set_var("SOPS_AGE_KEY_FILE", "tests/setup/sops/age1");

    Ok(())
}

#[allow(dead_code)]
pub fn aws_test_config() -> AwsClientConfig {
    let mut aws_conf = AwsClientConfig::default();
    aws_conf.endpoint("http://localhost:4566/"); // Localstack
    aws_conf
}

/**
 * create test IAM role to impersonate, delete it first if already exists
 */
#[allow(dead_code)]
pub async fn aws_ensure_role_exists(role_name: &str) -> Result<(), anyhow::Error> {
    let client = get_iam_client(&aws_test_config()).await?;
    let existing_role_result = client.get_role().role_name(role_name).send().await;

    if existing_role_result.is_ok() {
        // role exists, clean before running test
        client.delete_role().role_name(role_name).send().await?;
    }

    client
        .create_role()
        .role_name(role_name)
        .assume_role_policy_document(
            r#"{
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
        }"#,
        )
        .send()
        .await
        .expect("Valid create role response");

    Ok(())
}

#[allow(dead_code)]
pub fn create_dummy_context() -> NovopsContext {
    NovopsContext {
        env_name: String::from("dev"),
        app_name: String::from("test-empty"),
        workdir: PathBuf::from("/tmp"),
        config_file_data: NovopsConfigFile {
            name: Some(String::from("test-empty")),
            environments: HashMap::new(),
            config: Some(NovopsConfig {
                default: Some(NovopsConfigDefault {
                    environment: Some(String::from("dev")),
                }),
                hashivault: None,
                aws: None,
            }),
        },
        env_var_filepath: PathBuf::from("/tmp/vars"),
        dry_run: false,
    }
}
