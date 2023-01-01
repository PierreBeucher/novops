use std::{process::exit, io};

use tokio;
use clap::{Arg, Command, value_parser, ArgAction, crate_version};
use novops::{self, core::NovopsConfigFile};
use schemars::schema_for;
use clap_complete::{generate, Shell};

fn build_cli() -> Command {
    let app = Command::new("novops")
        .about("Platform agnostic secret manager")
        .version(crate_version!())
        .author("Novadiscovery")
        .subcommand(
            Command::new("load")
            .about("Load a Novops environment")
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
                .help("Create a symlink pointing to working directory.")
                .long("symlink")
                .short('s')
                .value_name("SYMLINK_PATH")
                .required(false)
            )
            .arg(Arg::new("dry_run")
                .help("Perform a dry-run: not external service is be called and dummy outputs is written to disk. Used for testing purposes.")
                .long("dry-run")
                .value_name("DRY_RUN")
                .action(ArgAction::SetTrue)
                .required(false)
            )
        )
        .subcommand(
            Command::new("completion")
            .about("Output completion code for the shells support by clap_complete::\
                    - For bash, run `source <(novops completion bash)` \
                    - For zsh, run `novops completion zsh > _novops && fpath+=($PWD) && compinit`"
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
async fn main() -> () {

    let mut app = build_cli();

    let m = app.get_matches_mut();

    if let Some(load_subc) = m.subcommand_matches("load") {

        let args = novops::NovopsArgs{ 
            config: load_subc.get_one::<String>("config").unwrap().clone(),
            env: load_subc.get_one::<String>("environment").map(String::from),
            working_directory: load_subc.get_one::<String>("working_dir").map(String::from),
            symlink: load_subc.get_one::<String>("symlink").map(String::from),
            dry_run: load_subc.get_one::<bool>("dry_run").map(|e| *e)
        };

        novops::load_environment(args).await.unwrap();
        exit(0)
    };

    if let Some(cmd) = m.subcommand_matches("completion") {
        let name = app.get_name().to_string();
        generate(*cmd.get_one::<Shell>("shell").unwrap(), &mut app, name, &mut io::stdout());
        exit(0)
    };

    if let Some(_cmd) = m.subcommand_matches("schema") {
        let schema = schema_for!(NovopsConfigFile);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        exit(0)
    }
    
    println!("Please provide a subcommand. Use --help to see available commands.");
    exit(1)
}
