---
title: 如何发布一个版本
source: #1eb08a683e2c
---

推送 `vX.Y.Z` 形式的 tag（预发布 tag 不匹配）会运行 `.github/workflows/release.yml`：

1. **build** 编译五个目标：macOS arm64 和 x86_64、Linux x86_64 和 arm64（静态 musl）、Windows x86_64，命名为 `restloop-<target>[.exe]`。
2. **release** 写出 `SHA256SUMS` 以及每个二进制对应的一个 `.sha256`，并创建 GitHub release。如果该 release 已存在，它会拒绝运行：已发布的二进制永远不会被替换，因为 npm 包固定了它们的校验和。此时应只重跑失败的 job。
3. **homebrew** 重新生成 `lhwzds/homebrew-tap` 中的 `restloop` formula（需要 `HOMEBREW_TAP_TOKEN` secret），只有内容变化时才提交。

手动运行该 workflow（`workflow_dispatch`）会构建并计算校验和，但不发布任何东西，相当于一次演练。
