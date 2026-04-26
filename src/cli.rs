use clap::Parser;

pub const DEFAULT_BASE_URL: &str = "http://localhost:8787/v1";
pub const DEFAULT_MODEL: &str = "google/gemma-4-e2b";

#[derive(Debug, Parser)]
#[command(version, about = "A small mystical CLI for local pondering")]
pub struct Args {
    /// The thing to ponder
    #[arg(required = true, trailing_var_arg = true)]
    pub prompt: Vec<String>,

    /// OpenAI-compatible API base URL
    #[arg(long)]
    pub base_url: Option<String>,

    /// Model name to request
    #[arg(long)]
    pub model: Option<String>,

    /// API key for endpoints that require authorization
    #[arg(long)]
    pub api_key: Option<String>,

    /// Tavily API key for the web_search tool
    #[arg(long)]
    pub tavily_api_key: Option<String>,

    /// Disable the animated orb while waiting
    #[arg(long)]
    pub no_orb: bool,

    /// Disable mystical status messages while waiting
    #[arg(long)]
    pub no_mystical: bool,

    /// Stream tokens as they arrive instead of showing the wait UI
    #[arg(long)]
    pub stream: bool,

    /// Disable built-in tools for non-streaming requests
    #[arg(long)]
    pub no_tools: bool,
}

impl Args {
    pub fn prompt_text(&self) -> String {
        self.prompt.join(" ")
    }
}
