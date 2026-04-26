# ponder

A small mystical CLI that sends a prompt to an OpenAI-compatible chat completions endpoint.

## Defaults

- Base URL: `http://192.168.1.40:8787/v1`
- Model: `google/gemma-4-e2b`
- API key: read from `LM_API_TOKEN`, then `OPENAI_API_KEY`, unless `--api-key` is passed
- Built-in tools are enabled for non-streaming requests
- Web search uses Tavily via `TAVILY_API_KEY` or `tavily_api_key` in config

## Usage

```sh
ponder "what should I build first?"
```

Or start an interactive prompt to avoid shell quoting issues:

```sh
ponder
```

You can also pipe a prompt through stdin:

```sh
printf "what's the next step?" | ponder
```

Build and install the current checkout for local testing:

```sh
./scripts/install-local.sh
```

With an explicit token:

```sh
ponder --api-key "$LM_API_TOKEN" "say hi in five words"
```

With another endpoint/model:

```sh
ponder --base-url http://localhost:1234/v1 --model local-model "explain SQLite"
```

Stream tokens as they arrive:

```sh
ponder --stream "tell me a short story about an orb"
```

Disable built-in tools for non-streaming requests:

```sh
ponder --no-tools "what time is it?"
```

Ask current web-backed questions:

```sh
export TAVILY_API_KEY="..."
ponder "search the web for the latest Rust release and summarize it"
```

Mystical status messages appear only in an interactive terminal. Non-interactive output stays plain.

## Config

Optional config lives at `~/.config/ponder/config.toml`:

```toml
base_url = "http://192.168.1.40:8787/v1"
model = "google/gemma-4-e2b"
# api_key = "..."
# tavily_api_key = "..."

[ui]
mystical_messages = true
```

CLI flags override config values.
