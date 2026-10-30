---
title: 测试覆盖了什么
source: #e2ecd04f60be
---

单元测试验证并防止其他页面所述行为回归：`too_long()` 接受记录在案的服务商措辞，并拒绝相近但不应匹配的情况（过长的 `tool_call_id`、500、网络错误）；`drop_old_output()` 把剩余输出不断减半，直到返回 0；`trim_thinking()` 保留最后两条回复，清空（而非删除）`reasoning_content`，并且只删除 `output_items` 中的 `reasoning` 项；`truncate()` 保留开头和结尾。`shell()` 的测试在每个 CI 平台上运行：同时有两路输出时的 exit 行、超时时的 kill，以及后台的 `sleep 30 &` 不会让命令一直挂着。

有一个端到端测试让 `run()` 对接一个回放预设回复的本地服务器（`serve()`）：一次工具调用、一次上下文溢出的 400 及其重试、从保存的 session 恢复、对没有 `choices` 的 200 的重试。然后切换到 Responses API：key 文件、按顺序发回的输出项（去掉 `message` 和 `function_call` 项的 `id` 字段，保留 reasoning 的 id 和 `call_id`）、对服务器错误和套餐暂时不可用的重试、溢出恢复，以及在套餐用尽、内容过滤、无效请求或 key 文件无法读取时立即失败。只有这个测试会涉及 `RESTLOOP_*` 变量；它会先清掉这些变量以及所有 `*_PROXY`。
