mod cli;
mod commands;
mod config;
mod error;
mod oauth_materials;
mod output;

use clap::Parser;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = cli::Cli::parse();
    if let Err(err) = commands::run(cli).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
