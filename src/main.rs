mod cli;
mod client;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use client::ChatClient;
use ui::WaitUi;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let prompt = args.prompt_text();
    let api_key = resolve_api_key(args.api_key);
    let client = ChatClient::new(args.base_url, api_key);

    let wait_ui = WaitUi::start(!args.no_orb, !args.no_mystical);
    let result = client.ponder(&args.model, &prompt).await;
    wait_ui.stop().await;

    println!("{}", result?);

    Ok(())
}

fn resolve_api_key(cli_api_key: String) -> String {
    if !cli_api_key.is_empty() {
        return cli_api_key;
    }

    std::env::var("LM_API_TOKEN")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .unwrap_or_default()
}
