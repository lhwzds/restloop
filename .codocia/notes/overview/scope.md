---
title: What it deliberately leaves out
order: 10
evidence:
  - src/main.rs:82-99 #f8741fcab940
  - src/main.rs:126-131 #e6a6524cff9c
  - commit:3b27682c
---

- **One tool.** The model reads, writes and edits files through `shell`; there is no file-edit, search or patch tool. Its only parameter is `command`.
- **No model-facing knobs.** The system prompt from `system_prompt()` and the tool schema are fixed. Commit `3b27682c` records why the command timeout is not a tool parameter: in A/B runs a model-adjustable timeout roughly doubled the number of steps.
- **No configuration file and no database.** All options are environment variables read at the point of use; RestLoop itself writes only the optional session and last-message files (see [[usage]]).
- **No sandbox and no confirmation.** Commands run with your permissions in the current directory. Isolation is the caller's job: a container or a throwaway checkout.
