---
title: 模型从命令中看到什么
source: #c638f833c11c
---

命令的工具结果第一行是 `exit: <status>`，后面是合并后的输出。有两种例外：参数无效时返回 `error: arguments must be JSON…`，shell 无法启动时返回 `error: <reason>`。status 是退出码；如果 shell 被信号终止则为 `killed`，超时则为 `timeout after Ns`。超过 10,000 个字符（`MAX_OUTPUT`）的输出只保留开头和结尾各 5,000 个字符，中间插入 `... [N chars truncated] ...`。无效的 UTF-8 会被替换。
