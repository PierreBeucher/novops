mod test_lib;

use novops::modules::variables::VariableOutput;
use novops::{wrap_var_output_in_quotes, format_variable_outputs};
use test_lib::test_setup;


#[tokio::test]
async fn test_prepare_variable_output() -> Result<(), anyhow::Error>{
    test_setup().await?;

    let val1 = "VALUE1";
    let var1single = VariableOutput{
        name: String::from("VAR1"),
        value: String::from(val1),
        quote_method: None
    };

    let var1double = VariableOutput{
        name: String::from("VAR1"),
        value: String::from(val1),
        quote_method: Some(String::from("double"))
    };

    // Special characters should be escaped
    let val2=r#"special_char_'"'"'!?`$abc_#~%*µ€{}[]-°+@à^ç=\"#;
    let var2single = VariableOutput{
        name: String::from("VAR2"),
        value: String::from(val2),
        quote_method: Some(String::from("single"))
    };

    let var2double = VariableOutput{
        name: String::from("VAR2"),
        value: String::from(val2),
        quote_method: Some(String::from("double"))
    };

    let result1single = wrap_var_output_in_quotes(&var1single)?;
    let result2single = wrap_var_output_in_quotes(&var2single)?;
    let result1double = wrap_var_output_in_quotes(&var1double)?;
    let result2double = wrap_var_output_in_quotes(&var2double)?;

    assert_eq!(result1single, "'VALUE1'");
    assert_eq!(result2single, "'special_char_'\"'\"'\"'\"'\"'\"'\"'\"'!?`$abc_#~%*µ€{}[]-°+@à^ç=\\'");
    assert_eq!(result1double, "\"VALUE1\"");
    assert_eq!(result2double, "\"special_char_'\"'\"'\"'\"'\"'\"'!?`$abc_#~%*µ€{}[]-°+@à^ç=\\\"");


    Ok(())        
}

#[tokio::test]
async fn test_format_variable_outputs() -> Result<(), anyhow::Error> {

    let var1 = VariableOutput{
        name: String::from("VAR1"),
        value: String::from("VALUE1"),
        quote_method: None
    };

    let var2 = VariableOutput{
        name: String::from("VAR2"),
        value: String::from("VALUE2"),
        quote_method: None
    };

    let vars = Vec::from([var1, var2]);

    // dotenv
    let result_dotenv = format_variable_outputs("dotenv", &vars)?;
    assert_eq!(result_dotenv, "VAR1='VALUE1'\nVAR2='VALUE2'\n");

    // dotenv-export
    let result_dotenv_export = format_variable_outputs("dotenv-export", &vars)?;
    assert_eq!(result_dotenv_export, "export VAR1='VALUE1'\nexport VAR2='VALUE2'\n");

    // unknown format expect error
    let result_unknown = format_variable_outputs("unknown-zzzz", &vars);
    assert!(result_unknown.is_err());

    Ok(())
}