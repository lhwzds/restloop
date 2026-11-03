---
title: The final-reply file
order: 40
evidence:
  - src/main.rs:131-136 #5c293202528a
  - src/main.rs:175-183 #959377ec60e7
  - src/main.rs:300-308 #8955fc804950
  - commit:e8312d19
---

With `RESTLOOP_LAST_MESSAGE=<path>`, each task first deletes that file, then, when the model replies without a tool call, writes the reply text to it with a trailing newline. For models that inline their reasoning, a leading `<think>` block is cut off up to the first `</think>` (unclosed, nothing is left); any other reply is written whole, even if it quotes `</think>`. Surrounding whitespace is trimmed either way. The delete is best-effort (errors are ignored), so a run that ends early (step limit, API error, kill) usually leaves no file rather than the previous answer; for certainty, pass a fresh path on every run.
