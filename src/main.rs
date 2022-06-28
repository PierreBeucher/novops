#[macro_use]
extern crate enum_dispatch;
extern crate xdg;
mod novops;
mod bitwarden;

use novops::{NovopsConfig, NovopsEnvironment, ResolvableNovopsValue, ResolvedNovopsFile, ResolvedNovopsVariable};
use std::io::Error;
use clap::Parser;
use std::fs;

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

    let env_name = String::from("dev"); // todo as arg

    // resolve concrete variable values and file content from config
    let (
        user_resolved_vars, 
        user_resolved_files
    ) = parse_config(&config, &env_name);

    println!("Resolved vars: {:?}", user_resolved_vars);
    println!("Resolved files: {:?}", user_resolved_files);

    // env var pointing to files
    let file_resolved_vars:Vec<ResolvedNovopsVariable> = user_resolved_files.iter()
        .map(|f| f.variable.clone())
        .collect();

    let mut all_resolved_vars: Vec<ResolvedNovopsVariable> = Vec::new();
    all_resolved_vars.extend(user_resolved_vars);
    all_resolved_vars.extend(file_resolved_vars);

    write_files(user_resolved_files);
    
    let exportable_vars = build_exportable_vars(all_resolved_vars);

    println!("Exportable vars: {:?}", exportable_vars);

    write_exportable_vars(exportable_vars, env_name);
    
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
fn parse_config(config: &NovopsConfig, env_name: &String) -> (Vec<ResolvedNovopsVariable>, Vec<ResolvedNovopsFile>) {
    let env_config: &NovopsEnvironment = &config.environments[env_name];

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
        let variable_name = match &file_def.variable {
            Some(v) => v.clone(),
            None => format!("NOVOPS_FILE_{:}_{:}", env_name.to_uppercase(), file_key.to_uppercase()),
        };
        
        let resolved_file = ResolvedNovopsFile {
            dest: dest.clone(),
            variable: ResolvedNovopsVariable {
                name: variable_name,
                value: dest.clone()
            },
            content: file_def.content.resolve()
        };

        file_vec.push(resolved_file);
    }

    return (variable_vec, file_vec)
}

/**
 * Write resolved files to disk
 */
fn write_files(files:Vec<ResolvedNovopsFile>){
    for f in files {
        fs::write(f.dest, f.content).expect("Unable to write file");
    }
}

/**
 * build a string of exportable variables in the form
 *  VAR=value
 *  FOO=bar
 */
fn build_exportable_vars(vars: Vec<ResolvedNovopsVariable>) -> String{
    let mut exportable_vars = String::new();
    for v in vars{
        let s = format!("{:}=\"{:}\"\n", &v.name, &v.value);
        exportable_vars.push_str(&s);
    }

    return exportable_vars;
}

/**
 * Write exportable variables under runtime directory
 */
fn write_exportable_vars(vars: String, env_name: String){
    let var_file = xdg::BaseDirectories::with_prefix(format!("novops/{:}", &env_name))
        .unwrap()
        .place_runtime_file("vars")
        .unwrap();
    fs::write(var_file, vars).expect("Unable to write file");
}