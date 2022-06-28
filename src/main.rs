#[macro_use]
extern crate enum_dispatch;
extern crate xdg;
mod novops;
mod bitwarden;

use novops::{NovopsConfig, NovopsEnvironment, ResolvableNovopsValue, ResolvedNovopsFile, ResolvedNovopsVariable};

use std::io::Error;
use clap::Parser;
use std::fs;
use text_io;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(name = "novops")]
#[clap(about = "Environment agnostic secret and config aggregator", long_about = None)]
struct NovopsArgs {
    #[clap(short = 'c', long, value_parser, default_value = ".novops.yml", help = "Config file to load")]
    config: String,

    #[clap(short = 'e', long, value_parser, help = "Name of environment to load")]
    env: Option<String>,

    #[clap(short = 'w', long = "workdir", value_parser, 
        help = "Working directory under which files and variable files will be stored by default. \
            Use XDG runtime directory if available ($XDG_RUNTIME_DIR/novops/<app>/<env>), default to current directory (.novops/<app>/<env>)")]
    working_directory: Option<String>
}

fn main() -> Result<(), Error> {
    // configure_logging();

    // Read CLI args and load config
    let args = NovopsArgs::parse();
    let config = read_config_file(&args.config).unwrap();
    let app_name = &config.name;

    // Env name is either specified as arg, as default or prompt user
    let env_name = read_environment_name(&config, &args.env);
    let env_config = &config.environments[&env_name];

    // working directory under which to save files
    let workdir = get_working_directory(&args, &app_name, &env_name);
    println!("Using workdir: {:?}", &workdir);

    // resolve concrete variable values and file content from config
    let (
        user_resolved_vars, 
        user_resolved_files
    ) = parse_environment(env_config, &env_name, &workdir);

    // env var pointing to files
    let file_resolved_vars:Vec<ResolvedNovopsVariable> = user_resolved_files.iter()
        .map(|f| f.variable.clone())
        .collect();

    let mut all_resolved_vars: Vec<ResolvedNovopsVariable> = Vec::new();
    all_resolved_vars.extend(user_resolved_vars);
    all_resolved_vars.extend(file_resolved_vars);

    // write resolved files and variable
    prepare_workir(&workdir);
    write_resolved_files(user_resolved_files);
    
    let exportable_vars = build_exportable_vars(all_resolved_vars);

    let exported_var_path = write_exportable_vars(&exportable_vars, &workdir);

    println!("Novops environment loaded ! Export variables with:");
    println!("  source {:}", exported_var_path);
    
    Ok(())
}

fn read_config_file(config_path: &str) -> Result<NovopsConfig, serde_yaml::Error> {
    let f = std::fs::File::open(config_path).unwrap();
    let config: NovopsConfig = serde_yaml::from_reader(f).unwrap();

    return Ok(config);
}

/** 
 * Read the environment name with this precedence (higher to lowest):
 * - CLI flag
 * - prompt user (using config's default if no choice given)
 */
fn read_environment_name(config: &NovopsConfig, flag: &Option<String>) -> String {

    match flag {
        Some(e) => e.clone(),
        None => prompt_for_environment(config)
    }
}

/**
 * Retrieve working directory where files and exportable variable file will be written by default
 * with the following precedence:
 * - CLI flag working directory
 * - XDG runtime dir (if available), such as $XDg_RUNTIME_DIR/novops/<app>/<env>
 * - Current directory such as .novops/<app>/<env>
 */
fn get_working_directory(args: &NovopsArgs, app_name: &String, env_name: &String) -> String {
    let workdir_suffix = format!("novops/{:}/{:}", app_name, env_name);


    match &args.working_directory {
        Some(w) => return w.clone(),
        None => {
            let xdg_dir = xdg::BaseDirectories::with_prefix(&workdir_suffix).unwrap();

            // Try to use XDG if available
            // Fallback to current directory
            if xdg_dir.has_runtime_directory() {
                let runtime_dir = xdg_dir.get_runtime_directory().unwrap().clone().into_os_string().into_string().unwrap().clone();
                return format!("{:}/{:}", runtime_dir, workdir_suffix)
            } else {
                return format!(".{:}", &workdir_suffix)
            }
        },
    }
    
}

/**
 * Prepare working directory (ensure directories exists so that we can create files)
 */
fn prepare_workir(workir: &String){
    fs::create_dir_all(workir).unwrap();
    fs::create_dir_all(format!("{:}/files", workir)).unwrap();
}

/**
 * Prompt user for environment name
 */
fn prompt_for_environment(config: &NovopsConfig) -> String{

    // read config for environments and eventual default environment 
    let environments = config.environments.keys().cloned().collect::<Vec<String>>();
    let default_environment: Option<String> = match &config.default {
        Some(d) => match &d.environment {
            Some(default_env) => Some(default_env.clone()),
            None => None
        },
        None => None
    };

    // prompt user, show default environment if any
    let mut prompt_msg = format!("Select environment: {:}", environments.join(", "));
    if default_environment.is_some(){
        prompt_msg.push_str(&format!(" (default: {:})", default_environment.unwrap()));
    }
    println!("{prompt_msg}");

    let selected: String = text_io::read!("{}\n");

    if selected.is_empty() {
        match &config.default {
            Some(d) => match &d.environment {
                Some(default_env) => default_env.clone(),
                None => panic!("No environment selected and no default in config."),
            },
            None => panic!("No environment selected and no default in config."),
        }
    } else {
        return selected
    }
}

/**
 * Parse configuration and resolve file and variables into concrete values 
 * Return a Vector of tuples for variables and files and their resolved values
 */
fn parse_environment(env_config: &NovopsEnvironment, env_name: &String, workdir_root: &String) -> (Vec<ResolvedNovopsVariable>, Vec<ResolvedNovopsFile>) {
    
    // subpath in workdir where to store file by default
    let file_workdir = format!("{:}/files", workdir_root);

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

    // resolve file
    let mut file_vec: Vec<ResolvedNovopsFile> = Vec::new();
    for (file_key, file_def) in &env_config.files {

        // if dest provided, use it
        // otherwise use working directory
        let dest = match &file_def.dest {
            Some(s) => s.clone(),
            None => format!("{:}/{:}", file_workdir, file_key)
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
fn write_resolved_files(files:Vec<ResolvedNovopsFile>){
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
        let s = format!("export {:}=\"{:}\"\n", &v.name, &v.value);
        exportable_vars.push_str(&s);
    }

    return exportable_vars;
}

/**
 * Write exportable variables under runtime directory
 */
fn write_exportable_vars(vars: &String, working_dir: &String) -> String{
    let var_file = format!("{:}/vars", working_dir);
    fs::write(&var_file, vars).expect("Unable to write file");
    return var_file;
}