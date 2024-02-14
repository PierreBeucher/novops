mod test_lib;

use novops::modules::variables::VariableOutput;
use novops::{make_context, NovopsLoadArgs, 
    load_environment_write_vars, prepare_exec_command, should_error_tty, 
    list_environments, list_outputs_for_environment, check_working_dir_permissions,
    get_config_file_path};
use novops::core::{NovopsContext, NovopsConfig, NovopsConfigFile, NovopsConfigDefault, NovopsEnvironmentInput};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use log::info;
use test_lib::{clean_and_setup_test_dir, TEST_DIR, load_env_dryrun_for, test_setup};

const CONFIG_EMPTY: &str = "tests/.novops.empty.yml";
const CONFIG_STANDALONE: &str = "tests/.novops.plain-strings.yml";

/**
 * Test a config is properly loaded into a NovopsContext
 */
#[tokio::test]
async fn test_load_simple_config() -> Result<(), anyhow::Error>{
    test_setup().await?;

    let workdir = clean_and_setup_test_dir("test_load_simple_config")?;

    let args = NovopsLoadArgs {
        config: Some(String::from(CONFIG_EMPTY)),
        env: Some(String::from("dev")),
        working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()),
        skip_working_directory_check: Some(false),
        dry_run: None
    };
    let result = make_context(&args).await?;

    info!("Result: {:?}", result);

    assert_eq!(result, 
        NovopsContext {
            env_name: String::from("dev"),
            app_name: String::from("test-empty"),
            workdir: workdir.clone(),
            config_file_data: NovopsConfigFile{
                name: Some(String::from("test-empty")),
                environments: HashMap::from([
                    (String::from("dev"), NovopsEnvironmentInput {
                        variables: None,
                        files: None,
                        aws: None,
                        hashivault: None,
                        sops_dotenv: None,
                    })
                ]),
                config: Some(NovopsConfig { 
                    default: Some(NovopsConfigDefault {
                            environment: Some(String::from("dev"))
                    }), 
                    hashivault: None,
                    aws: None
                })
            },
            env_var_filepath: workdir.join("vars"),
            dry_run: false
        }
    );

    Ok(())
}


/**
 * Run Novops and check expected files and variables are generated:
 * - var file with expected export value (in any order)
 * - Generated files and content
 */
#[tokio::test]
async fn test_simple_run() -> Result<(), anyhow::Error>{
    test_setup().await?;

    let workdir = clean_and_setup_test_dir("test_simple_run")?;

    load_environment_write_vars(&NovopsLoadArgs { 
            config: Some(String::from(CONFIG_STANDALONE)),
            env: Some(String::from("dev")), 
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()),
            skip_working_directory_check: Some(false),
            dry_run: None
        },
        &Some(String::from(".envrc")),
        &String::from("dotenv-export"),
        true
    ).await?;   

    let expected_var_file = PathBuf::from(&workdir).join("vars");
    let expected_var_content = fs::read_to_string(expected_var_file)?;
    
    let expected_file_dog_path = PathBuf::from(&workdir).join("file_1811bdd29f2cfe95e6e23402e2390fa1012708fc52ef8b8a29ee540b1c481534");
    let expected_file_dog_content = fs::read_to_string(&expected_file_dog_path)?;
    let file_dog_metadata = fs::metadata(&expected_file_dog_path)?;
    let file_dog_mode = file_dog_metadata.permissions().mode();

    let expected_file_cat_path = PathBuf::from("/tmp/novops_cat");
    let expected_file_cat_content = fs::read_to_string(expected_file_cat_path)?;

    // Expect to match content of CONFIG_STANDALONE
    // use r#"_"# for raw string literal
    // check if our file content contains expected export
    // naïve but sufficient for our needs
    assert!(&expected_var_content.contains(r#"export SPECIAL_CHARACTERS='special_char_'"'"'!?`$abc_#~%*µ€{}[]-°+@à^ç=\'"#));
    assert!(&expected_var_content.contains( "export MY_APP_HOST='localhost'"));
    assert!(&expected_var_content.contains( &format!("export DOG_PATH='{:}'",
        &expected_file_dog_path.clone().into_os_string().into_string().unwrap())));
    assert!(&expected_var_content.contains( "export NOVOPS_CAT_VAR='/tmp/novops_cat'"));

    // expect file permission to be 0600 (user readonly)
    // use a bitwise AND on ocal value to check for user-only permission 0600
    assert_eq!(file_dog_metadata.permissions().mode() & 0o777, 0o600, "Expected {:?} to have permission {:o}, found {:o}", 
        &expected_file_dog_path, 0o600, &file_dog_mode);
    
    
    assert_eq!(expected_file_dog_content, "woof");
    assert_eq!(expected_file_cat_content, "meow");
    
    Ok(())  
    
}

#[tokio::test]
async fn test_symlink_flag() -> Result<(), anyhow::Error> {
    test_setup().await?;

    let workdir = clean_and_setup_test_dir("test_symlink_flag")?;

    let expect_symlink_at = PathBuf::from(TEST_DIR).join("test-symlink");
    load_environment_write_vars(&NovopsLoadArgs { 
            config: Some(String::from(CONFIG_STANDALONE)),
            env: Some(String::from("dev")),
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()),
            skip_working_directory_check: Some(false),
            dry_run: None
        },
        &Some(expect_symlink_at.clone().into_os_string().into_string().unwrap()),
        &String::from("dotenv-export"),
        true
    ).await?;

    let symlink_metadata = fs::symlink_metadata(&expect_symlink_at)?;
    assert!(symlink_metadata.is_symlink(), "{:?} does not seem to be a symlink: {:?}", &expect_symlink_at, symlink_metadata);

    // symlink is expected to point to var file under our working directory
    let symlink_dest = fs::read_link(&expect_symlink_at).unwrap();
    assert_eq!(symlink_dest, PathBuf::from(&workdir).join("vars"), "Symlink destination is not as expected");

    // run again with different symlink dest
    // expect existing symlink to be overriden
    let workdir_override = clean_and_setup_test_dir("test_symlink_flag_override")?;
    load_environment_write_vars(&NovopsLoadArgs { 
            config: Some(String::from(CONFIG_STANDALONE)),
            env: Some(String::from("staging")),
            working_directory: Some(workdir_override.clone().into_os_string().into_string().unwrap()), 
            skip_working_directory_check: Some(false),
            dry_run: None
        },
        &Some(expect_symlink_at.clone().into_os_string().into_string().unwrap()),
        &String::from("dotenv-export"),
        true
    ).await?;

    let overriden_symlink_dest = fs::read_link(&expect_symlink_at).unwrap();
    assert_eq!(overriden_symlink_dest, PathBuf::from(&workdir_override).join("vars"), "Symlink destination is not as expected");

    Ok(())
}

