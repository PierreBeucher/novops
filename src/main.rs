use std::fs;
use std::io::Error;

fn main() -> Result<(), Error> {

    let fc = fs::read_to_string(".novops.yml")?;
    println!("Loaded config:\n{}", fc);

    Ok(())
}