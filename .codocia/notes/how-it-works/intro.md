---
title: 
order: 0
evidence:
  - src/main.rs:126-131 #e6a6524cff9c
  - src/main.rs:140-150 #e9aed182c6f9
  - src/main.rs:216-218 #6ebd9760f8d2
---

Everything happens in `run()`: one iteration (a step) is one model reply plus the commands it requested.

```mermaid
flowchart TD
  A[messages + shell tool] --> B["complete(): POST /chat/completions or /responses"]
  B -->|"context overflow"| C["drop_old_output()"] --> B
  B --> D[append assistant message, print its text]
  D -->|no tool_calls| E[save session, write last message, return Ok]
  D -->|tool_calls| F["shell() each command in order"]
  F --> G[append one tool message per call; save session]
  G -->|step < RESTLOOP_MAX_STEPS| B
  G -->|limit reached| H["Err: stopped after N steps"]
```
