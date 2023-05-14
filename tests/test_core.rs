
#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use novops::{make_context, NovopsArgs, load_environment};
    use novops::core::{NovopsContext, NovopsConfig, NovopsConfigFile, NovopsConfigDefault, NovopsEnvironmentInput};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use log::info;
    use crate::test_utils::{clean_and_setup_test_dir, TEST_DIR, load_env_dryrun_for, test_setup};

    const CONFIG_EMPTY: &str = "tests/.novops.empty.yml";
    const CONFIG_STANDALONE: &str = "tests/.novops.plain-strings.yml";

    /**
     * Test a config is properly loaded into a NovopsContext
     */
    #[tokio::test]
    async fn test_load_simple_config() -> Result<(), anyhow::Error>{
        test_setup();

        let workdir = clean_and_setup_test_dir("test_load_simple_config")?;

        let args = NovopsArgs {
            config: String::from(CONFIG_EMPTY),
            env: Some(String::from("dev")),
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()),
            symlink: None,
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
                    name: String::from("test-empty"),
                    environments: HashMap::from([
                        (String::from("dev"), NovopsEnvironmentInput {
                            variables: None,
                            files: None,
                            aws: None,
                            hashivault: None,
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
        test_setup();

        let workdir = clean_and_setup_test_dir("test_simple_run")?;

        load_environment(NovopsArgs { 
            config: String::from(CONFIG_STANDALONE),
            env: Some(String::from("dev")), 
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()), 
            symlink: None,
            dry_run: None
        }).await?;   

        let expected_var_file = PathBuf::from(&workdir).join("vars");
        let expected_var_content = fs::read_to_string(expected_var_file)?;
        
        let expected_file_dog_path = PathBuf::from(&workdir).join("file_1811bdd29f2cfe95e6e23402e2390fa1012708fc52ef8b8a29ee540b1c481534");
        let expected_file_dog_content = fs::read_to_string(&expected_file_dog_path)?;
        let file_dog_metadata = fs::metadata(&expected_file_dog_path)?;
        let file_dog_mode = file_dog_metadata.permissions().mode();

        let expected_file_cat_path = PathBuf::from("/tmp/novops_cat");
        let expected_file_cat_content = fs::read_to_string(&expected_file_cat_path)?;

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
        test_setup();

        let workdir = clean_and_setup_test_dir("test_symlink_flag")?;

        let expect_symlink_at = PathBuf::from(TEST_DIR).join("test-symlink");
        load_environment(NovopsArgs { 
            config: String::from(CONFIG_STANDALONE),
            env: Some(String::from("dev")), 
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()), 
            symlink: Some(expect_symlink_at.clone().into_os_string().into_string().unwrap()),
            dry_run: None
        }).await?;

        let symlink_metadata = fs::symlink_metadata(&expect_symlink_at)?;
        assert!(symlink_metadata.is_symlink(), "{:?} does not seem to be a symlink: {:?}", &expect_symlink_at, symlink_metadata);

        // symlink is expected to point to var file under our working directory
        let symlink_dest = fs::read_link(&expect_symlink_at).unwrap();
        assert_eq!(symlink_dest, PathBuf::from(&workdir).join("vars"), "Symlink destination is not as expected");

        // run again with different symlink dest
        // expect existing symlink to be overriden
        let workdir_override = clean_and_setup_test_dir("test_symlink_flag_override")?;
        load_environment(NovopsArgs { 
            config: String::from(CONFIG_STANDALONE),
            env: Some(String::from("staging")), 
            working_directory: Some(workdir_override.clone().into_os_string().into_string().unwrap()), 
            symlink: Some(expect_symlink_at.clone().into_os_string().into_string().unwrap()),
            dry_run: None
        }).await?;

        let overriden_symlink_dest = fs::read_link(&expect_symlink_at).unwrap();
        assert_eq!(overriden_symlink_dest, PathBuf::from(&workdir_override).join("vars"), "Symlink destination is not as expected");

        Ok(())
    }

    /**
     * Ensure that a file/dir at symlink path result in failure
     */
    #[tokio::test]
    async fn test_symlink_no_file_override() -> Result<(), anyhow::Error> {
        test_setup();

        let workdir = clean_and_setup_test_dir("test_symlink_no_file_override")?;

        // create dummy file, we don't want it erased by symlink
        let symlink_path = PathBuf::from(&workdir).join("file-dont-override");
        fs::File::create(&symlink_path)?;
        
        // expect error as we cannot erase existing file
        let result = load_environment(NovopsArgs { 
            config: String::from(CONFIG_STANDALONE),
            env: Some(String::from("dev")), 
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()), 
            symlink: Some(symlink_path.clone().into_os_string().into_string().unwrap()),
            dry_run: None
        }).await;

        result.expect_err("Expected an error when loading with symlink trying to override existing file, got OK.");

        Ok(())
    }

    /**
     * Check all modules with dry run
     * Having non-empty values and no errors is enough
     */
    #[tokio::test]
    async fn test_dry_run() -> Result<(), anyhow::Error> {
        test_setup();
        
        let result = load_env_dryrun_for("all-modules", "dev").await?;

        info!("test_dry_run: Found variables: {:?}", &result.variables);
        info!("test_dry_run: Found files: {:?}", &result.files);


        assert!(result.variables.get("VAR").unwrap().value.len() > 0);
        assert!(result.variables.get("AWS_SECRETMANAGER").unwrap().value.len() > 0);
        assert!(result.variables.get("AWS_SSM_PARAMETER").unwrap().value.len() > 0);
        assert!(result.variables.get("HASHIVAULT_KV_V2").unwrap().value.len() > 0);
        assert!(result.variables.get("BITWARDEN").unwrap().value.len() > 0);
        assert!(result.variables.get("GCLOUD_SECRETMANAGER").unwrap().value.len() > 0);
        assert!(result.files.get("/tmp/novopsfile").unwrap().content.len() > 0);
    
        // aws.assumerole
        assert!(result.variables.get("AWS_ACCESS_KEY_ID").unwrap().value.len() > 0);
        assert!(result.variables.get("AWS_SESSION_TOKEN").unwrap().value.len() > 0);
        assert!(result.variables.get("AWS_SECRET_ACCESS_KEY").unwrap().value.len() > 0);

        Ok(())
    }

}