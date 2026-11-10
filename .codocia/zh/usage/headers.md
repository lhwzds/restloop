---
title: 额外的请求头
source: #1582af3a253c
---

`RESTLOOP_HEADERS` 存放额外的请求头，每行一个 `Name: value`（跳过空行，CRLF 也可以），随每个模型请求一起发送；同一个名字写两次就发两次。例如 OpenCode Go 需要 `x-opencode-session: <id>`。以下名字不论大小写都会被拒绝：`Authorization`、`Proxy-Authorization`、`Content-Type`、`Content-Length`、`Host`、`Transfer-Encoding`、`Accept-Encoding`；其他合法的名字都会发送。某一行不合法（没有 `:`、请求头无效或被拒绝）或文本不是 UTF-8 时，程序在一开始就以 2 退出。错误信息给出行号（`RESTLOOP_HEADERS: line 2 has no ':'`），名字被拒绝时还给出该名字，但从不包含值；UTF-8 错误没有行号。作为库调用时设置 `Config.headers`，由 `run()` 先拒绝（`restloop::bad_header()`）。
