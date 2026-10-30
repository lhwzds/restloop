---
title: Requests and retries
order: 20
evidence:
  - src/main.rs:313-399 #d0cc03b4bdc1
  - commit:333b7b64
---

`complete()` posts `{"model", "messages", "tools"}` to `/chat/completions`, plus `reasoning_effort` if set, with no sampling parameters, token limit or streaming. The key is read before every attempt (see [[usage]]); an unreadable key file fails the task with `cannot read <path>`. Each attempt may take up to 600 s.

Network errors, HTTP 429, HTTP 5xx and an HTTP 200 without `choices[0].message` (how MiniMax reports some errors, `333b7b64`) are retried, five attempts in all, sleeping 2, 4, 8 and 16 s and printing `restloop: <what>, retrying`. A body naming `insufficient_quota` or `subscription_sharing_usage_limit_exceeded` (credits or plan used up) fails at once. Any other status, or the fifth failure, becomes `HTTP <status>: <body>` (or `request failed: ...`, `no reply: ...`) and fails the task (exit codes in [[usage]]). A 200 whose body is not JSON fails at once with `bad response`.
