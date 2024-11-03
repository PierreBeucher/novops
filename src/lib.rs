pub mod core;
pub mod modules;
pub mod resolve;

use crate::core::{NovopsEnvironmentInput, NovopsConfigFile, NovopsContext};
use crate::modules::files::FileOutput;
use crate::modules::variables::VariableOutput;
use crate::resolve::resolve_environment_inputs_parallel;
use log::{info, debug, error, warn};
use std::os::unix::prelude::{OpenOptionsExt, PermissionsExt};
use std::os::unix::fs::{MetadataExt, symlink};
use std::{fs::{self, symlink_metadata, remove_file}, io::prelude::*};

use anyhow::Context;
use std::os::unix;
use std::io::IsTerminal;
use std::path::{PathBuf, Path};
use std::env;
use std::collections::HashMap;
use schemars::schema_for;
use std::process::Command;
use std::os::unix::process::CommandExt;

use console::Term;

#[derive(Debug)]
pub struct NovopsLoadArgs {
    pub config: Option<String>,

    pub env: Option<String>,

    pub working_directory: Option<String>,

    pub skip_working_directory_check: Option<bool>,

    pub dry_run: Option<bool>

}

const FORMAT_DOTENV_EXPORT: &str = "dotenv-export";
const FORMAT_DOTENV_PLAIN: &str = "dotenv";
const FORMAT_ALL: [&str; 2] = [ FORMAT_DOTENV_EXPORT, FORMAT_DOTENV_PLAIN ];

/**
 * Structure containing all Outputs after resolving
 */
#[derive(Debug, Clone)]
pub struct NovopsOutputs {
    pub context: NovopsContext,
    pub variables: HashMap<String, VariableOutput>,
    pub files: HashMap<String, FileOutput>
}


/// Use by `novops load` to load environment and write variables to stdout or file
/// Load environment and write variables to stdout or file
/// Checks if stdout is tty for safety to avoid showing secrets on screen
pub async fn load_environment_write_vars(args: &NovopsLoadArgs, symlink: &Option<String>, format: &str, skip_tty_check: bool) -> Result<(), anyhow::Error> {

    // safety checks
    check_stdout_tty_and_exit(skip_tty_check, symlink);

    let outputs = load_context_and_resolve(args).await?;

    // Write files to filesystem
    export_file_outputs(&outputs.files.clone().into_values().collect())?;

    // Write env file to filesystem
    let vars: Vec<VariableOutput> = outputs.variables.clone().into_values().collect();
    export_variable_outputs(format, symlink, &vars, &outputs.context.workdir)?;

    info!("Novops environment loaded !");

    Ok(())
}

/// Used by `novops run` to load environment and run child process
pub async fn load_environment_and_exec(args: &NovopsLoadArgs, command_args: Vec<&String>) -> Result<(), anyhow::Error> {

    let outputs = load_context_and_resolve(args).await?;

    // Write files to filesystem
    export_file_outputs(&outputs.files.clone().into_values().collect())?;

    // Run child process with variables
    let vars : Vec<VariableOutput> = outputs.variables.clone().into_values().collect();
    let mut cmd = prepare_exec_command(command_args, &vars);
    exec_replace(&mut cmd)
        .with_context(|| format!("Error running process {:?} {:?}", &cmd.get_program(), &cmd.get_args()))?;

    Ok(())
}

