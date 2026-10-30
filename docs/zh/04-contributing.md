---
title: 构建、发布与贡献
---

# 构建、发布与贡献

<sub>[English](../04-contributing.md) · **中文**</sub>

`cargo build --release` 生成 `target/release/restloop`。这个 crate 使用 edition 2024，并在 `if` 中使用 `let` 链（在 `shell()` 里），所以需要两者都已稳定的工具链（Rust 1.88 或更新）。release profile 针对体积优化（`opt-level = "z"`、LTO、单个 codegen unit、`panic = "abort"`、strip）。

测试位于 `src/tests.rs`，用 `cargo test` 运行；`.github/workflows/ci.yml` 会在每次 push 到 `main` 和每个 pull request 时，在 macOS、Linux 和 Windows 上运行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings` 和测试。提 pull request 之前，请先在本地跑一遍这三条命令。

<sub>来源: [Cargo.toml:1-18](../../Cargo.toml#L1-L18) · [main.rs:408-415](../../src/main.rs#L408-L415) · [ci.yml:1-25](../../.github/workflows/ci.yml#L1-L25)</sub>

## 测试覆盖了什么

单元测试验证并防止其他页面描述的行为回归：`too_long()` 接受记录在案的服务商措辞，并拒绝相近但不应匹配的情况（过长的 `tool_call_id`、500、网络错误）；`drop_old_output()` 把剩余输出不断减半，直到返回 0；`trim_thinking()` 保留最后两条回复，并清空（而非删除）`reasoning_content`；`truncate()` 保留开头和结尾。`shell()` 的测试在每个 CI 平台上运行，检查同时有两路输出时的 exit 行、超时时的 kill，以及后台的 `sleep 30 &` 不会让命令一直挂着。

有一个端到端测试会启动本地 HTTP 服务器（`serve()`）回放预设的回复，驱动 `run()` 依次经历一次工具调用、一次上下文溢出的 400 及其重试，然后从保存的 session 恢复，检查发送的消息是否相同、是否没有残留临时文件。它还检查没有 `choices` 的 HTTP 200 会被重试，而不是被当作最终回复。这是唯一会涉及 `RESTLOOP_*` 变量的测试；它会先清掉这些变量以及所有 `*_PROXY`，因此你 shell 里的设置不会混进来。

<sub>来源: [tests.rs:8-88](../../src/tests.rs#L8-L88) · [tests.rs:90-126](../../src/tests.rs#L90-L126) · [tests.rs:128-215](../../src/tests.rs#L128-L215) · [commit 4c468f27](https://github.com/lhwzds/restloop/commit/4c468f27) · [commit 333b7b64](https://github.com/lhwzds/restloop/commit/333b7b64)</sub>

## 修改循环

从项目历史中可以总结出几条约定：

- **选项用环境变量，默认值保持旧行为。** 用 `env_num` 读取数值，这样无效值会回退到默认值；并把变量加到 `main()` 的 `--help` 文本和 README 里。
- **保持面向模型的部分固定不变**（见 [概览](01-overview.md)）。修改 `system_prompt()` 或工具 schema 应附上 benchmark 证据。
- **保持 prompt 前缀稳定。** 任何改写早先消息的操作都会破坏服务商的 prompt 缓存；RestLoop 只在服务商报告溢出时才这样做。基于阈值的裁剪在一次 A/B 测试显示没有收益后已被移除（`f09603f4`）。
- **程序保持一个文件、两个依赖。** 程序就是 `src/main.rs`，测试在 `src/tests.rs`。程序的小巧和可读性正是这个项目的意义所在。

<sub>来源: [main.rs:118-124](../../src/main.rs#L118-L124) · [tests.rs:1-6](../../src/tests.rs#L1-L6) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c) · [commit 740555b1](https://github.com/lhwzds/restloop/commit/740555b1) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

## 如何发布一个版本

推送 `vX.Y.Z` 形式的 tag（预发布 tag 不匹配）会运行 `.github/workflows/release.yml`：

1. **build** 编译五个目标：macOS arm64 和 x86_64、Linux x86_64 和 arm64（静态 musl）、Windows x86_64，命名为 `restloop-<target>[.exe]`。
2. **release** 写出 `SHA256SUMS` 以及每个二进制对应的一个 `.sha256`，并创建 GitHub release。如果该 release 已存在，它会拒绝运行：已发布的二进制永远不会被替换，因为 npm 包固定了它们的校验和。此时应只重跑失败的 job。
3. **homebrew** 重新生成 `lhwzds/homebrew-tap` 中的 `restloop` formula（需要 `HOMEBREW_TAP_TOKEN` secret），只有内容变化时才提交。

手动运行该 workflow（`workflow_dispatch`）会构建并计算校验和，但不发布任何东西，相当于一次演练。

<sub>来源: [release.yml:1-75](../../.github/workflows/release.yml#L1-L75) · [release.yml:77-96](../../.github/workflows/release.yml#L77-L96) · [release.yml:124-139](../../.github/workflows/release.yml#L124-L139) · [commit b41c62a7](https://github.com/lhwzds/restloop/commit/b41c62a7) · [commit e077f5e0](https://github.com/lhwzds/restloop/commit/e077f5e0)</sub>

## npm 包

npm 包里不含二进制。`npm/bin/restloop.js` 在 `checksums.json` 中查找 `"<platform> <arch>"`，首次运行时从 `v<version>` GitHub release 下载对应的 asset，校验其 SHA-256，并存放在自己旁边（先写临时文件再重命名）。之后的运行跳过下载。无论哪种情况，launcher 都会运行该二进制，透传参数和 stdio，并返回其退出状态。

发布是手动的，因为 npm trusted publishing 需要修改账号设置，所以 npm 不放进 workflow（`a21f7c21`）：GitHub release 完成后，运行 `node npm/stamp.js X.Y.Z SHA256SUMS`，它会设置包版本并为五个平台重写 `checksums.json`，然后在 `npm/` 下执行 `npm publish --access public`。在这个 commit 上，`checksums.json` 包含 v0.2.0 的五个条目：`darwin arm64`、`darwin x64`、`linux x64`、`linux arm64`（musl）和 `win32 x64`。

<sub>来源: [restloop.js:1-61](../../npm/bin/restloop.js#L1-L61) · [stamp.js:1-31](../../npm/stamp.js#L1-L31) · [checksums.json](../../npm/checksums.json) · [release.yml:5-6](../../.github/workflows/release.yml#L5-L6) · [commit a21f7c21](https://github.com/lhwzds/restloop/commit/a21f7c21)</sub>
