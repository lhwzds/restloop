---
title: What the tests cover
order: 5
evidence:
  - src/tests.rs:8-97 #306212c7f3e0
  - src/tests.rs:96-139 #7f1ea52b3fe3
  - src/tests.rs:141-370 #9feebca06bbe
  - commit:4c468f27
  - commit:333b7b64
---

Unit tests pin what other pages describe: `too_long()` accepts the recorded provider wordings and rejects near misses (an overlong `tool_call_id`, a 500, a network error); `drop_old_output()` halves the remaining outputs until it returns 0; `trim_thinking()` spares the last two replies, empties rather than removes `reasoning_content`, and drops only the `reasoning` items of `output_items`; `truncate()` keeps head and tail. The `shell()` tests run on each CI platform: the exit line with both streams, a kill at the timeout, and background `sleep 30 &` not blocking.

One end-to-end test drives `run()` against a local server (`serve()`) replaying canned replies: a tool call, a context-overflow 400 and its retry, a resume from the saved session, a retried 200 without `choices`. Then, over the Responses API: the key file, output items sent back in order (`id` fields on `message` and `function_call` items dropped, reasoning ids and `call_id` kept), retried server and unavailable-plan failures, overflow recovery, and immediate failure for a used-up plan, a content filter, an invalid request or an unreadable key file. Only this test touches `RESTLOOP_*`; it clears them and any `*_PROXY` first.
