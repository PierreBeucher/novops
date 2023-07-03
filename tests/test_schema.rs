
#[cfg(test)]
mod test_utils;

#[cfg(test)]
mod tests {
    use novops::get_config_schema;
    use crate::test_utils::test_setup;
    use std::fs;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_load_simple_config() -> Result<(), anyhow::Error>{
        test_setup().await?;

        let schema_path = "docs/schema/config-schema.json";
        let schema = get_config_schema()?;
        let expected = fs::read_to_string(schema_path)?.trim().to_string();

        assert_eq!(schema, expected, "Generated schema and {} do not match. Did you run 'make doc' and commit changes?", schema_path);

        Ok(())
    }

}