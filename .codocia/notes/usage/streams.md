---
title: What goes to stdout and stderr
order: 30
evidence:
  - src/main.rs:151-165 #cb51ce04a2dd
  - src/main.rs:189-214 #8c6d3e5de5e6
  - src/main.rs:144-147 #0803b9405cbf
---

stdout carries only the model's text: every non-empty reply is printed, trimmed, including the short comments a model writes alongside tool calls. There is no separator between steps, so a script that needs only the final answer should use `RESTLOOP_LAST_MESSAGE`.

stderr carries everything else: each command as a dimmed `$ <command>`, then its duration and the first 10 lines of its output, retry and context-overflow notices, and with `RESTLOOP_LOG_USAGE=1` one line per step that gets a reply, from that reply's usage (retried requests are not logged):

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

Fields the provider does not report print as `null`.
