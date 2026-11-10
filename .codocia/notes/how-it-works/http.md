---
title: Requests and retries
order: 20
evidence:
  - src/lib.rs:319-450 #ca78ec2a0ca7
  - commit:333b7b64
---

`complete()` posts `{"model", "messages", "tools"}` to `/chat/completions`, plus `reasoning_effort` if set, without sampling parameters, token limit or streaming. The key is reread per attempt; an unreadable key file fails with `cannot read <path>`, a key with an inner line break with `the key cannot go in a header`. Redirects are not followed (a 3xx fails), so the extra headers ([[usage]]) never reach another host.

Network errors, HTTP 429, 5xx, a 200 without `choices[0].message` (MiniMax errors, `333b7b64`) and an empty reply (no tool call, no text past a leading `<think>`) are retried: five attempts, sleeping 2, 4, 8, 16 s, printing `restloop: <what>, retrying`; a fifth empty reply is final. A body with `insufficient_quota` or `subscription_sharing_usage_limit_exceeded` fails at once, unless the reply is empty. Other statuses and a fifth failure fail the task: `<what>: <text>` (`HTTP <status>`, `request failed` or `no reply`), or the deadline form below. A non-JSON 200 fails at once with `bad response`.
