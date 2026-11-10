---
title: 模型从命令中看到什么
source: #9df88bd56f28
---

命令的工具结果第一行是 `exit: <status>`，后面是合并后的输出。例外：参数无效时返回 `error: arguments must be JSON…`，shell 无法启动时返回 `error: <reason>`。status 是退出码；shell 被信号终止时为 `killed`，子进程已消失且没有状态时为 `exited, status unknown`，超时则为 `timeout after Ns`；被停止的命令未见退出时会加上 `(child did not exit after SIGKILL)`。超过 10,000 个字符（`MAX_OUTPUT`）的输出保留开头和结尾各 5,000 个字符，中间插入 `... [N chars truncated] ...`。超过 1 MiB 的输出会先被丢弃；无效的 UTF-8 会被替换。
