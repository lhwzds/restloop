---
title: What the tests cover
order: 5
evidence:
  - src/tests.rs:167-297 #6e2df1e27725
  - src/tests.rs:30-96 #ed265fea4271
  - src/tests.rs:431-456 #5751d86ebf05
  - src/tests.rs:820-910 #39cb273e0db0
  - src/tests.rs:932-1055 #e08207184185
  - src/main.rs:162-465 #bac765abd21f
  - commit:4c468f27
  - commit:333b7b64
  - commit:a4ba4994
---

Unit tests in `src/tests.rs` pin: `too_long()` accepts recorded provider wordings and rejects near misses; `drop_old_output()` halves until it returns 0; `trim_thinking()` spares the last two replies, empties `reasoning_content` and drops only `reasoning` items of `output_items`; `truncate()` keeps head and tail. `shell()` tests: the exit line with both streams, a timeout kill, a background `sleep 30 &` not blocking.

One end-to-end test drives `run()` against `serve()`, a local canned-reply server: a tool call, overflow recovery, a session resume, retried empty replies or replies without `choices`; over the Responses API, output items sent back in order, retried and fatal failures; under a deadline, hangs, 500s and empty replies stop early with `deadline of`. It alone sets `RESTLOOP_*`, clears `*_PROXY` first and checks commands get none; it and a test of the proxy guard hold one environment lock, and every changed variable is put back. Other core tests use a recording tool and check that a passed deadline runs no command and a failing key ends the run at once. The CLI's tests in `src/main.rs` cover `config()`, the key file and session files.
