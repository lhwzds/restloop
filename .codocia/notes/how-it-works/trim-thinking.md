---
title: Emptying old reasoning
order: 40
evidence:
  - src/lib.rs:217-246 #4b7800512438
  - commit:70f40924
  - commit:f09603f4
---

`trim_thinking()` empties the reasoning of every assistant message except the last two: `reasoning_content` becomes `""`, `reasoning` items are removed from `output_items` (message items keep their phase), and a leading `<think>` block is cut from `content`. The key stays, empty (`70f40924`): DeepSeek's thinking mode rejects (HTTP 400) a tool-turn assistant message without it. Already emptied messages do not count as changed, so retries terminate.
