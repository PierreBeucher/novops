use std::process::exit;

use tokio;
use clap::{Arg, App, SubCommand};
use novops;

#[tokio::main]
async fn main() -> () {

    let app = App::new("novops")
        .about("Platform agnostic secret aggregator")
        .subcommand(
            SubCommand::with_name("load")
            .about("Load a Novops environment")
            .arg(Arg::with_name("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration to use.")
                .takes_value(true)
                .required(false)
                .default_value(".novops.yml")
            )
            .arg(Arg::with_name("environment")
                .help("Environment to load. Prompt is not specified.")
                .long("env")
                .short('e')
                .value_name("ENVNAME")
                .takes_value(true)
                .required(false)
            )
            .arg(Arg::with_name("working_dir")
                .help("Working directory under which files and secrets will be saved. \
                    Default to XDG_RUNTIME_DIR if available, or a secured temporary files otherwise. \
                    Warning: make sure to use a secured file when using this option. (i.e. directory with 0600 mod)")
                .long("working-dir")
                .short('w')
                .value_name("DIR")
                .takes_value(true)
                .required(false)
            )
            .arg(Arg::with_name("symlink")
                .help("Create a symlink pointing to working directory.")
                .long("symlink")
                .short('s')
                .value_name("SYMLINK_PATH")
                .takes_value(true)
                .required(false)
            )
        )
    ; 
        
    let m = app.get_matches();

    match m.subcommand_matches("load") {
        Some(load_subc) => {
            let args = novops::NovopsArgs{ 
                config: load_subc.value_of("config").unwrap().to_string(),
                env: load_subc.value_of("environment").map(String::from),
                working_directory: load_subc.value_of("working_dir").map(String::from),
                symlink: load_subc.value_of("symlink").map(String::from)
            };

            novops::load_environment(args).await.unwrap();
            exit(0)
        },
        None => {},
    };
    
    println!("Please provide a subcommand. Use --help to see available commands.");
    exit(1)
}

