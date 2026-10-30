---
title: The time budget
order: 50
evidence:
  - src/main.rs:86-98 #d6d7e3cb4087
  - src/main.rs:101-111 #2e145d43c30d
  - src/main.rs:195-206 #7b0f13bf0b23
  - commit:f9d3b4f0
---

`RESTLOOP_DEADLINE` (`f9d3b4f0`, after Terminal-Bench tasks spent their limit on installs and sweeps unaware of it) does three things:

1. A new conversation's system prompt ends with `You have Ns in total for this task; each tool result ends with the time used so far.` A resumed session's new task carries a similar sentence (see [[usage]]).
2. Each command's tool result ends with `[time used: Ns of Ds]`.
3. Each command's timeout becomes the smaller of `RESTLOOP_TIMEOUT` and the time left; only the time left is floored at 1 s.

RestLoop never stops the model at the deadline; the caller does. Time counts from process start, so in stdin mode the budget covers all tasks.
