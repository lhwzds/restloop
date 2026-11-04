---
title: The time budget
order: 50
evidence:
  - src/main.rs:84-96 #d6d7e3cb4087
  - src/main.rs:99-109 #2e145d43c30d
  - src/main.rs:192-203 #7b0f13bf0b23
  - src/main.rs:348-356 #04e8e7844c5b
  - src/main.rs:402-415 #884710d6423d
  - commit:f9d3b4f0
  - commit:a4ba4994
---

`RESTLOOP_DEADLINE` (`f9d3b4f0`: Terminal-Bench tasks ran out of time unaware) bounds the task:

1. A new conversation's system prompt ends with `You have Ns in total for this task; each tool result ends with the time used so far.`; a resumed session's new task, a similar sentence ([[usage]]).
2. Each command's tool result ends with `[time used: Ns of Ds]`.
3. A command's timeout is the smaller of `RESTLOOP_TIMEOUT` and the time left (floored at 1 s).
4. A model request gets the time left after the key file is read (600 s at most); none starts after it: `restloop: deadline of <N>s reached`. A retry sleep reaching it stops with `deadline of <N>s reached (<what>: <text>)`; a failed fifth attempt only if the deadline has passed (`a4ba4994`).

Time counts from process start: in stdin mode one budget covers all tasks.
