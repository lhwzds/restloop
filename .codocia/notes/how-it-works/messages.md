---
title: What the conversation holds
order: 10
evidence:
  - src/main.rs:82-99 #f8741fcab940
  - src/main.rs:161-177 #07c5f773336d
  - src/main.rs:189-214 #8c6d3e5de5e6
---

The system prompt from `system_prompt()` comes first: instructions plus OS, shell, working directory, command timeout and, with a deadline, the budget. Then come user tasks and, per model reply, an assistant message keeping only `content`, `tool_calls` and, when present, `reasoning_content` and the Responses API's `output_items`.

Each tool call gets one `tool` message with the matching `tool_call_id`. Arguments that are not JSON with a `command` string get an error quoting them, so the model can fix them.
