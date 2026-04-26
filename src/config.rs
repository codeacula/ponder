use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::cli::{Args, DEFAULT_BASE_URL, DEFAULT_MODEL};

#[derive(Debug)]
pub struct Settings {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
    pub tavily_api_key: Option<String>,
    pub show_mystical: bool,
    pub stream: bool,
    pub tools: bool,
}

impl Settings {
    pub fn load(args: &Args) -> Result<Self> {
        let config = FileConfig::load()?;
        let api_key = args
            .api_key
            .clone()
            .or(config.api_key)
            .or_else(env_api_key)
            .unwrap_or_default();
        let tavily_api_key = args
            .tavily_api_key
            .clone()
            .or(config.tavily_api_key)
            .or_else(env_tavily_api_key);

        Ok(Self {
            base_url: args
                .base_url
                .clone()
                .or(config.base_url)
                .unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
            model: args
                .model
                .clone()
                .or(config.model)
                .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            api_key,
            tavily_api_key,
            show_mystical: config
                .ui
                .as_ref()
                .and_then(|ui| ui.mystical_messages)
                .unwrap_or(true)
                && !args.no_mystical,
            stream: args.stream,
            tools: !args.no_tools,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
struct FileConfig {
    base_url: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    tavily_api_key: Option<String>,
    ui: Option<UiConfig>,
}

impl FileConfig {
    fn load() -> Result<Self> {
        let Some(path) = config_path() else {
            return Ok(Self::default());
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config file at {}", path.display()))?;

        toml::from_str(&text)
            .with_context(|| format!("failed to parse config file at {}", path.display()))
    }
}

#[derive(Debug, Deserialize)]
struct UiConfig {
    mystical_messages: Option<bool>,
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("ponder").join("config.toml"))
}

fn env_api_key() -> Option<String> {
    std::env::var("LM_API_TOKEN")
        .or_else(|_| std::env::var("OPENAI_API_KEY"))
        .ok()
        .filter(|key| !key.is_empty())
}

fn env_tavily_api_key() -> Option<String> {
    std::env::var("TAVILY_API_KEY")
        .ok()
        .filter(|key| !key.is_empty())
}
