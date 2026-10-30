---
title: 时间预算
source: #f83fa60d25fa
---

`RESTLOOP_DEADLINE`（`f9d3b4f0`，起因是 Terminal-Bench 任务在不知道时限的情况下把时间耗在了安装和扫描上）做三件事：

1. 新对话的系统 prompt 末尾加上 `You have Ns in total for this task; each tool result ends with the time used so far.`。恢复的 session 则在新任务中附带一句类似的话（见 [[usage]]）。
2. 每条命令的工具结果末尾加上 `[time used: Ns of Ds]`。
3. 每条命令的超时取 `RESTLOOP_TIMEOUT` 和剩余时间中较小的那个；只有剩余时间有 1 s 的下限。

deadline 到了，RestLoop 也不会让模型停下；由调用方负责。时间从进程启动开始计算，所以在 stdin 模式下，预算覆盖所有任务。
