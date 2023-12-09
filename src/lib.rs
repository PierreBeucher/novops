pub mod core;
pub mod modules;

use crate::core::{ResolveTo, NovopsEnvironmentInput, NovopsConfigFile, NovopsContext};
use crate::modules::files::FileOutput;
use crate::modules::variables::VariableOutput;
use log::{info, debug, error};

use std::os::linux::fs::MetadataExt;
use std::os::unix::prelude::OpenOptionsExt;
use std::{fs, io::prelude::*, os::unix::prelude::PermissionsExt};
use users;
use anyhow::{self, Context};
use std::os::unix;
use std::path::PathBuf;
use std::env;
use is_terminal::IsTerminal; // todo use std::io::IsTerminal; rust 1.71+
use std::collections::HashMap;
use schemars::schema_for;
use std::process::Command;
use std::os::unix::process::CommandExt;

#[derive(Debug)]
pub struct NovopsLoadArgs {
    pub config: String,

    pub env: Option<String>,

    pub working_directory: Option<String>,

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
    check_stdout_tty_and_exit(skip_tty_check, &symlink);

    let outputs = load_context_and_resolve(&args).await?;

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

    let outputs = load_context_and_resolve(&args).await?;

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

    let ctx = make_context(&args).await?;
    let novops_env = get_current_environment(&ctx).await?;
    
    // Revole inputs and export (write data to disk)
    let (var_out, file_out) = 
        resolve_environment_inputs(&ctx, novops_env).await?;

    return Ok(NovopsOutputs { 
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

    let mut command = Command::new(&child_program);
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
    let workdir = prepare_working_directory(&args, &app_name, &env_name)?;
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

    let mut variable_outputs: HashMap<String, VariableOutput> = HashMap::new();
    let mut file_outputs: HashMap<String, FileOutput> = HashMap::new();

    // Expose Novops internal variables
    // Load first so user can override via config if needed
    variable_outputs.insert(String::from("NOVOPS_ENVIRONMENT"), VariableOutput {
        name: String::from("NOVOPS_ENVIRONMENT"),
        value: ctx.env_name.clone()
    });
    
    for v in &inputs.variables.unwrap_or(vec![]) {
        let val = v.resolve(&ctx).await
            .with_context(|| format!("Couldn't resolve variable input {:?}", v))?;
        
        variable_outputs.insert(v.name.clone(), val);
    };

    for f in &inputs.files.unwrap_or(vec![]) {
        let r = f.resolve(&ctx).await
            .with_context(|| format!("Couldn't resolve file input {:?}", f))?;
        
        let fpath_str = r.dest.to_str()
            .ok_or(anyhow::anyhow!("Couldn't convert PathBuf '{:?}' to String", &r.dest))?;

        // FileInput generates both var and file output
        variable_outputs.insert(fpath_str.to_string(), r.variable.clone());
        file_outputs.insert(fpath_str.to_string(), r.clone());
    };

    match &inputs.aws {
        Some(aws) => {
            let r = aws.assume_role.resolve(&ctx).await
                .with_context(|| format!("Could not resolve AWS input {:?}", aws))?;

            for vo in r {
                variable_outputs.insert(vo.name.clone(), vo);
            }
            
        },
        None => (),
    }

    match &inputs.hashivault {
        Some(hashivault) => {
            let r = hashivault.aws.resolve(&ctx).await
                .with_context(|| format!("Could not resolve Hashivault input {:?}", hashivault))?;

            for vo in r {
                variable_outputs.insert(vo.name.clone(), vo);
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

    return Ok(config);
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
        Some(wd) => PathBuf::from(wd),
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
    return if workdir.is_absolute() {
        Ok(workdir)
    } else {
        Ok(env::current_dir()
        .with_context(|| "Couldn't get process current working directory")?
        .join(&workdir))
    };
}

/** 
 * Prepare a workding directory using xdg
 * Returns an error if XDG is not available or failed somehow
*/
fn prepare_working_directory_xdg(app_name: &String, env_name: &String) -> Result<PathBuf, anyhow::Error> {
    let xdg_prefix = format!("novops/{:}/{:}", app_name, env_name);

    let xdg_basedir = xdg::BaseDirectories::new()?
        .create_runtime_directory(&xdg_prefix)?;
    
    return Ok(xdg_basedir);
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
    
    return Ok(PathBuf::from(workdir));
}

/**
 * Prompt user for environment name
 */
fn prompt_for_environment(config_file_data: &NovopsConfigFile) -> Result<String, anyhow::Error>{

    // read config for environments and eventual default environment 
    let environments = config_file_data.environments.keys().cloned().collect::<Vec<String>>();
    let default_env_value = String::default();
    let default_env = config_file_data.config.as_ref()
        .and_then(|c| c.default.as_ref())
        .and_then(|d| d.environment.as_ref())
        .unwrap_or(&default_env_value);

    // prompt user, show default environment if any
    // only show 'default: xxx' if default environment is defined
    let mut prompt_msg = format!("Select environment: {:}", environments.join(", "));

    if ! default_env.is_empty() {
        prompt_msg.push_str(&format!(" (default: {:})", &default_env))
    };
    
    // use println, we want to prompt user not log something
    eprintln!("{prompt_msg}");
    
    let mut read_env = String::new();
    std::io::stdin().read_line(&mut read_env)
        .with_context(|| "Error reading stdin for environment name user input.")?;

    let selected_env = read_env.trim_end().to_string();

    return if selected_env.is_empty() {
        if default_env.is_empty() {
            Err(anyhow::anyhow!("No environment selected and no default in config."))
        } else {
            Ok(default_env.clone())
        }
    } else {
        Ok(selected_env)
    };
}

/**
 * Write resolved files to disk
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
 * Write a sourceable environment variable file to disk
 * With a content like
 * '
 *  export VAR=value
 *  export FOO=bar
 * '  
 * 
 */
fn export_variable_outputs(format: &str, symlink: &Option<String>, vars: &Vec<VariableOutput>, working_dir: &PathBuf) -> Result<(), anyhow::Error>{

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

    return Ok(())    
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
        let safe_val = v.value.replace("'", "'\"'\"'");
        safe_vars.push(VariableOutput { name: v.name.clone(), value: safe_val })
    }

    return safe_vars;
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

    return Ok(vars_string);

}

fn create_symlink(lnk: &PathBuf, target: &PathBuf) -> Result<(), anyhow::Error> {
    let attr_result = fs::symlink_metadata(&lnk);
    if attr_result.is_ok() {
        let attr = attr_result?;
        if attr.is_symlink() && attr.st_uid() == users::get_current_uid(){
            debug!("Deleting existing symlink {:?} before creating new one", &lnk);
            fs::remove_file(&lnk).with_context(|| format!("Couldn't remove existing symlink {:?}", &lnk))?;
        } else {
            return Err(anyhow::anyhow!("Symlink creation error: {:?} already exists and is not a symlink you own.", &lnk));
        }
    } 
    
    unix::fs::symlink(&target, &lnk)
        .with_context(|| format!("Couldn't create symlink {:?} -> {:?}", &lnk, &target))
}
    
pub fn get_config_schema() -> Result<String, anyhow::Error>{
    let schema = schema_for!(NovopsConfigFile);
    let output = serde_json::to_string_pretty(&schema)
        .with_context(|| "Couldn't convert JSON schema to pretty string.")?;
    return Ok(output)
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

    return false
}