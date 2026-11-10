---
title: The final-reply file
order: 40
evidence:
  - src/lib.rs:98-102 #73ad085257df
  - src/lib.rs:142-150 #aee3445253b7
  - src/lib.rs:286-294 #ddbbdee7cbd4
  - commit:e8312d19
  - src/main.rs:26-37 #2164701a5216
  - tests/cli.rs:81-120 #7b847e3afae2
---

With `RESTLOOP_LAST_MESSAGE=<path>`, each task first deletes that file (unless RestLoop exited with status 2 first), then, when the model replies without a tool call, writes the reply text to it with a trailing newline. A leading `<think>` block (inline reasoning) is cut off up to the first `</think>` (unclosed, nothing is left); any other reply is written whole, even if it quotes `</think>`. Surrounding whitespace is trimmed either way. The delete is best-effort, so a run that ends early (step limit, API error, kill) usually leaves no file rather than the previous answer; to be sure, pass a fresh path each run.
