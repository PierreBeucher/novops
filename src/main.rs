use std::{process::exit, io};

use anyhow::Context;
use tokio;
use clap::{Arg, Command, value_parser, ArgAction, crate_version};
use novops::{self, init_logger, get_config_schema};
use clap_complete::{generate, Shell};
use log::error;
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

    let app = Command::new("novops")
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
    ; 
    return app
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    
    init_logger(); // first things first

    let mut app = build_cli();

    let m = app.get_matches_mut();

    if let Some(load_subc) = m.subcommand_matches("load") {

        let symlink = load_subc.get_one::<String>("symlink").map(String::from);
        
        let env_format = load_subc.get_one::<String>("format")
            .ok_or(anyhow::anyhow!("Format is None. This is probably a bug as CLI defines default value."))?.clone();

        let skip_tty_check = load_subc.get_one::<bool>("skip_tty_check").map(|e| *e)
            .ok_or(anyhow::anyhow!("skip_tty_check is None. This is probably a bug as CLI defines default value."))?.clone();

        let args = novops::NovopsLoadArgs{ 
            config: load_subc.get_one::<String>("config")
                .ok_or(anyhow::anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone(),
            env: load_subc.get_one::<String>("environment").map(String::from),
            working_directory: load_subc.get_one::<String>("working_dir").map(String::from),
            dry_run: load_subc.get_one::<bool>("dry_run").map(|e| *e)
        };

        novops::load_environment_write_vars(&args, &symlink, &env_format, skip_tty_check).await
            .with_context(|| "Failed to load environment. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;

        exit(0)
    };

    if let Some(load_subc) = m.subcommand_matches("run") {
        
        let args = novops::NovopsLoadArgs{ 
            config: load_subc.get_one::<String>("config")
                .ok_or(anyhow::anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone(),
            env: load_subc.get_one::<String>("environment").map(String::from),
            working_directory: load_subc.get_one::<String>("working_dir").map(String::from),
            dry_run: load_subc.get_one::<bool>("dry_run").map(|e| *e)
        };

        let command_args: Vec<&String> = load_subc.get_many::<String>("command")
            .ok_or(anyhow::anyhow!("Command is required. This is probably a bug as CLi requires it."))?
            .collect();

        novops::load_environment_and_exec(&args, command_args).await
            .with_context(|| "Failed to load environment and exec command. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;

        exit(2)
    };

    if let Some(list_subc) = m.subcommand_matches("list") {


        if let Some(list_envs_subc) = list_subc.subcommand_matches("environments") {
            let config_file = list_envs_subc.get_one::<String>("config")
                .ok_or(anyhow::anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone();

            let envs = novops::list_environments(&config_file).await
                .with_context(|| "Failed to list environments. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;

            let output_format = list_envs_subc.get_one::<String>("format")
                .ok_or(anyhow::anyhow!("Format is None. This is probably a bug as CLI defines default value."))?.clone();

            match output_format.as_str() {
                LIST_CMD_OUTPUT_JSON => {
                    let json = serde_json::to_string(&envs)
                        .with_context(|| "Failed to serialize environments to JSON. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;
                    println!("{}", json);
                    exit(0)
                },
                LIST_CMD_OUTPUT_PLAIN => {
                    println!("{}", envs.join("\n"));
                    exit(0)
                },
                _ => {
                    println!("Unknown format: {}", output_format);
                    exit(1)
                }
            }
        };
        
        if let Some(list_outputs_subc) = list_subc.subcommand_matches("outputs") {
            let config_file = list_outputs_subc.get_one::<String>("config")
                .ok_or(anyhow::anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone();

            let env_name = list_outputs_subc.get_one::<String>("environment").map(String::from);

            let output_format = list_outputs_subc.get_one::<String>("format")
                .ok_or(anyhow::anyhow!("Format is None. This is probably a bug as CLI defines default value."))?.clone();

            let outputs = novops::list_outputs_for_environment(&config_file, env_name).await
                .with_context(|| "Failed to list outputs. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;
            
            
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
                        .with_context(|| "Failed to serialize environments to JSON. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;
                    println!("{}", json);
                    exit(0)
                },
                LIST_CMD_OUTPUT_PLAIN => {
                    println!("Variables:");
                    println!();
                    println!("{}", result.get("variables").unwrap().join("\n"));
                    println!();
                    println!("Files:");
                    println!();
                    println!("{}", result.get("files").unwrap().join("\n"));
                    exit(0)
                },
                _ => {
                    println!("Unknown format: {}", output_format);
                    exit(1)
                }
            }
        };
    };

    if let Some(cmd) = m.subcommand_matches("completion") {
        let name = app.get_name().to_string();
        let shell_arg = cmd.get_one::<Shell>("shell")
            .ok_or(anyhow::anyhow!("Shell is required"))?;
        
        generate(*shell_arg, &mut app, name, &mut io::stdout());
        exit(0)
    };

    if let Some(_cmd) = m.subcommand_matches("schema") {
        let schema = get_config_schema()?;
        println!("{}", schema);
        exit(0)
    }
    
    error!("Please provide a subcommand. Use --help to see available commands.");
    exit(1)
}
