use clap::Parser;

pub const DEFAULT_BASE_URL: &str = "http://192.168.1.40:8787/v1";
pub const DEFAULT_MODEL: &str = "google/gemma-4-e2b";
pub const DEFAULT_API_KEY: &str = "";

#[derive(Debug, Parser)]
#[command(version, about = "A small mystical CLI for local pondering")]
pub struct Args {
    /// The thing to ponder
    #[arg(required = true, trailing_var_arg = true)]
    pub prompt: Vec<String>,

    /// OpenAI-compatible API base URL
    #[arg(long, default_value = DEFAULT_BASE_URL)]
    pub base_url: String,

    /// Model name to request
    #[arg(long, default_value = DEFAULT_MODEL)]
    pub model: String,

    /// API key for endpoints that require authorization
    #[arg(long, default_value = DEFAULT_API_KEY)]
    pub api_key: String,

    /// Disable the animated orb while waiting
    #[arg(long)]
    pub no_orb: bool,

    /// Disable mystical status messages while waiting
    #[arg(long)]
    pub no_mystical: bool,
}

impl Args {
    pub fn prompt_text(&self) -> String {
        self.prompt.join(" ")
    }
}
