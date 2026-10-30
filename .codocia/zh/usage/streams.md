---
title: stdout 和 stderr 各输出什么
source: #0df42aaf2426
---

stdout 只输出模型的文本：每条非空回复都会去掉首尾空白后打印，包括模型在调用工具时顺带写的简短说明。步骤之间没有分隔符，所以只需要最终答案的脚本应该用 `RESTLOOP_LAST_MESSAGE`。

其余内容都走 stderr：每条命令以暗色的 `$ <command>` 显示，接着是耗时和输出的前 10 行；还有重试和上下文溢出的提示；设置 `RESTLOOP_LOG_USAGE=1` 时，每一步拿到回复后输出一行，取自该回复的 usage（重试的请求不单独记录）：

```
usage: step=3 prompt_tokens=8123 completion_tokens=412 cached_tokens=7936 elapsed=41s
```

服务商没有返回的字段打印为 `null`。
