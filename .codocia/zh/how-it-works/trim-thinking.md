---
title: 清空旧的推理内容
source: #152d88a8409b
---

`trim_thinking()` 会清空除最后两条之外所有 assistant 消息的推理内容：`reasoning_content` 变为 `""`，`output_items` 中的 `reasoning` 项被删除（message 项保留其 phase），`content` 中 `</think>` 及其之前的文本被截掉。自 `70f40924` 起，这个 key 会保留并置为空值，而不是删除，因为 DeepSeek 的思考模式在工具轮次中的 assistant 消息缺少该字段时会返回 HTTP 400。已经清空过的消息不算作有改动，因此反复的溢出重试一定会终止。
