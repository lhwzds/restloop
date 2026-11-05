---
title: What goes to stdout and stderr
order: 30
evidence:
  - src/lib.rs:48-52 #42989d73814e
  - src/lib.rs:80-94 #fbc16d527075
  - src/lib.rs:121-122 #517656552f29
  - src/lib.rs:129-133 #ef2b850ba233
  - src/lib.rs:396-397 #ef5644d396bb
  - src/main.rs:60-64 #9627391266f6
---

stdout carries only the model's text: every non-empty reply is printed, trimmed, including the short comments a model writes alongside tool calls. There is no separator between steps, so a script that needs only the final answer should use `RESTLOOP_LAST_MESSAGE`.

stderr carries everything else: each command as a dimmed `$ <command>`, then its duration and the first 10 lines of its output, retry and context-overflow notices, and with `RESTLOOP_LOG_USAGE=1` one line per step that gets a reply, from that reply's usage (retried requests are not logged):

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

Fields the provider does not report print as `null`. These lines come from the core's `run()`, not the CLI, so a program using it as a library gets them on its stderr too: commands and results, retries, the overflow notice, and session or last-message save errors. Only the stdin prompt `> ` is the CLI's.