/// Load an environment without side effect and return outputs
pub async fn load_context_and_resolve(args: &NovopsLoadArgs) -> Result<NovopsOutputs, anyhow::Error> {
    init_logger();

    debug!("Loading context for {:?}", &args);

    let ctx = make_context(args).await?;
    let novops_env = get_current_environment(&ctx).await?;

    let (raw_var_outputs, raw_file_outputs) = resolve_environment_inputs_parallel(&ctx, novops_env).await?;

    // Transform raw output into their corresponding HashMaps
    let mut var_outputs: HashMap<String, VariableOutput> = HashMap::new();
    let mut file_outputs: HashMap<String, FileOutput> = HashMap::new();

    // Expose Novops internal variables
    // Set first so user can override via config if needed
    var_outputs.insert(String::from("NOVOPS_ENVIRONMENT"), VariableOutput {
        name: String::from("NOVOPS_ENVIRONMENT"),
        value: ctx.env_name.clone()
    });

    for v in raw_var_outputs.iter() { var_outputs.insert(v.name.clone(), v.clone()); }
    for f in raw_file_outputs {
        
        let fpath_str = f.dest.to_str()
            .ok_or(anyhow::anyhow!("Couldn't convert PathBuf '{:?}' to String", &f.dest))?;

        // FileInput generates both var and file output
        var_outputs.insert(fpath_str.to_string(), f.variable.clone());
        file_outputs.insert(fpath_str.to_string(), f.clone());
    };

    Ok(NovopsOutputs { 
        context: ctx, 
        variables: var_outputs, 
        files: file_outputs 
    })
}

pub fn prepare_exec_command(mut command_args: Vec<&String>, variables: &Vec<VariableOutput>) -> Command{
    // first element of command argument is passed as child program
    // everything else is passed as arguments
    let child_program = command_args.remove(0);
    let child_args = command_args;

    let mut command = Command::new(child_program);
    command.args(&child_args);
    

    for var in variables {
        command.env(&var.name, &var.value);
    }

    command
}

/**
 * Run child process replacing current process
 * Never returns () as child process should have replaced current process
 */
fn exec_replace(cmd: &mut Command) -> Result<(), anyhow::Error>{

    info!("Running child command: {:?} {:?}", &cmd.get_program(), &cmd.get_args());
    let error = cmd.exec();
    
    Err(anyhow::Error::new(error))
}

/**
 * Initialize logger. Ca be called more than once. 
 * Novops always logs to stderr as stdout is reserved from output environment variables.
 */
pub fn init_logger() {
    let log_init_result = env_logger::Builder::new()
        .parse_default_env()
        .target(env_logger::Target::Stderr)
        .try_init();

    // Allow multiple invocation of logger
    match log_init_result {
        Ok(_) => {},
        Err(e) => {debug!("env_logger::try_init() error: {:?} - logger was probably already initialized, no worries.", e)},
    }
}

/**
 * List all environments from config file
 */
pub async fn list_environments(config_file: Option<String>) -> Result<Vec<String>, anyhow::Error> {
    init_logger();

    debug!("Listing environments from {:?}", &config_file);

    let config = read_config_file(&config_file)
        .with_context(|| "Error reading Novops config file")?;

    let envs = list_environments_from_config(&config);
    Ok(envs)
}

/**
 * List all outputs for an environment from config file
 * Use dry-run mode to generate outputs
 */
pub async fn list_outputs_for_environment(config_file: Option<String>, env_name: Option<String>) -> Result<NovopsOutputs, anyhow::Error> {
    init_logger();

    debug!("Listing outputs for environment {:?} from {:?}", &env_name, &config_file);
    
    let dryrun_args = NovopsLoadArgs{ 
        config: config_file,
        env: env_name,
        working_directory: None,
        skip_working_directory_check: Some(false),
        dry_run: Some(true)
    };

    let outputs = load_context_and_resolve(&dryrun_args).await?;
    Ok(outputs)
}
/**
 * Generate Novops context from arguments, env vars and Novops config
 */
