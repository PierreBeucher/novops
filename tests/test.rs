
#[cfg(test)]
mod tests {
    use novops::{make_context, NovopsArgs, run};
    use novops::core::{NovopsContext, NovopsConfig, NovopsConfigDefault, NovopsEnvironmentInput};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::fs;

    const CONFIG_EMPTY: &str = "tests/.novops.empty.yml";
    const CONFIG_STANDALONE: &str = "tests/.novops.standalone.yml";

    /**
     * Test a config is probably loaded into a NovopsContext
     */
    #[tokio::test]
    async fn test_load_simple_config() -> Result<(), anyhow::Error>{

        let args = NovopsArgs {
            config: String::from(CONFIG_EMPTY),
            env: Some(String::from("dev")),
            working_directory: Some(String::from("/tmp/novops")),
            symlink: None
        };
        let result = make_context(&args).await?;

        println!("Result: {:?}", result);

        assert_eq!(result, 
            NovopsContext {
                env_name: String::from("dev"),
                app_name: String::from("test-empty"),
                workdir: String::from("tests/output/test_load_simple_config"),
                config: NovopsConfig{
                    name: String::from("test-empty"),
                    environments: HashMap::from([
                        (String::from("dev"), NovopsEnvironmentInput {
                            variables: vec![],
                            files: vec![],
                            aws: None
                        })
                    ]),
                    default: Some(NovopsConfigDefault {
                        environment: Some(String::from("dev"))
                    })
                },
                env_var_filepath: PathBuf::from(r"/tmp/novops/vars")
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
        let workdir = String::from("tests/output/test_simple_run");
        run(NovopsArgs { 
            config: String::from(CONFIG_STANDALONE),
            env: Some(String::from("dev")), 
            working_directory: Some(workdir.clone()), 
            symlink: None 
        }).await?;   

        let expected_var_file = PathBuf::from(&workdir).join("vars");
        let expected_var_content = fs::read_to_string(expected_var_file)?;
        
        let expected_file_dog_path = PathBuf::from(&workdir).join("file_dog");
        let expected_file_dog_content = fs::read_to_string(&expected_file_dog_path)?;

        let expected_file_cat_path = PathBuf::from("/tmp/novops_cat");
        let expected_file_cat_content = fs::read_to_string(&expected_file_cat_path)?;

        // use r#"_"# for raw string literal
        // check if our file content contains expected export
        // naïve but sufficient for our needs
        assert!(&expected_var_content.contains(r#"export SPECIAL_CHARACTERS='special_char_'"'"'!?`$abc_#~%*µ€{}[]-°+@à^ç=\'"#));
        assert!(&expected_var_content.contains( r"export MY_APP_HOST='localhost'"));
        assert!(&expected_var_content.contains( r"export NOVOPS_TEST_STANDALONE_FILE_DOG='tests/output/test_simple_run/file_dog'"));
        assert!(&expected_var_content.contains( r"export NOVOPS_CAT_VAR='/tmp/novops_cat'"));
        
        assert_eq!(expected_file_dog_content, "woof");
        assert_eq!(expected_file_cat_content, "meow");
        
        Ok(())  
        
    }
    

}