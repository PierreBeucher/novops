
#[cfg(test)]
mod tests {
    use novops::{make_context, NovopsArgs};
    use novops::core::{NovopsContext, NovopsConfig, NovopsConfigDefault, NovopsEnvironmentInput};
    use std::collections::HashMap;
    use std::path::PathBuf;

    /**
     * Test a config is probably loaded into a NovopsContext
     */
    #[tokio::test]
    async fn test_load_simple_config() -> Result<(), anyhow::Error>{

        let standalone_config = "tests/.novops.empty.yml";
        let args = NovopsArgs {
            config: String::from(standalone_config),
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
                workdir: String::from("/tmp/novops"),
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

}