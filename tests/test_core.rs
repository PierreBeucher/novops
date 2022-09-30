
#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use novops::{make_context, NovopsArgs, load_environment};
    use novops::core::{NovopsContext, NovopsConfig, NovopsConfigFile, NovopsConfigDefault, NovopsEnvironmentInput};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::fs;
    use crate::test_utils::clean_and_setup_test_dir;

    const CONFIG_EMPTY: &str = "tests/.novops.empty.yml";
    const CONFIG_STANDALONE: &str = "tests/.novops.standalone.yml";

    /**
     * Test a config is properly loaded into a NovopsContext
     */
    #[tokio::test]
    async fn test_load_simple_config() -> Result<(), anyhow::Error>{

        let workdir = clean_and_setup_test_dir("test_load_simple_config")?;

        let args = NovopsArgs {
            config: String::from(CONFIG_EMPTY),
            env: Some(String::from("dev")),
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()),
            symlink: None
        };
        let result = make_context(&args).await?;

        println!("Result: {:?}", result);

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
                            aws: None
                        })
                    ]),
                    config: Some(NovopsConfig { 
                        default: Some(NovopsConfigDefault {
                             environment: Some(String::from("dev"))
                        }), 
                        hashivault: None 
                    })
                },
                env_var_filepath: workdir.join("vars")
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
        let workdir = clean_and_setup_test_dir("test_simple_run")?;

        load_environment(NovopsArgs { 
            config: String::from(CONFIG_STANDALONE),
            env: Some(String::from("dev")), 
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()), 
            symlink: None 
        }).await?;   

        let expected_var_file = PathBuf::from(&workdir).join("vars");
        let expected_var_content = fs::read_to_string(expected_var_file)?;
        
        let expected_file_dog_path = PathBuf::from(&workdir).join("file_dog");
        let expected_file_dog_content = fs::read_to_string(&expected_file_dog_path)?;

        let expected_file_cat_path = PathBuf::from("/tmp/novops_cat");
        let expected_file_cat_content = fs::read_to_string(&expected_file_cat_path)?;

        // Expect to match content of CONFIG_STANDALONE
        // use r#"_"# for raw string literal
        // check if our file content contains expected export
        // naïve but sufficient for our needs
        assert!(&expected_var_content.contains(r#"export SPECIAL_CHARACTERS='special_char_'"'"'!?`$abc_#~%*µ€{}[]-°+@à^ç=\'"#));
        assert!(&expected_var_content.contains( "export MY_APP_HOST='localhost'"));
        assert!(&expected_var_content.contains( &format!("export NOVOPS_TEST_STANDALONE_FILE_DOG='{:}'",
            expected_file_dog_path.into_os_string().into_string().unwrap())));
        assert!(&expected_var_content.contains( "export NOVOPS_CAT_VAR='/tmp/novops_cat'"));
        
        assert_eq!(expected_file_dog_content, "woof");
        assert_eq!(expected_file_cat_content, "meow");
        
        Ok(())  
        
    }
    
    #[tokio::test]
    async fn test_symlink_flag() -> Result<(), anyhow::Error> {
        let workdir = clean_and_setup_test_dir("test_symlink_flag")?;

        let expect_symlink_at = PathBuf::from(&workdir).join("test-symlink");
        load_environment(NovopsArgs { 
            config: String::from(CONFIG_EMPTY),
            env: Some(String::from("dev")), 
            working_directory: Some(workdir.clone().into_os_string().into_string().unwrap()), 
            symlink: Some(expect_symlink_at.clone().into_os_string().into_string().unwrap())
        }).await?;

        let symlink_metadata = fs::symlink_metadata(&expect_symlink_at)?;
        assert!(symlink_metadata.is_symlink(), "{:?} does not seem to be a symlink: {:?}", &expect_symlink_at, symlink_metadata);

        // symlink is expected to point to var file under our working directory
        let symlink_dest = fs::read_link(expect_symlink_at).unwrap();
        assert_eq!(symlink_dest, PathBuf::from(&workdir).join("vars"));

        Ok(())
    }

}