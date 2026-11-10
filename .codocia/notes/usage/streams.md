---
title: What goes to stdout and stderr
order: 30
evidence:
  - src/shell.rs:270-277 #9e16c57c3adc
  - src/lib.rs:77-81 #42989d73814e
  - src/lib.rs:115-129 #fbc16d527075
  - src/lib.rs:156-157 #517656552f29
  - src/lib.rs:164-168 #ef2b850ba233
  - src/lib.rs:446-447 #ef5644d396bb
  - src/main.rs:80-84 #9627391266f6
---

stdout carries only the model's text: every non-empty reply, trimmed, including comments written alongside tool calls. Steps have no separator; for the final answer only, use `RESTLOOP_LAST_MESSAGE`.

stderr carries everything else: each command as a dimmed `$ <command>`, then its duration and the first 10 lines of its output, retry and context-overflow notices, and with `RESTLOOP_LOG_USAGE=1` one line per step that gets a reply, from that reply's usage (retried requests are not logged):

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

Fields the provider does not report print as `null`. They come from the core's `run()`, so a library caller gets them too; only the stdin prompt `> ` is the CLI's. The default runner adds `restloop: cannot stop a process group` when a SIGKILL fails on a group that is still there.
