---
title: 测试覆盖了什么
source: #cb06888e2a7f
---

单元测试锁定以下行为：`too_long()` 接受记录在案的服务商措辞，并拒绝相近但不应匹配的情况（过长的 `tool_call_id`、500、网络错误）；`drop_old_output()` 不断减半，直到返回 0；`trim_thinking()` 保留最后两条回复，清空 `reasoning_content`，并且只删除 `output_items` 中的 `reasoning` 项；`truncate()` 保留开头和结尾。`shell()` 的测试在每个 CI 平台上运行：同时有两路输出时的 exit 行、超时 kill、后台的 `sleep 30 &` 不会阻塞。

有一个端到端测试让 `run()` 对接 `serve()`，一个回放预设回复的本地服务器：一次工具调用、一次溢出的 400 及其重试、恢复 session、对没有 `choices` 的 200 的重试。然后是 Responses API：key 文件、按顺序发回的输出项（只有 `message` 和 `function_call` 项去掉 `id`）、对服务器错误和套餐暂时不可用的重试、溢出恢复，以及在套餐用尽、内容过滤、无效请求或 key 文件无法读取时立即失败。设置 deadline 时：服务器在 200 头之前或之后挂起、持续返回 500，都会提前以 `deadline of` 结束；400 照常报告；已过 deadline 则不发出任何请求。只有它会涉及 `RESTLOOP_*` 变量，并先清掉这些变量以及所有 `*_PROXY`。
