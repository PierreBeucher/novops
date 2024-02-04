pub mod core;
pub mod modules;

use crate::core::{ResolveTo, NovopsEnvironmentInput, NovopsConfigFile, NovopsContext};
use crate::modules::files::FileOutput;
use crate::modules::variables::VariableOutput;
use log::{info, debug, error, warn};

use std::os::linux::fs::MetadataExt;
use std::os::unix::prelude::OpenOptionsExt;
use std::{fs, io::prelude::*, os::unix::prelude::PermissionsExt};

use anyhow::{self, Context};
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
    pub config: String,

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
    export_file_outputs(&outputs)?;

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
    export_file_outputs(&outputs)?;

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
    
    // Revole inputs and export
    let (var_out, file_out) = 
        resolve_environment_inputs(&ctx, novops_env).await?;

    Ok(NovopsOutputs { 
        context: ctx, 
        variables: var_out, 
        files: file_out 
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
pub async fn list_environments(config_file: &str) -> Result<Vec<String>, anyhow::Error> {
    init_logger();

    debug!("Listing environments from {:?}", &config_file);

    let config = read_config_file(config_file)
        .with_context(|| format!("Error reading config file '{:}'", &config_file))?;

    let envs = list_environments_from_config(&config);
    Ok(envs)
}

/**
 * List all outputs for an environment from config file
 * Use dry-run mode to generate outputs
 */
pub async fn list_outputs_for_environment(config_file: &str, env_name: Option<String>) -> Result<NovopsOutputs, anyhow::Error> {
    init_logger();

    debug!("Listing outputs for environment {:?} from {:?}", &env_name, &config_file);
    
    let dryrun_args = NovopsLoadArgs{ 
        config: String::from(config_file),
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
        .with_context(|| format!("Error reading config file '{:}'", &args.config))?;

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
 * Resolve all Inputs to their concrete Output values
 * Depending on Input types, external systems will be called-upon (such as BitWarden, Hashicorp Vault...)
 */
pub async fn resolve_environment_inputs(ctx: &NovopsContext, inputs: NovopsEnvironmentInput) 
        -> Result<(HashMap<String, VariableOutput>, HashMap<String, FileOutput>), anyhow::Error>
    {
    
    let mut variable_outputs: HashMap<String, VariableOutput> = HashMap::from(generate_builtin_variable_outputs(&ctx));
    let mut file_outputs: HashMap<String, FileOutput> = HashMap::new();

    for v in &inputs.variables.unwrap_or_default() {
        let val = v.resolve(ctx).await
            .with_context(|| format!("Couldn't resolve variable input {:?}", v))?;
        
        variable_outputs.insert(v.name.clone(), val);
    };

    for f in &inputs.files.unwrap_or_default() {
        let r = f.resolve(ctx).await
            .with_context(|| format!("Couldn't resolve file input {:?}", f))?;
        
        let fpath_str = r.dest.to_str()
            .ok_or(anyhow::anyhow!("Couldn't convert PathBuf '{:?}' to String", &r.dest))?;

        // FileInput generates both var and file output
        variable_outputs.insert(fpath_str.to_string(), r.variable.clone());
        file_outputs.insert(fpath_str.to_string(), r.clone());
    };

    match &inputs.aws {
        Some(aws) => {
            let r = aws.assume_role.resolve(ctx).await
                .with_context(|| format!("Could not resolve AWS input {:?}", aws))?;

            for vo in r {
                variable_outputs.insert(vo.name.clone(), vo);
            }
            
        },
        None => (),
    }

    match &inputs.hashivault {
        Some(hashivault) => {
            let r = hashivault.aws.resolve(ctx).await
                .with_context(|| format!("Could not resolve Hashivault input {:?}", hashivault))?;

            for vo in r {
                variable_outputs.insert(vo.name.clone(), vo);
            }
            
        },
        None => (),
    }

    match &inputs.sops_dotenv {
        Some(sops_dotenv) => {
            for s in sops_dotenv {
                let r = s.resolve(ctx).await
                    .with_context(|| format!("Could not resolve SopsDotenv input {:?}", s))?;

                for vo in r {
                    variable_outputs.insert(vo.name.clone(), vo);
                }
            }
        },
        None => (),
    }

    Ok((variable_outputs, file_outputs))

}

fn read_config_file(config_path: &str) -> Result<NovopsConfigFile, anyhow::Error> {
    let f = std::fs::File::open(config_path)?;
    let config: NovopsConfigFile = serde_yaml::from_reader(f)
        .with_context(|| "Error parsing config file and Novops config schema. Maybe some config does not match expected schema?")?;

    Ok(config)
}

fn generate_builtin_variable_outputs(ctx: &NovopsContext) -> HashMap<String, VariableOutput> {
    let mut vars : HashMap<String, VariableOutput> = HashMap::new();

    // Expose current Novops environment as variable internal variables
    // Load first so user can override via config if needed
    vars.insert(String::from("NOVOPS_ENVIRONMENT"), VariableOutput {
        name: String::from("NOVOPS_ENVIRONMENT"),
        value: ctx.env_name.clone(),
        quote_method: None,
    });

    vars.insert(String::from("NOVOPS_CURRENT_ENVIRONMENT"), VariableOutput {
        name: String::from("NOVOPS_CURRENT_ENVIRONMENT"),
        value: ctx.env_name.clone(),
        quote_method: None,
    });

    // Set PS1 as current environment except if disabled in config
    // TODO check config for disable
    vars.insert(String::from("PS1"), VariableOutput {
        name: String::from("PS1"),
        value: format!("({:}) $PS1", ctx.env_name.clone()),
        quote_method: Some(String::from(QUOTE_METHOD_DOUBLE)),
    });

    vars
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
    let workdir_owner_uid = metadata.st_uid();
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
fn export_file_outputs(outputs: &NovopsOutputs) -> Result<(), anyhow::Error>{
    
    let files: Vec<FileOutput> = outputs.files.clone().into_values().collect();

    for f in files {
        let mut fd = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .mode(0o600)
            .open(&f.dest)
            .with_context(|| format!("Can't open {:?} for write with mode 0600", &f.dest))?;

        fd.write_all(&f.content)
            .with_context(|| format!("Can't write to {:?} after opening file descriptor", &f.dest))?;
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

    let vars_string = format_variable_outputs(format, &vars)?;
    
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

const QUOTE_METHOD_SINGLE: &str = "single";
const QUOTE_METHOD_DOUBLE: &str = "double";

pub fn wrap_var_output_in_quotes(var: &VariableOutput) -> Result<String, anyhow::Error> {

    let quoted_val = match var.quote_method.as_deref() {
        
        // 'somevalue'
        Some(QUOTE_METHOD_SINGLE) | None => {
            let escaped_val = var.value.replace("'", "'\"'\"'"); // 'a'b' => 'a'"'"'b'
            format!("'{:}'", escaped_val)
        }

        // "somevalue"
        Some(QUOTE_METHOD_DOUBLE) => {
            let escaped_val = var.value.replace("\"", "\"'\"'\""); // "a"b" => "a"'"'"b"
            format!("\"{:}\"", escaped_val)
        }
        
        Some(qm) => return Err(anyhow::anyhow!("Unkwon quote method '{:?}' for {:?}", qm, var)),
    };

    Ok(quoted_val)
}

/**
 * Transform VariableOutputs to string according to given format
 * and escape quotes
 */
pub fn format_variable_outputs(format: &str, vars: &Vec<VariableOutput>) -> Result<String, anyhow::Error> {
    let mut vars_string = String::new();

    // Prefix to build string such as `export MYVAR='xxx'` or `MYVAR=xxx` wihout export
    // TODO deprecated this global 'format' flag in favor of per-variable option
    let variable_prefix = match format {
        FORMAT_DOTENV_EXPORT => "export ",
        FORMAT_DOTENV_PLAIN => "",
        _ => return Err(anyhow::anyhow!("Unknown format {:}. Available formats: {:?}", format, FORMAT_ALL))
    };

    for v in vars {
        
        let quoted_val = wrap_var_output_in_quotes(&v)?;

        let val_line = format!("{:}{:}={:}\n", &variable_prefix, &v.name, &quoted_val);
        vars_string.push_str(&val_line);
    }

    Ok(vars_string)

}

fn create_symlink(lnk: &PathBuf, target: &PathBuf) -> Result<(), anyhow::Error> {
    let attr_result = fs::symlink_metadata(lnk);
    if attr_result.is_ok() {
        let attr = attr_result?;
        if attr.is_symlink() && attr.st_uid() == users::get_current_uid(){
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