pub async fn make_context(args: &NovopsLoadArgs) -> Result<NovopsContext, anyhow::Error> {
    // Read CLI args and load config
    let config = read_config_file(&args.config)
        .with_context(|| "Error reading config file.")?;

    // app name may be specififed by user
    // if not, use current directory name
    let app_name = match &config.name {
        Some(n) => n.clone(),
        None => {
            let curdir = env::current_dir()
                .with_context(|| "Couldn't read current directory.")?;
            
                let default_app_name = curdir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("root");
            
            String::from(default_app_name)
            // may not yield exact folder name as some non-UTF8 char may be replaced
            // but it's acceptable for our purpose        
        }
    };

    // Env name is either specified as arg, as default or prompt user
    let env_name = read_environment_name(&config, &args.env)?;

    // working directory under which to save files (if any)
    let workdir = prepare_working_directory(args, &app_name, &env_name)?;
    info!("Using workdir: {:?}", &workdir);

    // environment variable file which will contain variable output the user can export
    let env_var_filepath = workdir.join("vars");

    let ctx = NovopsContext {
        env_name: env_name.clone(),
        app_name: app_name.clone(),
        workdir: workdir.clone(),
        config_file_data: config.clone(),
        env_var_filepath,
        dry_run: args.dry_run.unwrap_or(false)
    };

    debug!("Prepared context: {:?}", &ctx);

    Ok(ctx)

}

/**
 * Read config and load all Input types into a NovopsInputs struct
 */
pub async fn get_current_environment(ctx: &NovopsContext) -> Result<NovopsEnvironmentInput, anyhow::Error> {    
    let novops_env = ctx.config_file_data.environments.get(&ctx.env_name)
        .with_context(|| format!("Environment '{}' not found in config.", &ctx.env_name))?;
    
    Ok(novops_env.clone())
}

/**
 * Read Novops configuration file. Use provided config file if Option is Some, default to .novops.y[a]ml in current directory. 
 */
fn read_config_file(config_path_opt: &Option<String>) -> Result<NovopsConfigFile, anyhow::Error> {
    
    let current_dir = env::current_dir()?;
    let cfg_path = get_config_file_path(&current_dir, config_path_opt)?;

    debug!("Loading config file path '{:?}'", &cfg_path);
    
    let f = std::fs::File::open(&cfg_path)
        .with_context(|| format!("Failed to open Novops config {:?}", &cfg_path))?;  
    let config: NovopsConfigFile = serde_yaml::from_reader(f)
        .with_context(|| format!("Error parsing config file {:?}. Does it match expected config schema?", &cfg_path))?;

    Ok(config)
}

/**
 * Return given option value is Some, or found .yml / .yaml config.
 */
pub fn get_config_file_path(current_dir: &Path, config_path_opt: &Option<String>) -> Result<PathBuf, anyhow::Error> {
    let yaml_path = current_dir.join(".novops.yaml");
    let yml_path = current_dir.join(".novops.yml");

    match config_path_opt.clone() {
        Some(e) => Ok(PathBuf::from(e)),
        None => {
            if yaml_path.exists() {
                Ok(yaml_path)
            } else if yml_path.exists() {
                Ok(yml_path)
            } else {
                Err(anyhow::anyhow!("Config file .novops.yml or .novops.yaml not found in current directory. You can set a custom config file path with -c/--config PATH."))
            }
        },
    }
}

/** 
 * Read the environment name with this precedence (higher to lowest):
 * - CLI flag
 * - prompt user (using config's default if no choice given)
 */
fn read_environment_name(config: &NovopsConfigFile, flag: &Option<String>) -> Result<String, anyhow::Error> {

    match flag {
        Some(e) => Ok(e.clone()),
        None => Ok(prompt_for_environment(config)?)
    }
}

/**
 * Retrieve working directory where files and exportable variable file will be written by default
 * with the following precedence:
 * - CLI flag working directory
 * - XDG runtime dir (if available), such as $XDg_RUNTIME_DIR/novops/<app>/<env>
 * - Default to /tmp/novops/<uid>/<app>/<env> (with /tmp/novops/<uid> limited to user)
 * 
 * Returns the absolute path to working directory
 */
