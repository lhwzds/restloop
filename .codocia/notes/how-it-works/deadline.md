---
title: The time budget
order: 50
evidence:
  - src/lib.rs:207-215 #fbcf814957a0
  - src/main.rs:55-63 #02e9290cdf22
  - src/lib.rs:158-163 #7bdd6b0a9261
  - src/lib.rs:169-174 #4758f987fd4d
  - src/lib.rs:340-364 #524655f60319
  - src/lib.rs:430-442 #a50f37f2e39f
  - src/shell.rs:102-107 #2fb5b60766c7
  - commit:f9d3b4f0
  - commit:a4ba4994
---

`RESTLOOP_DEADLINE` (`f9d3b4f0`: Terminal-Bench tasks ran out of time unaware) bounds the task:

1. A new conversation's system prompt ends with `You have Ns in total for this task; each tool result ends with the time used so far.`; a resumed session's new task, a similar sentence ([[usage]]).
2. Each command's tool result ends with `[time used: Ns of Ds]`.
3. A command's timeout is the smaller of `RESTLOOP_TIMEOUT` and the time left (floored at 1 s); the CLI starts none after it.
4. A model request gets the time left after the key is read (600 s at most); none starts after it: `restloop: deadline of <N>s reached`. A retry sleep reaching it stops with `deadline of <N>s reached (<what>: <text>)`; a failed fifth attempt only if the deadline has passed (`a4ba4994`).

Time counts from process start; one budget covers all stdin tasks.
