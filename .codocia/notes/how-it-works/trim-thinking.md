---
title: Emptying old reasoning
order: 40
evidence:
  - src/main.rs:235-266 #e5ff8fe6f9c7
  - commit:70f40924
  - commit:f09603f4
---

`trim_thinking()` empties the reasoning of every assistant message except the last two: `reasoning_content` becomes `""`, `reasoning` items are removed from `output_items` (message items keep their phase), and any text up to `</think>` is cut from `content`. Since `70f40924` the key is kept with an empty value rather than deleted, since DeepSeek's thinking mode rejects (HTTP 400) a tool-turn assistant message missing it. Already emptied messages do not count as changed, so retries terminate.
