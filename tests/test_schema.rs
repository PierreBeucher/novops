
mod test_lib;

use novops::get_config_schema;
use test_lib::test_setup;
use std::fs;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_generated_schema() -> Result<(), anyhow::Error>{
    test_setup().await?;

    let schema_path = "docs/schema/config-schema.json";
    let schema = get_config_schema()?;
    let expected = fs::read_to_string(schema_path)?.trim().to_string();

    assert_eq!(schema, expected, "Generated schema and {} do not match. Did you run 'make doc' and commit changes?", schema_path);

    Ok(())
}

