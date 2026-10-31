---
title: Session files: resume and fork
order: 50
evidence:
  - src/main.rs:28-29 #f5d692c0b065
  - src/main.rs:63-80 #782eaa92dbb8
  - src/main.rs:220-233 #3f74ef231148
  - README.md:62-79 #13e3ca92648b
  - commit:ffd11c37
  - commit:4c468f27
  - src/main.rs:33-38 #28c49aae845e
---

`RESTLOOP_SESSION=<path>` persists the conversation as JSON:

```json
{"version": 1, "messages": [{"role": "system", "content": "..."}, ...]}
```

`messages` is exactly what the next request will send, in OpenAI chat format (with the Responses API, before conversion). `conversation()` loads it: a missing file starts a new conversation, created after the first step; an existing file needs `version` 1 and a non-empty `messages` array, or RestLoop exits with status 2. Nothing else is checked, so the file may be edited between runs: any valid chat messages are sent as they are. After editing an assistant message, remove its `output_items`, which would otherwise be sent instead.

Run again with the same path to resume; copy the file first to fork. The saved system prompt is reused unchanged. With `RESTLOOP_DEADLINE`, a resumed task gets `[You have Ns in total for this request; the time used in tool results counts from now.]` appended. The file is replaced after every step through a `<path>.<pid>.tmp` rename, so after a kill it holds the last complete step. Concurrent runs never corrupt it, but the last writer wins.