fn prepare_working_directory(args: &NovopsLoadArgs, app_name: &String, env_name: &String) -> Result<PathBuf, anyhow::Error> {
    
    let workdir = match &args.working_directory {
        Some(custom_workdir) => {
            
            // If custom working specified, check permissions
            if ! args.skip_working_directory_check.unwrap_or(false) {
                let custom_workdir_path = PathBuf::from(custom_workdir);
                check_working_dir_permissions(&custom_workdir_path)
                    .with_context(|| "Working directory permissions are unsafe. Use --skip-workdir-check to skip this check.")?;
            } else {
                warn!("Skipping working directory permissions check. This is unsafe !")
            }
        
            PathBuf::from(custom_workdir)
        },
        None => match prepare_working_directory_xdg(app_name, env_name) {
                Ok(s) => s,
                Err(e) => {
                    info!("Using /tmp as XDG did not seem available: {:?}", e);
                    prepare_working_directory_tmp(app_name, env_name)?
                },
        }
    };

    // workdir may be passed as relative path or auto-generated as absolute path
    // to avoid confusion always return it as absolute path
    if workdir.is_absolute() {
        Ok(workdir)
    } else {
        Ok(env::current_dir()
        .with_context(|| "Couldn't get process current working directory")?
        .join(&workdir))
    }
}

/** 
 * Prepare a workding directory using xdg
 * Returns an error if XDG is not available or failed somehow
*/
fn prepare_working_directory_xdg(app_name: &String, env_name: &String) -> Result<PathBuf, anyhow::Error> {
    let xdg_prefix = format!("novops/{:}/{:}", app_name, env_name);

    let xdg_basedir = xdg::BaseDirectories::new()?
        .create_runtime_directory(xdg_prefix)?;
    
    Ok(xdg_basedir)
}


/**
 * Use /tmp as base for Novops workdir
 */
fn prepare_working_directory_tmp(app_name: &String, env_name: &String) -> Result<PathBuf, anyhow::Error>{
    let user_workdir = format!("/tmp/novops/{:}", users::get_current_uid());
    let workdir = format!("{:}/{:}/{:}", user_workdir, app_name, env_name);

    // make sure user workdir exists with safe permissions
    // first empty current workdir (if any) for current app/env
    fs::create_dir_all(&user_workdir)
        .with_context(|| format!("Couldn't create user working directory {:?}", &user_workdir))?;
    
    fs::set_permissions(&user_workdir, fs::Permissions::from_mode(0o0700))
        .with_context(|| format!("Couldn't set permission on user working directory {:?}", &user_workdir))?;
    
    // create current app/env workdir under user workdir
    fs::create_dir_all(&workdir)
        .with_context(|| format!("Couldn't create working directory {:?}", &workdir))?;
    
    Ok(PathBuf::from(workdir))
}

/**
 * Check workdir is safe: should not be read/write/executable 
 * by anyone other than current user or root
 */
pub fn check_working_dir_permissions(workdir: &PathBuf) -> Result<(), anyhow::Error> {
    let metadata = fs::metadata(workdir)
        .with_context(|| format!("Couldn't get metadata for working directory {:?}", &workdir))?;

    // check mode is rwx------ (or more restrictive)
    let mode =  metadata.permissions().mode();
    if metadata.permissions().mode() & 0o777 != 0o700 {
        return Err(anyhow::anyhow!("Working directory {:?} mode {:o} (octal) is too large. Only current user should have access.", &workdir, &mode));
    }

    // check owner is current user or root
    let workdir_owner_uid = metadata.uid();
    let current_uid = users::get_current_uid();
    if workdir_owner_uid != current_uid && workdir_owner_uid != 0 {
        return Err(anyhow::anyhow!("Working directory {:?} ownership is unsafe (owned by {:}). Only current user {:} or root can have ownership.", &workdir, &workdir_owner_uid, &current_uid));
    }

    Ok(())
}

/**
 * Prompt user for environment name using dialoguer for nice UI
 */
