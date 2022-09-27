use tokio;

#[tokio::main]
async fn main() -> () {
    match novops::parse_arg_and_run().await {
        Ok(e) => e,
        Err(e) => println!("{:?}", e),
    };
}

