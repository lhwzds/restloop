---
title: Recovering from a full context window
order: 30
evidence:
  - src/main.rs:141-150 #970aaf8029af
  - src/main.rs:268-311 #e157a484a14a
  - commit:21f3c1dc
  - commit:04004c68
---

Since `21f3c1dc`, `too_long()` recognises a context overflow: an HTTP 400 or 413, or a Responses stream failure (`no reply: `), whose text contains a provider phrase (`context_length_exceeded`, "maximum context length", "context window exceeds", "prompt is too long", "exceeds the available context size"). `04004c68` dropped looser words like "too long" after an overlong `tool_call_id` triggered history cuts.

On a match, `drop_old_output()` cuts the older half (rounded up) of the `tool` messages not yet shortened to their first line plus `[output removed to fit the context window]`, empties old reasoning with `trim_thinking()`, and the request is retried. Each retry halves what is left; when nothing changes, the error is returned. Commands and visible replies stay, so the model still knows what it did.
