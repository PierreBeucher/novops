use std::path::PathBuf;
use std::fs;
use std::env;

pub const TEST_DIR: &str = "tests/output";

/**
 * Create a temporary dir to be used for test
 * Mostly used as Novops workdir named after test
 */
#[cfg(test)]
#[allow(dead_code)]
pub fn clean_and_setup_test_dir(test_name: &str) -> Result<PathBuf, anyhow::Error> {
  let test_output_dir = env::current_dir()?.join(TEST_DIR).join(test_name);
  
  if test_output_dir.exists(){
      fs::remove_dir_all(&test_output_dir)?;
  }
  
  fs::create_dir_all(&test_output_dir)?;
  return Ok(test_output_dir)
}