---
title: 测试覆盖了什么
source: #a790ce590003
---

`src/tests.rs` 中的单元测试锁定以下行为：`too_long()` 接受记录在案的服务商措辞，并拒绝相近但不应匹配的情况；`drop_old_output()` 不断减半，直到返回 0；`trim_thinking()` 保留最后两条回复，清空 `reasoning_content`，并且只删除 `output_items` 中的 `reasoning` 项；`truncate()` 保留开头和结尾。`shell()` 的测试：同时有两路输出时的 exit 行、超时 kill、后台的 `sleep 30 &` 不会阻塞。

有一个端到端测试让 `run()` 对接 `serve()`，一个回放预设回复的本地服务器：一次工具调用、溢出恢复、恢复 session、对空回复或没有 `choices` 的回复的重试；Responses API 下，按顺序发回的输出项、可重试和立即失败的错误；设置 deadline 时，挂起、500 和空回复都会提前以 `deadline of` 结束。只有它会涉及 `RESTLOOP_*` 变量，先清掉这些变量以及 `*_PROXY`，并检查命令拿不到其中任何一个。其他核心测试用一个记录命令的工具代替 shell，检查已过 deadline 时不运行任何命令、key 失败时立即结束运行。CLI 的测试在 `src/main.rs`，覆盖 `config()`、key 文件和 session 文件。
