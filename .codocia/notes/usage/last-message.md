---
title: The final-reply file
order: 40
evidence:
  - src/main.rs:134-139 #5c293202528a
  - src/main.rs:178-187 #aedb0e299b46
  - commit:e8312d19
---

With `RESTLOOP_LAST_MESSAGE=<path>`, each task first deletes that file, then, when the model replies without a tool call, writes the reply text to it with a trailing newline. Anything up to the last `</think>` is cut off, for models that inline their reasoning. The delete is best-effort (errors are ignored), so a run that ends early (step limit, API error, kill) usually leaves no file rather than the previous answer; for certainty, pass a fresh path on every run.