/**
 * Ensure that a file/dir at symlink path result in failure
 */
#[tokio::test]
async fn test_symlink_no_file_override() -> Result<(), anyhow::Error> {
    test_setup().await?;

    let workdir = clean_and_setup_test_dir("test_symlink_no_file_override")?;

    // create dummy file, we don't want it erased by symlink
    let symlink_path = PathBuf::from(&workdir).join("file-dont-override");
    fs::File::create(&symlink_path)?;
    
    // expect error as we cannot erase existing file
    let result = load_environment_write_vars(&NovopsLoadArgs { 
            config: Some(String::from(CONFIG_STANDALONE)),
            env: Some(String::from("dev")), 
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()),
            skip_working_directory_check: Some(false),
            dry_run: None
        }, 
        &Some(symlink_path.clone().into_os_string().into_string().unwrap()),
        &String::from("dotenv-export"),
        true
    ).await;

    result.expect_err("Expected an error when loading with symlink trying to override existing file, got OK.");

    Ok(())
}

/**
 * Check all modules with dry run
 * Having non-empty values and no errors is enough
 */
#[tokio::test]
async fn test_dry_run() -> Result<(), anyhow::Error> {
    test_setup().await?;
    
    let result = load_env_dryrun_for("all-modules", "dev").await?;

    info!("test_dry_run: Found variables: {:?}", &result.variables);
    info!("test_dry_run: Found files: {:?}", &result.files);


    assert!(!result.variables.get("VAR").unwrap().value.is_empty());
    assert!(!result.variables.get("AWS_SECRETMANAGER").unwrap().value.is_empty());
    assert!(!result.variables.get("AWS_SSM_PARAMETER").unwrap().value.is_empty());
    assert!(!result.variables.get("HASHIVAULT_KV_V2").unwrap().value.is_empty());
    assert!(!result.variables.get("BITWARDEN").unwrap().value.is_empty());
    assert!(!result.variables.get("GCLOUD_SECRETMANAGER").unwrap().value.is_empty());
    assert!(!result.files.get("/tmp/novopsfile").unwrap().content.is_empty());

    // aws.assumerole
    assert!(!result.variables.get("AWS_ACCESS_KEY_ID").unwrap().value.is_empty());
    assert!(!result.variables.get("AWS_SESSION_TOKEN").unwrap().value.is_empty());
    assert!(!result.variables.get("AWS_SECRET_ACCESS_KEY").unwrap().value.is_empty());

    Ok(())
}

/**
 * Check all modules with dry run
 * Having non-empty values and no errors is enough
 */
#[tokio::test]
async fn test_run_prepare_process() -> Result<(), anyhow::Error> {
    test_setup().await?;


    let cmd =String::from("sh");
    let arg1 =String::from("-c");
    let arg2 =String::from("echo foo");
    let args = vec![&cmd, &arg1, &arg2];

    let var1 = "FOO";
    let val1 = "barzzz";
    let vars : Vec<VariableOutput> = vec![VariableOutput{ 
        name: String::from(var1), 
        value: String::from(val1)
    }];

    let result = prepare_exec_command(args, &vars);
    
    assert_eq!(result.get_envs().len(), 1);

    let os_vars : Vec<(&OsStr, Option<&OsStr>)> = result.get_envs().collect();
    assert_eq!(os_vars[0], (OsStr::new(var1), Some(OsStr::new(val1))));

    assert_eq!(result.get_program(), OsStr::new("sh"));

    let result_args : Vec<&OsStr> = result.get_args().collect();
    assert_eq!(result_args, vec![OsStr::new(&arg1), OsStr::new(&arg2)]);
    
    Ok(())
}

