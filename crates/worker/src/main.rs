mod cli;
mod commands;
mod config;
mod error;
mod oauth_materials;
mod output;

use clap::Parser;
use std::path::{Path, PathBuf};

#[tokio::main]
async fn main() {
    if version_requested() {
        print_version();
        return;
    }

    let cli = cli::Cli::parse();
    if let Err(err) = load_dotenv(cli.env_file.as_deref()) {
        eprintln!("{err}");
        std::process::exit(1);
    }

    if let Err(err) = commands::run(cli).await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn version_requested() -> bool {
    std::env::args_os()
        .skip(1)
        .any(|arg| arg == "--version" || arg == "-V")
}

fn print_version() {
    println!(
        "ibkrctl {}\nworker {}\ncore {}",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_VERSION"),
        ibkr_core::VERSION
    );
}

fn load_dotenv(env_file: Option<&Path>) -> Result<(), DotenvLoadError> {
    match env_file {
        Some(path) => dotenvy::from_path(path).map_err(|source| DotenvLoadError {
            path: path.to_path_buf(),
            source,
        }),
        None => {
            dotenvy::dotenv().ok();
            Ok(())
        }
    }
}

#[derive(Debug)]
struct DotenvLoadError {
    path: PathBuf,
    source: dotenvy::Error,
}

impl std::fmt::Display for DotenvLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "failed to load env file {}: {}",
            self.path.display(),
            self.source
        )
    }
}
