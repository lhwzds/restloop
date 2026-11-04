---
title: Requests and retries
order: 20
evidence:
  - src/main.rs:314-423 #601ebd333a85
  - commit:333b7b64
---

`complete()` posts `{"model", "messages", "tools"}` to `/chat/completions`, plus `reasoning_effort` if set, without sampling parameters, token limit or streaming. The key is reread per attempt ([[usage]]); an unreadable key file fails with `cannot read <path>`.

Network errors (unreadable bodies included), HTTP 429, 5xx, a 200 without `choices[0].message` (some MiniMax errors, `333b7b64`) and an empty reply (no tool call, no text outside a leading `<think>`) are retried: five attempts, sleeping 2, 4, 8, 16 s, printing `restloop: <what>, retrying`; a fifth empty reply is the final one. A body with `insufficient_quota` or `subscription_sharing_usage_limit_exceeded` fails at once, unless the reply is empty. Other statuses, and the fifth failure, fail the task with `<what>: <text>` (`HTTP <status>`, `request failed` or `no reply`), or the deadline form below. A non-JSON 200 fails at once with `bad response`.
