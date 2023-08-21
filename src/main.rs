use std::{process::exit, io, os::unix::process::CommandExt};

use anyhow::Context;
use tokio;
use clap::{Arg, Command, value_parser, ArgAction, crate_version};
use novops::{self, init_logger, get_config_schema};
use clap_complete::{generate, Shell};
use log::{error, info};
use std::process;
            

fn build_cli() -> Command {
    
    // Args used by both load and run commands
    let arg_config = Arg::new("config")
        .short('c')
        .long("config")
        .value_name("FILE")
        .help("Configuration to use.")
        .required(false)
        .num_args(1)
        .default_value(".novops.yml");
        
    let arg_environment = Arg::new("environment")
        .help("Environment to load. Prompt is not specified.")
        .long("env")
        .short('e')
        .value_name("ENVNAME")
        .required(false);
        
    let arg_workdir = Arg::new("working_dir")
        .help("Working directory under which files and secrets will be saved. \
            Default to XDG_RUNTIME_DIR if available, or a secured temporary files otherwise. \
            Warning: make sure to use a secured file when using this option. (i.e. directory with 0600 mod)")
        .long("working-dir")
        .short('w')
        .value_name("DIR")
        .required(false);

    let arg_dryrun = Arg::new("dry_run")
        .help("Perform a dry-run: not external service will be called and dummy outputs is written to disk. Used for testing purposes.")
        .long("dry-run")
        .value_name("DRY_RUN")
        .action(ArgAction::SetTrue)
        .required(false);

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
                .short('s')
                .value_name("SYMLINK")
                .required(false)
            )
            .arg(Arg::new("format")
                .help("Format for environment variables: dotenv-export|dotenv")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .default_value("dotenv-export")
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

        let args = novops::NovopsArgs{ 
            config: load_subc.get_one::<String>("config")
                .ok_or(anyhow::anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone(),
            env: load_subc.get_one::<String>("environment").map(String::from),
            format: load_subc.get_one::<String>("format")
                .ok_or(anyhow::anyhow!("Format is None. This is probably a bug as CLI defines default value."))?.clone(),
            working_directory: load_subc.get_one::<String>("working_dir").map(String::from),
            symlink: load_subc.get_one::<String>("symlink").map(String::from),
            dry_run: load_subc.get_one::<bool>("dry_run").map(|e| *e)
        };

        novops::load_environment_write_vars(&args).await
            .with_context(|| "Failed to load environment. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;

        exit(0)
    };

    if let Some(load_subc) = m.subcommand_matches("run") {
        
        let args = novops::NovopsArgs{ 
            config: load_subc.get_one::<String>("config")
                .ok_or(anyhow::anyhow!("Config is None. This is probably a bug as CLI defines default value."))?.clone(),
            env: load_subc.get_one::<String>("environment").map(String::from),
            format: String::from("TODOREMOVE"),
            working_directory: load_subc.get_one::<String>("working_dir").map(String::from),
            symlink: Some(String::from("TODOREMOVE")),
            dry_run: load_subc.get_one::<bool>("dry_run").map(|e| *e)
        };

        let mut command_arg: Vec<&String> = load_subc.get_many::<String>("command")
            .ok_or(anyhow::anyhow!("Command is required. This is probably a bug as CLi requires it."))?
            .collect();

        let outputs = novops::load_environment_no_write_vars(&args).await
            .with_context(|| "Failed to load environment. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;

        // first element of command argument is passed as child program
        // everything else is passed as arguments
        let child_program = command_arg.remove(0);
        let child_args = command_arg;

        let mut child = process::Command::new(&child_program);
        child.args(&child_args);

        info!("Running child command: {:?}", child);

        for(_ , val) in outputs.variables {
            child.env(&val.name, &val.value);
        }

        let error = child.exec();

        // exec should have replaced current process
        // if not, an error occured
        error!("Error running child command: {:}", error);
        exit(2)
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
