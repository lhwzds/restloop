---
title: Extra request headers
order: 25
evidence:
  - src/main.rs:26-37 #2164701a5216
  - src/main.rs:179-201 #28f0a5b478c6
  - src/lib.rs:46-75 #772e10a56755
  - src/lib.rs:82-92 #4adcc8ba087a
---

`RESTLOOP_HEADERS` holds extra request headers, one `Name: value` per line (blank lines skipped, CRLF fine), sent with every model request; a name given twice goes twice. OpenCode Go, for example, wants `x-opencode-session: <id>`. These names are refused, in any case: `Authorization`, `Proxy-Authorization`, `Content-Type`, `Content-Length`, `Host`, `Transfer-Encoding`, `Accept-Encoding`; any other valid name is sent. A bad line (no `:`, an invalid or refused header) or non-UTF-8 text exits with 2 first. The error gives the line number (`RESTLOOP_HEADERS: line 2 has no ':'`) and, for a refused name, the name, never the value; the UTF-8 error has no line number. Library callers set `Config.headers`, which `run()` refuses first (`restloop::bad_header()`).