fn prompt_for_environment(config_file_data: &NovopsConfigFile) -> Result<String, anyhow::Error>{

    let environments = config_file_data.environments.iter()
        .map(|e| e.0.clone())
        .collect::<Vec<String>>();

    let default = config_file_data.config.clone()
        .and_then(|c| c.default)
        .and_then(|d| d.environment);

    // If no environment, no happy
    if environments.is_empty() {
        return Err(anyhow::anyhow!("No environment configured."));
    }

    // If only one environment, no need to prompt
    if environments.len() == 1 {
        debug!("Only one environment configured, using it by default");
        return Ok(environments[0].clone());
    }

    // Look for default environment index in environment list
    // to point to this environment by default.
    // Error if default environment not found in environment list
    let default_index = match default {
        Some(d) => {
            let idx = environments.iter().position(|e | e == &d);
            match idx {
                Some(i) => i,
                None => {
                    return Err(anyhow::anyhow!("Default environment '{:}' not found in config", &d));
                },
            }
        },
        None => 0,
    };

    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select environment")
        .default(default_index)
        .items(&environments)
        .interact_on_opt(&Term::stderr())
        .with_context(|| "Couldn't prompt for environment")?;

    let selected = match selection {
        Some(index) => environments[index].clone(),
        None => return Err(anyhow::anyhow!("No environment selected")),
    };

    Ok(selected)
}

/**
 * Return a sorted list of environments from config
 */
fn list_environments_from_config(config_file_data: &NovopsConfigFile) -> Vec<String> {
    let mut sorted = config_file_data.environments.keys().cloned().collect::<Vec<String>>();
    sorted.sort();
    sorted
}

/**
 * Write resolved files to protected directory
 */
pub fn export_file_outputs(files: &Vec<FileOutput>) -> Result<(), anyhow::Error>{
    
    for f in files {
        let mut fd = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .mode(0o600)
            .open(&f.dest)
            .with_context(|| format!("Can't open {:?} for write with mode 0600", &f.dest))?;

        fd.write_all(&f.content)
            .with_context(|| format!("Can't write to {:?} after opening file descriptor", &f.dest))?;

        if let Some(sl) = &f.symlink {
            let sl_meta = symlink_metadata(sl);

            if sl_meta.is_ok() {
                let m = sl_meta.unwrap();
                if !m.is_symlink() {
                    return Err(anyhow::anyhow!("{:?} already exists and is not a symlink. Won't overwrite a non-symlink file. Did you specify the right symlink dest?", &sl))
                } else {
                    remove_file(sl).with_context(|| format!("Can't remove {:?} prior to creating symlink", &sl))?;
                }
            }

            symlink(&f.dest, sl)
                .with_context(|| format!("Can't create symlink {:?} -> {:?}", &sl, &f.dest))?;
        }
    }

    Ok(())
}

/**
 * Write a sourceable environment variable file
 * With a content like
 * '
 *  export VAR=value
 *  export FOO=bar
 * '  
 * 
 */
fn export_variable_outputs(format: &str, symlink: &Option<String>, vars: &Vec<VariableOutput>, working_dir: &Path) -> Result<(), anyhow::Error>{

    let safe_vars = prepare_variable_outputs(vars);
    let vars_string = format_variable_outputs(format, &safe_vars)?;
    
    // If symlink option provided, create symlink
    // otherwise show variables in stdout
    match symlink {
        Some(lnk_str) => {
            let workdir_var_file = working_dir.join("vars");
            info!("Writing variable file in working directory at {:?}", &workdir_var_file);

            fs::write(&workdir_var_file, vars_string)
                .with_context(|| format!("Error writing file output at {:?}", &workdir_var_file))?;
                    
            let lnk = PathBuf::from(&lnk_str);
            create_symlink(&lnk, &workdir_var_file)
                .with_context(|| format!("Couldn't create symlink {:}", lnk_str))?
        },
        None => {
            println!("{:}", vars_string)
        }
    }

    Ok(())    
}

/**
 * make sure our variables can be exported and return safe VariableOutput values 
 * use single quotes to avoid interpolation
 * if single quote "'" are in password, replace them by ` '"'"' ` 
 * which will cause bash to interpret them properly
 * first ' ends initial quoation, then wrap our quote with "'" and start a new quotation with '
 * for example password ` abc'def ` will become ` export pass='abc'"'"'def' `
 */
