#[macro_use]
extern crate enum_dispatch;
extern crate xdg;
mod novops;
mod bitwarden;

use novops::{NovopsConfig, NovopsEnvironment, ResolvableNovopsValue, ResolvedNovopsFile, ResolvedNovopsVariable};
use std::io::Error;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "novops")]
#[clap(about = "Environment agnostic secret and config aggregator", long_about = None)]
struct NovopsArgs {
    #[clap(short, long, value_parser, default_value = ".novops.yml", help = "Config file to load")]
    config: String
}

fn main() -> Result<(), Error> {

    let args = NovopsArgs::parse();
    println!("Loading config: {:}", args.config);

    let config = read_config(&args.config).unwrap();
    println!("Found config: {:?}", config);

    let (
        resolved_variables, 
        resolved_files
    ) = parse_config(&config, String::from("dev"));

    println!("Resolved vars: {:?}", resolved_variables);
    println!("Resolved files: {:?}", resolved_files);

    Ok(())
}


fn read_config(config_path: &str) -> Result<NovopsConfig, serde_yaml::Error> {
    let f = std::fs::File::open(config_path).unwrap();
    let config: NovopsConfig = serde_yaml::from_reader(f).unwrap();

    return Ok(config);
}

/**
 * Parse configuration and resolve file and variables into concrete values 
 * Return a Vector of tuples for variables and files and their resolved values
 */
fn parse_config(config: &NovopsConfig, env_name: String) -> (Vec<ResolvedNovopsVariable>, Vec<ResolvedNovopsFile>) {
    let env_config: &NovopsEnvironment = &config.environments[&env_name];

    // resolve variables
    // straightforward: variable name is key in config, value is resolvable
    let mut variable_vec: Vec<ResolvedNovopsVariable> = Vec::new();
    for (var_key, var_value) in &env_config.variables {
        let resolved = ResolvedNovopsVariable{
            name: var_key.clone(),
            value: var_value.resolve()
        };
        variable_vec.push(resolved);
    }

    let xdg_basedir = xdg::BaseDirectories::with_prefix(format!("novops/{:}/files", env_name)).unwrap();

    // resolve file
    let mut file_vec: Vec<ResolvedNovopsFile> = Vec::new();
    for (file_key, file_def) in &env_config.files {

        // if dest provided, use it
        // otherwise default to XDG runtime dir named after file entry
        let dest = match &file_def.dest {
            Some(s) => s.clone(),
            None =>  xdg_basedir.place_runtime_file(&file_key)
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
        };

        // variable pointing to file path
        // if variable name is provided, use it
        // otherwise default to NOVOPS_<env>_<key>
        let variable = match &file_def.variable {
            Some(v) => v.clone(),
            None => format!("NOVOPS_FILE_{:}_{:}", env_name.to_uppercase(), file_key.to_uppercase()),
        };
        
        let resolved_file = ResolvedNovopsFile {
            dest: dest,
            variable: variable,
            content: file_def.content.resolve()
        };

        file_vec.push(resolved_file);
    }

    return (variable_vec, file_vec)
}

// /**
//  * Write resolved files to disk
//  */
// fn write_files(files:Vec<(NovopsFile, String)>){
//     for (file_def, content) in files {
//         let dest = match file_def.dest {
//             Some(s) => s,
//             None => format!("/run/user/1000/{:}", file_def.)
//         };

//         fs::write(file_def.dest, data).expect("Unable to write file");
//     }
// }