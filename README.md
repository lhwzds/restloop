# RestFlow

A minimal agent loop in about 300 lines of Rust, packaged as a ~1.5 MB binary.

RestFlow is a personal open-source project. It gives a model one tool, `shell`,
and runs it in a loop until the model stops calling the tool. The model reads,
writes and edits files with shell commands, the way mini-swe-agent does.

| | |
|---|---|
| Source | `src/main.rs`, about 300 lines |
| Dependencies | `ureq` (HTTPS), `serde_json` |
| Binary (release) | macOS arm64 1.3 MB, Linux x86_64 1.7 MB, Windows x86_64 1.5 MB |
| Storage | none: no database, no config file, no session files |

## Install

Download a binary from [Releases](https://github.com/lhwzds/restflow/releases), or:

```bash
cargo install --git https://github.com/lhwzds/restflow
```

## Use

RestFlow talks to any OpenAI-compatible `/chat/completions` endpoint
(OpenAI, Anthropic's compatibility endpoint, OpenRouter, GLM, DeepSeek, MiniMax, Ollama, ...).

```bash
export RESTFLOW_BASE_URL=https://api.openai.com/v1   # default
export RESTFLOW_API_KEY=...
export RESTFLOW_MODEL=...
export RESTFLOW_TIMEOUT=120                          # seconds per command, optional
export RESTFLOW_MAX_STEPS=100                        # model calls per task, optional
export RESTFLOW_LOG_USAGE=1                          # token usage per request on stderr, optional
export RESTFLOW_DEADLINE=1800                        # seconds for the whole task, optional; the model sees the time used

restflow "fix the failing test in this repo"   # one task, then exit
restflow                                       # read tasks from stdin, keeping the conversation
```

The assistant's text goes to stdout; commands and their output go to stderr.

## How it works

1. Send the conversation and the `shell` tool to the model.
2. Run each requested command with no stdin, in a fresh shell in the current
   directory: `sh` on macOS/Linux; on Windows Git Bash when it is installed,
   otherwise PowerShell. A command is killed, with everything it started, after
   120 s (`RESTFLOW_TIMEOUT` changes this, e.g. for long builds). Send back the
   exit code and output (truncated to 10,000 characters).
3. Repeat until the model replies without a tool call, or after 100 steps
   (`RESTFLOW_MAX_STEPS`).

Requests that fail with 429, 5xx or a network error are retried up to 5 times.

There is no sandbox and no confirmation: commands run with your permissions in
the current directory. Run it in a container or a throwaway checkout when that matters.

## History

Versions up to 0.5 were a much larger agent CLI with a TUI, a daemon and
executable skills. They remain available under the `v0.5.0` tag and earlier.

## License

Apache-2.0
