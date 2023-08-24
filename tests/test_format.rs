
#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use novops::modules::variables::VariableOutput;
    use novops::{sanitize_variable_outputs, build_source_file_content};
    use crate::test_utils::test_setup;

    
    #[tokio::test]
    async fn test_prepare_variable_output() -> Result<(), anyhow::Error>{
        test_setup().await?;

        let val1 = "VALUE1";
        let var1 = VariableOutput{
            name: String::from("VAR1"),
            value: String::from(val1)
        };

        // Special characters should be escaped
        let val2=r#"special_char_'"'"'!?`$abc_#~%*µ€{}[]-°+@à^ç=\"#;
        let var2 = VariableOutput{
            name: String::from("VAR2"),
            value: String::from(val2)
        };

        let vars = Vec::from([var1, var2]);

        let result = sanitize_variable_outputs(&vars);

        assert_eq!(result[0].value, val1);
        assert_eq!(result[1].value, "special_char_'\"'\"'\"'\"'\"'\"'\"'\"'!?`$abc_#~%*µ€{}[]-°+@à^ç=\\");

        Ok(())        
    }

    #[tokio::test]
    async fn test_format_variable_outputs() -> Result<(), anyhow::Error> {

        let var1 = VariableOutput{
            name: String::from("VAR1"),
            value: String::from("VALUE1")
        };

        let var2 = VariableOutput{
            name: String::from("VAR2"),
            value: String::from("VALUE2")
        };

        let vars = Vec::from([var1, var2]);

        // dotenv
        let result_dotenv = build_source_file_content("dotenv", &vars)?;
        assert_eq!(result_dotenv, "VAR1='VALUE1'\nVAR2='VALUE2'\n\nunload () {\n  unset -f unload\n  unset VAR1\n  unset VAR2\n}\n\n");

        // dotenv-export
        let result_dotenv_export = build_source_file_content("dotenv-export", &vars)?;
        assert_eq!(result_dotenv_export, "export VAR1='VALUE1'\nexport VAR2='VALUE2'\n\nunload () {\n  unset -f unload\n  unset VAR1\n  unset VAR2\n}\n\n");

        // unknown format expect error
        let result_unknown = build_source_file_content("unknown-zzzz", &vars);
        assert!(result_unknown.is_err());

        Ok(())
    }

}