---
title: What the conversation holds
order: 10
evidence:
  - src/lib.rs:204-215 #41769e581b81
  - src/lib.rs:125-141 #07c5f773336d
  - src/lib.rs:152-155 #515c5a424a88
  - src/lib.rs:180-181 #e0b6ca9cbfb5
---

The system prompt (`system_prompt()`) comes first: instructions plus OS, shell, working directory, command timeout and, with a deadline, the budget. Then come user tasks and, per model reply, an assistant message keeping only `content`, `tool_calls` and, when present, `reasoning_content` and the Responses API's `output_items`.

Each tool call gets one `tool` message with the matching `tool_call_id`.