pub fn prepare_variable_outputs(vars: &Vec<VariableOutput>) ->  Vec<VariableOutput> {
    let mut safe_vars : Vec<VariableOutput> = Vec::new();
    for v in vars{
        let safe_val = v.value.replace('\'', "'\"'\"'");
        safe_vars.push(VariableOutput { name: v.name.clone(), value: safe_val })
    }

    safe_vars
}

/**
 * Transform VariableOutputs to string according to given format 
 */
pub fn format_variable_outputs(format: &str, safe_vars: &Vec<VariableOutput>) -> Result<String, anyhow::Error> {
    let mut vars_string = String::new();

    if format == FORMAT_DOTENV_EXPORT {
        // build a string such as
        //  '
        //   export VAR="value"
        //   export FOO="bar"
        //  '
        for v in safe_vars{
            let s = format!("export {:}='{:}'\n", &v.name, &v.value);
            vars_string.push_str(&s);
        }
    } else if format == FORMAT_DOTENV_PLAIN {
        // build a string such as
        //  '
        //   VAR="value"
        //   FOO="bar"
        //  '
        for v in safe_vars{
            let s = format!("{:}='{:}'\n", &v.name, &v.value);
            vars_string.push_str(&s);
        }
    } else {
        return Err(anyhow::format_err!("Unknown format {:}. Available formats: {:?}", format, FORMAT_ALL))
    }

    Ok(vars_string)

}

fn create_symlink(lnk: &PathBuf, target: &PathBuf) -> Result<(), anyhow::Error> {
    let attr_result = fs::symlink_metadata(lnk);
    if attr_result.is_ok() {
        let attr = attr_result?;
        if attr.is_symlink() && attr.uid() == users::get_current_uid(){
            debug!("Deleting existing symlink {:?} before creating new one", &lnk);
            fs::remove_file(lnk).with_context(|| format!("Couldn't remove existing symlink {:?}", &lnk))?;
        } else {
            return Err(anyhow::anyhow!("Symlink creation error: {:?} already exists and is not a symlink you own.", &lnk));
        }
    } 
    
    unix::fs::symlink(target, lnk)
        .with_context(|| format!("Couldn't create symlink {:?} -> {:?}", &lnk, &target))
}
    
pub fn get_config_schema() -> Result<String, anyhow::Error>{
    let schema = schema_for!(NovopsConfigFile);
    let output = serde_json::to_string_pretty(&schema)
        .with_context(|| "Couldn't convert JSON schema to pretty string.")?;
    Ok(output)
}

/// See should_error_tty
fn check_stdout_tty_and_exit(skip_tty_check: bool, symlink: &Option<String>) {

    let terminal_is_tty = std::io::stdout().is_terminal();

    if should_error_tty(terminal_is_tty, skip_tty_check, symlink) {
        error!("Stdout is a terminal, secrets won't be loaded to avoid exposing them. \
            Either use 'novops load' with -s/--symlink to write in a secure directory, 'source<(novops load [OPTIONS]) or 'novops run' to load secrets directly in process memory. \
            If you really want to show secrets on terminal, use --skip-tty-check to skip this check.");
        std::process::exit(3)
    }
}

/// Check whether novops whould error and exi to avoid outputting secrets on stoud
/// If skip_tty_check is trued, check is ignored
/// Otherwise, if tty is a terminal and symlink is none, secrets may be outputted directly in terminal
/// Novops failsafe in such case to avoid exposing secret
pub fn should_error_tty(terminal_is_tty: bool, skip_tty_check: bool, symlink: &Option<String>) -> bool {

    // conditions could have been wrote in a single line 
    // but keep explicit structure for readibility
    if skip_tty_check {
        return false
    } 
    
    if terminal_is_tty && symlink.is_none() {
        return true
    }

    false
}