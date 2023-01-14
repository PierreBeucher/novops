use std::path::PathBuf;
use std::fs;
use std::env;
use novops::{NovopsOutputs, NovopsArgs, load_context_and_resolve};
use log::debug;

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

/**
 * Load Novops environment for tests/.novops.<conf_name>.yml
 */
#[cfg(test)]
#[allow(dead_code)]
pub async fn load_env_for(conf_name: &str, env: &str) -> Result<NovopsOutputs, anyhow::Error> {
  _load_env_for(conf_name, env, false).await
}

#[cfg(test)]
#[allow(dead_code)]
pub async fn load_env_dryrun_for(conf_name: &str, env: &str) -> Result<NovopsOutputs, anyhow::Error> {
  _load_env_for(conf_name, env, true).await
}

async fn _load_env_for(conf_name: &str, env: &str, dry_run: bool) -> Result<NovopsOutputs, anyhow::Error> {
  let args = NovopsArgs { 
    config: format!("tests/.novops.{}.yml", conf_name), 
    env: Some(env.to_string()), 
    working_directory: None,
    symlink: None,
    dry_run: Some(dry_run)
  };

  let outputs = load_context_and_resolve(&args).await?;

  return Ok(outputs);
}

/**
 * Perform test setup before running tests
 */
pub fn test_setup(){

  // enable logger
  match env_logger::try_init() {
    Ok(_) => {},
    Err(e) => {debug!("env_logger::try_init() error: {:?}", e)},
  };
}
