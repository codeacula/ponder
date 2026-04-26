use anyhow::{Context, Result, anyhow};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub fn definitions() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "current_time",
                "description": "Get the current local date and time for the machine running ponder.",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "web_search",
                "description": "Search the web for current or external information. Returns result titles, URLs, and snippets.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The web search query."
                        },
                        "max_results": {
                            "type": "integer",
                            "description": "Maximum number of results to return, from 1 to 5.",
                            "minimum": 1,
                            "maximum": 5
                        }
                    },
                    "required": ["query"],
                    "additionalProperties": false
                }
            }
        }),
    ]
}

pub async fn execute(
    http: &reqwest::Client,
    tavily_api_key: Option<&str>,
    name: &str,
    arguments: &str,
) -> Result<String> {
    match name {
        "current_time" => current_time(arguments),
        "web_search" => web_search(http, tavily_api_key, arguments).await,
        _ => Err(anyhow!("unknown tool requested: {name}")),
    }
}

fn current_time(_arguments: &str) -> Result<String> {
    let now = Local::now();
    Ok(json!({
        "local_time": now.to_rfc3339(),
        "timezone": now.format("%Z").to_string(),
        "unix_seconds": now.timestamp()
    })
    .to_string())
}

async fn web_search(
    http: &reqwest::Client,
    tavily_api_key: Option<&str>,
    arguments: &str,
) -> Result<String> {
    let api_key = tavily_api_key.ok_or_else(|| {
        anyhow!("web_search requires a Tavily API key. Set TAVILY_API_KEY or tavily_api_key in ~/.config/ponder/config.toml")
    })?;
    let args: WebSearchArgs = serde_json::from_str(arguments)
        .with_context(|| format!("invalid web_search arguments: {arguments}"))?;
    let query = args.query.trim();

    if query.is_empty() {
        return Err(anyhow!("web_search query cannot be empty"));
    }

    let max_results = args.max_results.unwrap_or(5).clamp(1, 5);

    let response: TavilyResponse = http
        .post("https://api.tavily.com/search")
        .json(&TavilyRequest {
            api_key,
            query,
            search_depth: "basic",
            max_results,
            include_answer: false,
            include_raw_content: false,
        })
        .send()
        .await
        .context("failed to reach Tavily search")?
        .error_for_status()
        .context("Tavily search returned an error")?
        .json()
        .await
        .context("failed to read Tavily search response")?;

    if response.results.is_empty() {
        return Err(anyhow!("web_search did not find any results"));
    }

    Ok(json!({
        "query": query,
        "results": response.results,
    })
    .to_string())
}

#[derive(Deserialize)]
struct WebSearchArgs {
    query: String,
    max_results: Option<usize>,
}

#[derive(Serialize)]
struct TavilyRequest<'a> {
    api_key: &'a str,
    query: &'a str,
    search_depth: &'a str,
    max_results: usize,
    include_answer: bool,
    include_raw_content: bool,
}

#[derive(Deserialize)]
struct TavilyResponse {
    results: Vec<TavilyResult>,
}

#[derive(Deserialize, Serialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: Option<f64>,
}
