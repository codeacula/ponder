mod cli;
mod client;
mod config;
mod tools;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::Args;
use client::ChatClient;
use config::Settings;
use std::io::{IsTerminal, Read, Write, stdin, stdout};
use ui::{WaitUi, print_answer};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let prompt = read_prompt(&args)?;
    let settings = Settings::load(&args)?;
    let client = ChatClient::new(settings.base_url, settings.api_key, settings.tavily_api_key);

    if settings.stream {
        client
            .stream_ponder(&settings.model, &prompt, stdout())
            .await?;
        println!();
        return Ok(());
    }

    let wait_ui = WaitUi::start(settings.show_mystical);
    let result = if settings.tools {
        client.ponder_with_tools(&settings.model, &prompt).await
    } else {
        client.ponder(&settings.model, &prompt).await
    };
    wait_ui.stop().await;

    print_answer(&result?)?;

    Ok(())
}

fn read_prompt(args: &Args) -> Result<String> {
    if let Some(prompt) = args.prompt_text() {
        return Ok(prompt);
    }

    let mut prompt = String::new();
    if stdin().is_terminal() {
        print!("ponder> ");
        stdout().flush()?;
        stdin().read_line(&mut prompt)?;
    } else {
        stdin().read_to_string(&mut prompt)?;
    }

    let prompt = prompt.trim().to_string();
    anyhow::ensure!(!prompt.is_empty(), "prompt cannot be empty");

    Ok(prompt)
}
