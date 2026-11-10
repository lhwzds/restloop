---
title: Session files: resume and fork
order: 50
evidence:
  - src/main.rs:54-64 #10f5eabcd7a4
  - src/main.rs:130-153 #d7c22a10b96d
  - src/lib.rs:188-202 #60540309b16c
  - README.md:84-102 #a3c876cccc7a
  - commit:ffd11c37
  - commit:4c468f27
---

`RESTLOOP_SESSION=<path>` persists the conversation as JSON:

```json
{"version": 1, "messages": [{"role": "system", "content": "..."}, ...]}
```

`messages` is what the next request sends, in OpenAI chat format (before any Responses conversion). `conversation()` loads it: a missing file starts a new conversation, created after the first step; an existing file needs `version` 1 and a non-empty `messages` array, or RestLoop exits with status 2. Nothing else is checked, so the file may be edited between runs. After editing an assistant message, remove its `output_items`, which would otherwise be sent instead.

Run again with the same path to resume; copy the file first to fork. The saved system prompt is reused. With `RESTLOOP_DEADLINE`, a resumed task gets `[You have Ns in total for this request; the time used in tool results counts from now.]` appended. The file is replaced after every step through a `<path>.<pid>.tmp` rename, so after a kill it holds the last complete step; concurrent runs never corrupt it, the last writer wins.
