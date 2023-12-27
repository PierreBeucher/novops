use std::io;

use anyhow::{Context, anyhow};

use clap::{Arg, Command, value_parser, ArgAction, crate_version, ArgMatches};
use novops::{self, init_logger, get_config_schema, NovopsLoadArgs};
use clap_complete::{generate, Shell};
use std::collections::HashMap;

// Format for novops list commands
const LIST_CMD_OUTPUT_PLAIN: &str = "plain";
const LIST_CMD_OUTPUT_JSON: &str = "json";

fn build_cli() -> Command {
    
    // Args used by both load and run commands
    let arg_config = Arg::new("config")
        .short('c')
        .long("config")
        .env("NOVOPS_CONFIG")
        .value_name("FILE")
        .help("Configuration to use.")
        .required(false)
        .num_args(1)
        .default_value(".novops.yml");
        
    let arg_environment = Arg::new("environment")
        .help("Environment to load. Prompt if not specified.")
        .long("env")
        .env("NOVOPS_ENVIRONMENT")
        .short('e')
        .value_name("ENVNAME")
        .required(false);
        
    let arg_workdir = Arg::new("working_dir")
        .help("Working directory under which files and secrets will be saved. \
            Default to XDG_RUNTIME_DIR if available, or a secured temporary files otherwise. \
            Warning: make sure to use a secured file when using this option. (i.e. directory with 0600 mod)")
        .long("working-dir")
        .env("NOVOPS_WORKDIR")
        .short('w')
        .value_name("DIR")
        .required(false);

    let arg_skip_workdir_check = Arg::new("skip_workdir_check")
        .help("Skip working directory safety checks for permissions and ownership. By default, working directory is ensured to be owned by current user without group or world access.")
        .long("skip-workdir-check")
        .env("NOVOPS_SKIP_WORKDIR_CHECK")
        .value_name("SKIP_WORKDIR_CHECK")
        .action(ArgAction::SetTrue)
        .default_value("false")
        .required(false);

    let arg_dryrun = Arg::new("dry_run")
        .help("Perform a dry-run: no external service will be called and dummy secrets are generated.")
        .long("dry-run")
        .env("NOVOPS_DRY_RUN")
        .value_name("DRY_RUN")
        .action(ArgAction::SetTrue)
        .required(false);

    let arg_output_format = Arg::new("format")
        .value_name("OUTPUT_FORMAT")
        .help("Output format.")
        .short('o')
        .default_value("plain");

     
    Command::new("novops")
        .about("Cross-plaform secret loader")
        .version(crate_version!())
        .author("Pierre Beucher")
        .subcommand(
            Command::new("load")
            .about("Load a Novops environment. Output resulting environment variables to stdout or to a file using -s/--symlink. ")
            .arg(&arg_config)
            .arg(&arg_environment)
            .arg(&arg_workdir)
            .arg(&arg_dryrun)
            .arg(&arg_skip_workdir_check)
            .arg(Arg::new("symlink")
                .help("Create a symlink pointing to generated environment variable file. Implies -o 'workdir'")
                .long("symlink")
                .env("NOVOPS_LOAD_SYMLINK")
                .short('s')
                .value_name("SYMLINK")
                .required(false)
            )
            .arg(Arg::new("format")
                .help("Format for environment variables: dotenv-export|dotenv")
                .short('f')
                .long("format")
                .env("NOVOPS_LOAD_FORMAT")
                .value_name("FORMAT")
                .default_value("dotenv-export")
            )
            .arg(Arg::new("skip_tty_check")
                .help("Do not check if stdout is a tty (terminal), risking exposing secrets on screen. This is unsecure.")
                .long("skip-tty-check")
                .env("NOVOPS_LOAD_SKIP_TTY_CHECK")
                .value_name("DRY_RUN")
                .action(ArgAction::SetTrue)
                .required(false)
            )
        )
        .subcommand(
            Command::new("run")
            .about("Run a command with loaded environment variables and files.")
            .long_about("Run a command with loaded environment variables and files. \n\
                Example: \n\
                novops run sh\n\
                novops run -- terraform apply\n\
                "
            )
            .arg(&arg_config)
            .arg(&arg_environment)
            .arg(&arg_workdir)
            .arg(&arg_dryrun)
            .arg(&arg_skip_workdir_check)
            .arg(Arg::new("command")
                .value_name("COMMAND")
                .action(ArgAction::Append)
                .help("Command to run.")
                .required(true)
            )
        )
        .subcommand(
            Command::new("list")
                .subcommand(
                    Command::new("environments")
                    .about("List available environments.")                    
                    .arg(&arg_config)
                    .arg(&arg_output_format)
                )
                .subcommand(
                    Command::new("outputs")
                    .about("List outputs for an environment.")
                    .arg(&arg_config)
                    .arg(&arg_environment)
                    .arg(&arg_output_format)
                )
        )
        .subcommand(
            Command::new("completion")
            .about("Output completion code for various shells.")
            .long_about("Output completion code for various shells. Examples: \n\
                    - bash: `source <(novops completion bash)` \n\
                    - zsh: `novops completion zsh > _novops && fpath+=($PWD) && compinit` \n\
                    \n\
                    See https://docs.rs/clap_complete/latest/clap_complete/enum.Shell.html for supported shells
                "
            )
            .arg(Arg::new("shell")
                .action(ArgAction::Set)
                .value_name("SHELL")
                .value_parser(value_parser!(Shell))
                .required(true)
            )
        )
        .subcommand(
            Command::new("schema")
            .about("Output Novops config JSON schema")
        )
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    
    init_logger(); // first things first

    let mut app = build_cli();

    let m = app.get_matches_mut();

    let result = match m.subcommand() {
        Some(("load", load_subc)) => cmd_load(load_subc).await,
        Some(("run", run_subc)) => cmd_run(run_subc).await,
        Some(("list", list_subc)) => {
            match list_subc.subcommand() {
                Some(("environments", list_envs_subc)) => cmd_list_envs(list_envs_subc).await,
                Some(("outputs", list_outputs_subc)) => cmd_list_outputs(list_outputs_subc).await,
                _ => cmd_error().await,
            }
        }
        Some(("completion", cmd)) => cmd_completion(cmd, app).await,
        Some(("schema", schema_args)) => cmd_schema(schema_args).await,
        _ => cmd_error().await,
    };

    result.with_context(|| "An error occured. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;
    
    Ok(())
}

//
// COMMAND FUNCTIONS
// Below functions are for sub-commands of main
//

async fn cmd_list_envs(cmd_args: &ArgMatches) -> Result<(), anyhow::Error> {
    let config_file = cmd_args.get_one::<String>("config")
        .ok_or(anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone();

    let output_format = cmd_args.get_one::<String>("format")
        .ok_or(anyhow!("Format is None. This is probably a bug as CLI defines default value."))?.clone();

    let envs = novops::list_environments(&config_file).await
        .with_context(|| "Failed to list environments.")?;

    match output_format.as_str() {
        LIST_CMD_OUTPUT_JSON => {
            let json = serde_json::to_string(&envs)
                .with_context(|| "Failed to serialize environments to JSON.")?;
            println!("{}", json);
        },
        LIST_CMD_OUTPUT_PLAIN => {
            println!("{}", envs.join("\n"));
        },
        _ => {
            return Err(anyhow!("Unknown format: {}", output_format));
        }
    }

    Ok(())
}

async fn cmd_list_outputs(cmd_args: &ArgMatches) -> Result<(), anyhow::Error> {

    let config_file = cmd_args.get_one::<String>("config")
        .ok_or(anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone();

    let env_name = cmd_args.get_one::<String>("environment").map(String::from);

    let output_format = cmd_args.get_one::<String>("format")
        .ok_or(anyhow!("Format is None. This is probably a bug as CLI defines default value."))?.clone();

    let outputs = novops::list_outputs_for_environment(&config_file, env_name).await
        .with_context(|| "Failed to list outputs.")?;
    
    
    // Result in the form { "variables": [ "VAR_NAME1", "VAR_NAME2" ], "files": ["/path/to/file1", "/path/to/file2", ...] }
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    result.insert("variables".to_string(), 
        outputs.variables.iter().map(|e| e.1.name.clone()).collect::<Vec<String>>()
    );
    result.insert("files".to_string(), 
        outputs.files.iter().map(|f| f.0.clone()).collect::<Vec<String>>()
    );

    match output_format.as_str() {
        LIST_CMD_OUTPUT_JSON => {
            let json = serde_json::to_string(&result)
                .with_context(|| "Failed to serialize environments to JSON.")?;
            println!("{}", json);
        },
        LIST_CMD_OUTPUT_PLAIN => {
            println!("Variables:");
            println!();
            println!("{}", result.get("variables").unwrap().join("\n"));
            println!();
            println!("Files:");
            println!();
            println!("{}", result.get("files").unwrap().join("\n"));
        },
        _ => {
            return Err(anyhow!("Unknown output format: {}", output_format));
        }
    }

    Ok(())
}

async fn cmd_run(cmd_args: &ArgMatches) -> Result<(), anyhow::Error> {
    let novops_load_args = build_novops_args(cmd_args)?;

    let command_args: Vec<&String> = cmd_args.get_many::<String>("command")
        .ok_or(anyhow!("Command is required. This is probably a bug as CLi requires it."))?
        .collect();

    novops::load_environment_and_exec(&novops_load_args, command_args).await
        .with_context(|| "Failed to load environment and exec command.")?;

    Ok(())
}   

async fn cmd_load(cmd_args: &ArgMatches) -> Result<(), anyhow::Error> {
    
    let symlink = cmd_args.get_one::<String>("symlink").map(String::from);
        
    let env_format = cmd_args.get_one::<String>("format")
        .ok_or(anyhow!("Format is None. This is probably a bug as CLI defines default value."))?.clone();

    let skip_tty_check = cmd_args.get_one::<bool>("skip_tty_check").copied()
        .ok_or(anyhow!("skip_tty_check is None. This is probably a bug as CLI defines default value."))?;

    let novops_load_args = build_novops_args(cmd_args)?;

    novops::load_environment_write_vars(&novops_load_args, &symlink, &env_format, skip_tty_check).await
        .with_context(|| "Failed to load environment.")?;

    Ok(())
}

async fn cmd_completion(cmd_args: &ArgMatches, mut app: Command) -> Result<(), anyhow::Error> {
    let name = app.get_name().to_string();
    let shell_arg = cmd_args.get_one::<Shell>("shell")
        .ok_or(anyhow!("Shell is required"))?;
    
    generate(*shell_arg, &mut app, name, &mut io::stdout());

    Ok(())
}

async fn cmd_schema(_: &ArgMatches) -> Result<(), anyhow::Error> {
    let schema = get_config_schema()?;
    println!("{}", schema);
    Ok(())
}

async fn cmd_error() -> Result<(), anyhow::Error> {
    Err(anyhow!("Please provide a valid subcommand. Use --help to see available commands."))
}

//
// UTILS
// 

/**
 * Build Novops load arguments using subcommand and expected arguments
 * Error if subcommand does not have all expected argument for Novops load
 */
fn build_novops_args(cmd_args: &ArgMatches) -> Result<NovopsLoadArgs, anyhow::Error> {
    let args = novops::NovopsLoadArgs{ 
        config: cmd_args.get_one::<String>("config")
            .ok_or(anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone(),
        env: cmd_args.get_one::<String>("environment").map(String::from),
        working_directory: cmd_args.get_one::<String>("working_dir").map(String::from),
        skip_working_directory_check: cmd_args.get_one::<bool>("skip_workdir_check").copied(),
        dry_run: cmd_args.get_one::<bool>("dry_run").copied()
    };

    Ok(args)
}