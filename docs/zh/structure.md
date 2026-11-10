---
title: 代码结构
---

# 代码结构

<sub>[English](../structure.md) · **中文**</sub>

本页由 Codocia 根据代码自动生成，每次 build 重新计算：功能区划分、各区自带的说明、区之间的 import 依赖，以及讲解各区的页面。

范围内共 16 个文件，4,392 行；已排除 `Cargo.lock`、`LICENSE`、`.gitignore`

## 文件结构

**仓库根目录**

- （根目录文件） — 340 行，3 个文件。A minimal agent loop: one model, one shell tool.\
  讲解：[概览](01-overview.md) (2), [用法与配置](02-usage.md) (1), [构建、发布与贡献](04-contributing.md) (1)
- `.github/` — 201 行，2 个文件\
  讲解：[构建、发布与贡献](04-contributing.md) (3)
- `npm/` — 162 行，5 个文件。A minimal coding-agent loop: it keeps working on your task so you can rest.\
  核心文件：`bin/restloop.js` `stamp.js`\
  讲解：[概览](01-overview.md) (1), [构建、发布与贡献](04-contributing.md) (1)
- `src/` — 3,012 行，4 个文件。RestLoop's core: the agent loop and the model client.\
  核心文件：`lib.rs` `shell.rs` `tests.rs`\
  讲解：[概览](01-overview.md) (5), [用法与配置](02-usage.md) (8), [工作原理](03-how-it-works.md) (9), [构建、发布与贡献](04-contributing.md) (3)
- `tests/` — 677 行，2 个文件\
  讲解：[用法与配置](02-usage.md) (1), [构建、发布与贡献](04-contributing.md) (1)