#[tokio::test]
async fn test_should_error_tty() -> Result<(), anyhow::Error> {

    let symlink_none = None;
    let symlink_some = Some(String::from(".envrc"));

    // terminal is tty
    assert!(!should_error_tty(true, true, &symlink_none), "Skipped tty check should not provoke failsafe");
    assert!(!should_error_tty(true, true, &symlink_some), "Skipped tty check should not provoke failsafe");
    assert!(should_error_tty(true, false, &symlink_none), "tty terminal without symlink should provoke failsafe");
    assert!(!should_error_tty(true, false, &symlink_some), "tty terminal with symlink should not provoke failsafe");

    // terminal is NOT tty
    assert!(!should_error_tty(false, true, &symlink_none), "Non-tty terminal should not cause failsafe");
    assert!(!should_error_tty(false, true, &symlink_some), "Non-tty terminal should not cause failsafe");
    assert!(!should_error_tty(false, false, &symlink_none), "Non-tty terminal should not cause failsafe");
    assert!(!should_error_tty(false, false, &symlink_some), "Non-tty terminal should not cause failsafe");

    Ok(())
}

#[tokio::test]
async fn test_default_loaded_vars() -> Result<(), anyhow::Error> {

    let result = load_env_dryrun_for("empty", "dev").await?;

    assert_eq!(result.variables.get("NOVOPS_ENVIRONMENT").unwrap().value, "dev");

    Ok(())
}

#[tokio::test]
async fn test_list_environments() -> Result<(), anyhow::Error> {
    test_setup().await?;

    let result = list_environments(Some(String::from("tests/.novops.multi-env.yml"))).await?;

    assert_eq!(result.len(), 4);
    assert_eq!(result[0], "dev");
    assert_eq!(result[1], "preprod");
    assert_eq!(result[2], "prod");
    assert_eq!(result[3], "staging");
    Ok(())
}

#[tokio::test]
async fn test_list_environment_output() -> Result<(), anyhow::Error> {
    test_setup().await?;

    let result = list_outputs_for_environment(Some(String::from("tests/.novops.multi-env.yml")), Some("dev".to_string())).await?;

    // Assert this
    assert_eq!(result.variables.len(), 3);
    assert_eq!(result.variables.get("MY_APP_HOST").unwrap().value, "localhost");

    assert_eq!(result.files.len(), 1);

    Ok(())
}

#[tokio::test]
async fn check_working_dir_permissions_test() -> Result<(), anyhow::Error> {

    fn make_tmp_dir(mode: u32) -> PathBuf {
        let dir = tempfile::tempdir().unwrap().into_path();
        let perm = Permissions::from_mode(mode);
        fs::set_permissions(&dir, perm).unwrap();
        dir
    }

    let dir_user = make_tmp_dir(0o700);
    let dir_group = make_tmp_dir(0o760);
    let dir_world = make_tmp_dir(0o706);
    
    let result_user = check_working_dir_permissions(&dir_user);
    assert!(result_user.is_ok(), "Directory with user-only permission should pass check, got {:?}", result_user);

    let result_group = check_working_dir_permissions(&dir_group);
    assert!(result_group.is_err(), "Directory with group permission should not pass check, got {:?}", result_group);

    let result_world = check_working_dir_permissions(&dir_world);
    assert!(result_world.is_err(), "Directory with world permission should not pass check, got {:?}", result_world);

    Ok(())
}

#[tokio::test]
async fn test_get_config_file_path() -> Result<(), anyhow::Error> {

    let test_config = "tests/.novops.plain-strings.yml";    
    
    // Empty dir, should fail
    let dir = tempfile::tempdir().unwrap().into_path();

    let result_no_config = get_config_file_path(&dir, &None);
    assert!(result_no_config.is_err(), "Should fail if no config available.");

    // .yml should be loaded
    let expect_conf_yml = dir.join(".novops.yml");
    fs::copy(test_config, &expect_conf_yml)?;
    let result_yml = get_config_file_path(&dir, &None);
    assert_eq!(result_yml?, expect_conf_yml);

    // .yaml should be loaded with precedence
    let expect_conf_yaml = dir.join(".novops.yaml");
    fs::copy(test_config, &expect_conf_yaml)?;
    let result_yaml = get_config_file_path(&dir, &None);
    assert_eq!(result_yaml?, expect_conf_yaml);
    
    // custom config should be loaded with precedence
    let result_custom_path = get_config_file_path(&dir, &Some(String::from(test_config)));
    assert_eq!(result_custom_path?, PathBuf::from(test_config));

    Ok(())
}