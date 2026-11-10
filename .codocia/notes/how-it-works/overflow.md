---
title: Recovering from a full context window
order: 30
evidence:
  - src/lib.rs:103-114 #87067936cf17
  - src/lib.rs:248-284 #73b2c15e6565
  - commit:21f3c1dc
  - commit:04004c68
---

`too_long()` (`21f3c1dc`) recognises a context overflow: an HTTP 400 or 413, or a Responses stream failure (`no reply: `), whose text contains a provider phrase (`context_length_exceeded`, "maximum context length", "context window exceeds", "prompt is too long", "exceeds the available context size"). `04004c68` dropped looser words like "too long", which an overlong `tool_call_id` matched.

On a match, `drop_old_output()` cuts the older half (rounded up) of the `tool` messages not yet shortened to their first line plus `[output removed to fit the context window]`, empties old reasoning with `trim_thinking()`, and the request is retried. Each retry halves the rest; when nothing changes, the error is returned. Commands and visible replies stay.
