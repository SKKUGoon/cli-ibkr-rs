use clap::Args;

#[derive(Debug, Args)]
pub struct PositionsLiveArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long, value_parser = ["a", "d"])]
    pub direction: Option<String>,
}
