---
title: What the tests cover
order: 5
evidence:
  - src/tests.rs:168-311 #d214861410e3
  - src/tests.rs:30-96 #ed265fea4271
  - src/tests.rs:472-497 #5751d86ebf05
  - src/tests.rs:861-937 #6410853dd61f
  - src/tests.rs:939-1078 #80ac499d18f3
  - src/main.rs:208-597 #1773d8660216
  - tests/cleanup.rs:1-6 #6c4c6279ec93
  - tests/cleanup.rs:336-361 #bb67748bce83
  - commit:4c468f27
  - commit:333b7b64
  - commit:a4ba4994
  - tests/cli.rs:81-120 #7b847e3afae2
  - src/tests.rs:991-1126 #e34e148da697
  - src/tests.rs:1213-1245 #fd9a59acc545
---

Unit tests in `src/tests.rs` pin: `too_long()` matches recorded provider wordings, not near misses; `drop_old_output()` halves until it returns 0; `trim_thinking()` spares the last two replies; `truncate()` keeps head and tail. `shell()`: the exit line with both streams, a timeout kill, a background `sleep 30 &` not blocking.

One end-to-end test drives `run()` against `serve()`, a canned-reply server: a tool call, overflow recovery, a resume, retried empty or `choices`-less replies; Responses output items sent back in order, retried and fatal failures; under a deadline, hangs, 500s and empty replies stop early with `deadline of`. Commands get none of restloop's settings. Environment tests hold one lock and restore it; request tests clear `*_PROXY`. Other core tests check that a passed deadline runs no command, a failing or unsendable key ends the run, extra headers arrive, a repeated name twice, a bad config header stops the run first and a redirect is not followed. `src/main.rs` tests `config()`, header parsing, the key file and session files; `tests/cleanup.rs` (Unix) checks no process outlives a run; `tests/cli.rs` (all platforms) checks the exit on a bad `RESTLOOP_HEADERS` ([[usage]]).
