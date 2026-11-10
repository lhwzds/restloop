---
title: 测试覆盖了什么
source: #fff974e214a7
---

`src/tests.rs` 中的单元测试锁定：`too_long()` 匹配记录在案的服务商措辞，而不匹配相近的说法；`drop_old_output()` 不断减半，直到返回 0；`trim_thinking()` 保留最后两条回复；`truncate()` 保留开头和结尾。`shell()`：同时有两路输出时的 exit 行、超时 kill、后台的 `sleep 30 &` 不会阻塞。

有一个端到端测试让 `run()` 对接 `serve()`，一个回放预设回复的服务器：一次工具调用、溢出恢复、恢复 session、对空回复或没有 `choices` 的回复的重试；按顺序发回的 Responses 输出项、可重试和致命的失败；设置 deadline 时，挂起、500 和空回复都会提前以 `deadline of` 结束。命令拿不到 restloop 的任何设置。用到环境的测试共用一把锁并恢复环境；发请求的测试会清掉 `*_PROXY`。其他核心测试检查：已过 deadline 时不运行任何命令；key 失败或无法放进请求头时结束运行；额外的请求头能送达，重复的名字送两次；配置中不合法的请求头会先让运行停下；不跟随重定向。`src/main.rs` 测试 `config()`、请求头解析、key 文件和 session 文件；`tests/cleanup.rs`（Unix）检查没有进程活过一次运行；`tests/cli.rs`（所有平台）检查不合法的 `RESTLOOP_HEADERS` 时的退出（[[usage]]）。
