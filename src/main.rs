use std::{process::exit, io};

use anyhow::Context;
use tokio;
use clap::{Arg, Command, value_parser, ArgAction, crate_version};
use novops::{self, init_logger, get_config_schema};
use clap_complete::{generate, Shell};
use log::error;

fn build_cli() -> Command {
    let app = Command::new("novops")
        .about("Cross-plaform secret loader")
        .version(crate_version!())
        .author("Pierre Beucher")
        .subcommand(
            Command::new("load")
            .about("Load a Novops environment. Output resulting environment variables to stdout or to a file using -s/--symlink. ")
            .arg(Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration to use.")
                .required(false)
                .num_args(1)
                .default_value(".novops.yml")
            )
            .arg(Arg::new("environment")
                .help("Environment to load. Prompt is not specified.")
                .long("env")
                .short('e')
                .value_name("ENVNAME")
                .required(false)
            )
            .arg(Arg::new("format")
                .help("Format for environment variables: dotenv-export|dotenv")
                .short('f')
                .long("format")
                .value_name("FORMAT")
                .default_value("dotenv-export")
            )
            .arg(Arg::new("working_dir")
                .help("Working directory under which files and secrets will be saved. \
                    Default to XDG_RUNTIME_DIR if available, or a secured temporary files otherwise. \
                    Warning: make sure to use a secured file when using this option. (i.e. directory with 0600 mod)")
                .long("working-dir")
                .short('w')
                .value_name("DIR")
                .required(false)
            )
            .arg(Arg::new("symlink")
                .help("Create a symlink pointing to generated environment variable file. Implies -o 'workdir'")
                .long("symlink")
                .short('s')
                .value_name("SYMLINK")
                .required(false)
            )
            .arg(Arg::new("dry_run")
                .help("Perform a dry-run: not external service will be called and dummy outputs is written to disk. Used for testing purposes.")
                .long("dry-run")
                .value_name("DRY_RUN")
                .action(ArgAction::SetTrue)
                .required(false)
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

        novops::load_environment(args).await
            .with_context(|| "Failed to load environment. Set environment variable RUST_LOG=[trace|debug|info|warn] or RUST_BACKTRACE=1 for more verbosity.")?;

        exit(0)
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
