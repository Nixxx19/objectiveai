#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    match objectiveai_cli::run(std::env::args_os()).await {
        Ok(output) => println!("{output}"),
        Err(e) => {
            println!("error: {e}");
            std::process::exit(1);
        }
    }
}
