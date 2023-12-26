mod test_utils;


#[cfg(test)]
mod tests {
    use crate::test_utils;

    #[tokio::test]
    async fn test_sops_value() -> Result<(), anyhow::Error> {

        test_utils::test_setup().await?;

        let expected_value =  "nestedValue";
        let expected_file_content = "nested:\n    data:\n        nestedKey: nestedValue\nanother_value: foo\n";

        let outputs = test_utils::load_env_for("sops", "dev").await?;
        
        assert_eq!(outputs.variables.get("SOPS_VALUE").unwrap().value, expected_value);
        assert_eq!(outputs.files.get("/tmp/SOPS_FILE").unwrap().content.clone(), expected_file_content.as_bytes());

        Ok(())
    }

    #[tokio::test]
    async fn test_sops_dotenv() -> Result<(), anyhow::Error> {

        test_utils::test_setup().await?;

        let outputs = test_utils::load_env_for("sops", "integ").await?;
        
        assert_eq!(outputs.variables.get("APP_TOKEN").unwrap().value, "s3cret!");
        assert_eq!(outputs.variables.get("app_host").unwrap().value, "http://localhost");
        assert_eq!(outputs.variables.get("WITH_LINES").unwrap().value, "foo\\nbar\\nbaz\\\\n\\nzzz\\n");
        assert_eq!(outputs.variables.get("WITH_EQUAL").unwrap().value, "EQUAL_CHAR=EQUAL_VALUE");
        assert_eq!(outputs.variables.get("nestedKey").unwrap().value, "nestedValue");

        Ok(())
    }
}