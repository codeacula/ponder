mod cli;
mod client;
mod config;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use client::ChatClient;
use config::Settings;
use std::io::stdout;
use ui::WaitUi;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let prompt = args.prompt_text();
    let settings = Settings::load(&args)?;
    let client = ChatClient::new(settings.base_url, settings.api_key);

    if settings.stream {
        client
            .stream_ponder(&settings.model, &prompt, stdout())
            .await?;
        println!();
        return Ok(());
    }

    let wait_ui = WaitUi::start(settings.show_orb, settings.show_mystical);
    let result = client.ponder(&settings.model, &prompt).await;
    wait_ui.stop().await;

    println!("{}", result?);

    Ok(())
}
