# ponder

A small mystical CLI that sends a prompt to an OpenAI-compatible chat completions endpoint.

## Defaults

- Base URL: `http://192.168.1.40:8787/v1`
- Model: `google/gemma-4-e2b`
- API key: read from `LM_API_TOKEN`, then `OPENAI_API_KEY`, unless `--api-key` is passed

## Usage

```sh
ponder "what should I build first?"
```

With an explicit token:

```sh
ponder --api-key "$LM_API_TOKEN" "say hi in five words"
```

With another endpoint/model:

```sh
ponder --base-url http://localhost:1234/v1 --model local-model "explain SQLite"
```

The orb animation appears only in an interactive terminal. Non-interactive output stays plain.
