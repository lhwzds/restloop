---
title: Session 文件：恢复与分叉
source: #891b2f8f9b10
---

`RESTLOOP_SESSION=<path>` 会把对话以 JSON 形式持久化：

```json
{"version": 1, "messages": [{"role": "system", "content": "..."}, ...]}
```

`messages` 就是下一次请求要发送的内容，采用 OpenAI chat 格式（使用 Responses API 时，是转换之前的内容）。`conversation()` 负责加载：文件不存在时开始新对话，并在第一步之后创建文件；文件已存在时，要求 `version` 为 1 且 `messages` 数组非空，否则 RestLoop 以状态 2 退出。除此之外不做任何检查，所以可以在两次运行之间编辑文件：合法的 chat 消息都会原样发送。编辑某条 assistant 消息后，要删掉它的 `output_items`，否则发出去的仍是它们。

用同一路径再次运行即可恢复；先复制文件就是分叉。保存的系统 prompt 原样复用。设置了 `RESTLOOP_DEADLINE` 时，恢复后的任务末尾会追加 `[You have Ns in total for this request; the time used in tool results counts from now.]`。每一步之后都会通过 `<path>.<pid>.tmp` 重命名来替换文件，所以进程被 kill 后，文件里保存的是最后一个完整的步骤。并发运行不会损坏文件，但最后写入的会覆盖前面的。
