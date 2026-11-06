---
title: What commands inherit
order: 25
evidence:
  - src/shell.rs:29-42 #dcb06b602531
  - src/shell.rs:130-135 #b5d0e47ff9c2
  - commit:8d5e3a78
---

Restloop's own settings are no longer passed in commands' environment: every `RESTLOOP_*` name is removed before a command starts (`restloop_var`), so the key, the key file and the URL do not reach it; on Windows the match ignores case. Commands inherit everything else, including any copy of the key under another name. A command running as the same user can still reach restloop's environment (`/proc/$PPID/environ`, `ps -E`) or read the key file if it knows the path; a sandbox or another user is the real fix.
