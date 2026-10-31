---
title: Requests and retries
order: 20
evidence:
  - src/main.rs:313-422 #e39c1c15c1e9
  - commit:333b7b64
---

`complete()` posts `{"model", "messages", "tools"}` to `/chat/completions`, plus `reasoning_effort` if set, without sampling parameters, token limit or streaming. The key is reread per attempt ([[usage]]); an unreadable key file fails with `cannot read <path>`. An attempt may take min(600 s, time left).

Network errors (unreadable bodies included), HTTP 429, 5xx and a 200 without `choices[0].message` (some MiniMax errors, `333b7b64`) are retried: five attempts, sleeping 2, 4, 8, 16 s, printing `restloop: <what>, retrying`. A body with `insufficient_quota` or `subscription_sharing_usage_limit_exceeded` fails at once. Other statuses fail the task at once with `<what>: <text>`, `<what>` being `HTTP <status>`, `request failed` or `no reply`; so does the fifth failure (once the deadline is reached, the deadline form below). A 200 whose body is not JSON fails at once with `bad response`.
