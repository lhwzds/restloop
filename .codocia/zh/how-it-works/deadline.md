---
title: 时间预算
source: #98f209351810
---

`RESTLOOP_DEADLINE`（`f9d3b4f0`：Terminal-Bench 任务曾在不知情时耗尽时间）从以下几方面约束任务：

1. 新对话的系统 prompt 末尾加上 `You have Ns in total for this task; each tool result ends with the time used so far.`；恢复的 session 则在新任务中附带一句类似的话（[[usage]]）。
2. 每条命令的工具结果末尾加上 `[time used: Ns of Ds]`。
3. 命令的超时取 `RESTLOOP_TIMEOUT` 和剩余时间中较小的那个（剩余时间下限为 1 s）。
4. 模型请求的超时是读取 key 文件之后的剩余时间（最多 600 s），deadline 之后不再发出请求：`restloop: deadline of <N>s reached`。重试前的等待会到达 deadline 时，以 `deadline of <N>s reached (<what>: <text>)` 结束；第五次尝试失败（没有等待）时，只有 deadline 已经过去才这样报告（`a4ba4994`）。

时间从进程启动开始计算：在 stdin 模式下，同一个预算覆盖所有任务。
