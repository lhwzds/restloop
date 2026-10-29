# RestLoop

A minimal coding-agent loop: it keeps working on your task so you can rest.
Under 500 lines of Rust, packaged as a ~1.5 MB binary.

RestLoop is a personal open-source project. It gives a model one tool, `shell`,
and runs it in a loop until the model stops calling the tool. The model reads,
writes and edits files with shell commands, the way mini-swe-agent does.

| | |
|---|---|
| Source | `src/main.rs`, under 500 lines |
| Dependencies | `ureq` (HTTPS), `serde_json` |
| Binary (release) | macOS arm64 1.3 MB, Linux x86_64 1.9 MB (static), Windows x86_64 1.5 MB |
| Storage | none by default: no database, no config file; a session file only when `RESTLOOP_SESSION` is set |

## Install

```bash
brew install lhwzds/tap/restloop                       # macOS, Linux
npm install -g restloop                                # downloads the binary on first run
cargo install --git https://github.com/lhwzds/restloop
```

Or download a binary from [Releases](https://github.com/lhwzds/restloop/releases)
(macOS, Linux x86_64 and arm64, Windows); `SHA256SUMS` lists their checksums.

## Use

RestLoop talks to any OpenAI-compatible `/chat/completions` endpoint
(OpenAI, Anthropic's compatibility endpoint, OpenRouter, GLM, DeepSeek, MiniMax, Ollama, ...).

```bash
export RESTLOOP_BASE_URL=https://api.openai.com/v1   # default
export RESTLOOP_API_KEY=...
export RESTLOOP_MODEL=...
export RESTLOOP_TIMEOUT=120                          # seconds per command, optional
export RESTLOOP_MAX_STEPS=100                        # model calls per task, optional
export RESTLOOP_LOG_USAGE=1                          # token usage per request on stderr, optional
export RESTLOOP_DEADLINE=1800                        # seconds for the whole task, optional; the model sees the time used
export RESTLOOP_LAST_MESSAGE=last.txt                # file that receives the final reply, optional
export RESTLOOP_TRIM_THINKING_AT=60000               # optional: past this many prompt tokens, keep reasoning of the last 2 replies only
export RESTLOOP_SESSION=task.json                    # optional: continue this conversation, saved after every step

restloop "fix the failing test in this repo"   # one task, then exit
restloop                                       # read tasks from stdin, keeping the conversation
```

The assistant's text goes to stdout; commands and their output go to stderr.

When the task has a time limit, set `RESTLOOP_DEADLINE` to it. On Terminal-Bench
(30 tasks, MiniMax-M3.1-Flash) this took RestLoop from 20 to 24 solved tasks and
from 5 timeouts to 1.

`RESTLOOP_SESSION` keeps a conversation across runs. If the file exists, RestLoop
continues the conversation in it with the new task; after every step it writes
the conversation back (`{"version": 1, "messages": [...]}`, exactly as the next
request will send it). Running again with the same file resumes; copying the file
first forks. The saved system prompt is reused unchanged, so a provider's prompt
cache still matches; a resumed run's `RESTLOOP_DEADLINE` is given with the new task.
Use a session file for one run at a time.

## How it works

1. Send the conversation and the `shell` tool to the model.
2. Run each requested command with no stdin, in a fresh shell in the current
   directory: `sh` on macOS/Linux; on Windows Git Bash when it is installed,
   otherwise PowerShell. A command is killed, with everything it started, after
   120 s (`RESTLOOP_TIMEOUT` changes this, e.g. for long builds). Send back the
   exit code and output (truncated to 10,000 characters).
3. Repeat until the model replies without a tool call, or after 100 steps
   (`RESTLOOP_MAX_STEPS`).

Requests that fail with 429, 5xx or a network error are retried up to 5 times.
When the conversation no longer fits the model's context window, the older half
of the command outputs is cut down to its exit line (and old reasoning dropped)
before the request is sent again.

There is no sandbox and no confirmation: commands run with your permissions in
the current directory. Run it in a container or a throwaway checkout when that matters.

## History

RestLoop was called RestFlow up to version 0.5, a much larger agent CLI with
a TUI, a daemon and executable skills; that code is in the git history before
this rewrite (commit `6d3f49b5`). The new name keeps its "Rest": the loop does
the work, you get to rest.

## License

Apache-2.0
