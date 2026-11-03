---
title: What the tests cover
order: 5
evidence:
  - src/tests.rs:8-97 #622ddde327a6
  - src/tests.rs:118-173 #e1b221e5e30a
  - src/tests.rs:153-438 #5fa8b3f8ef1f
  - commit:4c468f27
  - commit:333b7b64
  - commit:a4ba4994
---

Unit tests pin: `too_long()` accepts recorded provider wordings and rejects near misses (an overlong `tool_call_id`, a 500, a network error); `drop_old_output()` halves until it returns 0; `trim_thinking()` spares the last two replies, empties `reasoning_content` and drops only `reasoning` items of `output_items`; `truncate()` keeps head and tail. `shell()` tests: the exit line with both streams, a timeout kill, a background `sleep 30 &` not blocking.

One end-to-end test drives `run()` against `serve()`, a local canned-reply server: a tool call, an overflow 400 and its retry, a session resume, retried 200s without `choices` or with an empty or reasoning-only reply. Over the Responses API: the key file, output items sent back in order (only `message` and `function_call` items lose `id`), retried server and unavailable-plan failures, overflow recovery, failing at once on a used-up plan, content filter, invalid request or unreadable key file. Under a deadline: a server that hangs before or after its 200 headers, and repeated 500s, stop early with `deadline of`; a 400 is reported as usual; a passed deadline sends nothing. It alone touches `RESTLOOP_*`, clearing them and `*_PROXY` first.
