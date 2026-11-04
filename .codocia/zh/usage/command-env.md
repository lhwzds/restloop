---
title: 命令会继承什么
source: #332ed8397b35
---

RestLoop 自己的设置不再传入命令的环境：命令启动前会去掉所有 `RESTLOOP_*` 名称（`restloop_var`），所以 key、key 文件和 URL 都不会传给命令；在 Windows 上匹配不区分大小写。其他变量命令都会继承，包括以其他名字保存的 key 副本。以同一用户运行的命令仍然可以读到 restloop 的环境（`/proc/$PPID/environ`、`ps -E`），知道路径时也能读 key 文件；真正的解决办法是沙箱或换一个用户。
