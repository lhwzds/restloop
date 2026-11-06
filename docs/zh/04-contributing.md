---
title: 构建、发布与贡献
---

# 构建、发布与贡献

<sub>[English](../04-contributing.md) · **中文**</sub>

`cargo build --release` 生成 `target/release/restloop`。这个 crate 使用 edition 2024，并在 `if` 中使用 `let` 链，所以需要两者都已稳定的工具链（Rust 1.88 或更新）。release profile 针对体积优化（`opt-level = "z"`、LTO、单个 codegen unit、`panic = "abort"`、strip）。

核心和命令执行器的测试在 `src/tests.rs`，CLI 的测试（设置、key 文件、session 文件）在 `src/main.rs`，都用 `cargo test` 运行；`.github/workflows/ci.yml` 会在每次 push 到 `main` 和每个 pull request 时，在 macOS、Linux 和 Windows 上运行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings` 和测试；在 Unix 上还会以 release 运行 `tests/cleanup.rs`。提 pull request 之前，请先在本地跑一遍这三条命令。另有一个在 Linux 上运行的 `docs` job，检出完整历史（`fetch-depth: 0`，这样引用的 commit 才能找到），安装 Codocia 0.2 并运行 `codocia check`，所以过时的文档或 `README_CN.md` 会让 CI 失败。

<sub>来源: [Cargo.toml:1-22](../../Cargo.toml#L1-L22) · [tests.rs:1-2](../../src/tests.rs#L1-L2) · [main.rs:170-171](../../src/main.rs#L170-L171) · [ci.yml:1-37](../../.github/workflows/ci.yml#L1-L37)</sub>

## 测试覆盖了什么

`src/tests.rs` 中的单元测试锁定以下行为：`too_long()` 接受记录在案的服务商措辞，并拒绝相近但不应匹配的情况；`drop_old_output()` 不断减半，直到返回 0；`trim_thinking()` 保留最后两条回复；`truncate()` 保留开头和结尾。`shell()` 的测试：同时有两路输出时的 exit 行、超时 kill、后台的 `sleep 30 &` 不会阻塞。

有一个端到端测试让 `run()` 对接 `serve()`，一个回放预设回复的服务器：一次工具调用、溢出恢复、恢复 session、对空回复或没有 `choices` 的回复的重试；Responses API 下，按顺序发回的输出项、可重试和立即失败的错误；设置 deadline 时，挂起、500 和空回复都会提前以 `deadline of` 结束。只有它会设置 restloop 自己的设置，并检查命令拿不到其中任何一个。用到环境的测试共用一把锁，发请求的测试会清掉 `*_PROXY`，改过的变量都会恢复。其他核心测试用一个记录命令的工具代替 shell，检查已过 deadline 时不运行任何命令、key 失败时立即结束运行。CLI 的测试在 `src/main.rs`，覆盖 `config()`、key 文件和 session 文件；`tests/cleanup.rs`（Unix）运行二进制，检查没有进程活过一次运行。

<sub>来源: [tests.rs:167-310](../../src/tests.rs#L167-L310) · [tests.rs:30-96](../../src/tests.rs#L30-L96) · [tests.rs:443-468](../../src/tests.rs#L443-L468) · [tests.rs:832-908](../../src/tests.rs#L832-L908) · [tests.rs:930-1077](../../src/tests.rs#L930-L1077) · +6</sub>

## 修改循环

历史中总结出的约定：

- **选项用环境变量，默认值保持旧行为。** CLI 在 `config()` 中把它读进 `Config` 的一个字段；用 `env_num` 读取数值，这样无效值会回退到默认值；并把变量加到 `main()` 的 `--help` 文本和 README 里。
- **保持面向模型的部分固定不变**（见 [概览](01-overview.md)）。修改 `system_prompt()` 或工具 schema 应附上 benchmark 证据。
- **保持 prompt 前缀稳定。** 任何改写早先消息的操作都会破坏服务商的 prompt 缓存；RestLoop 只在服务商报告溢出时才这样做。基于阈值的裁剪在一次 A/B 测试显示没有收益后已被移除（`f09603f4`）。
- **核心的设置都放在 `Config`，保持很少的依赖。** `src/lib.rs` 不直接读取环境变量（其 HTTP 客户端 `ureq` 会遵循 `*_PROXY`），也不启动任何进程；这些属于 CLI（`src/main.rs`）和默认的命令执行器，即公开模块 `src/shell.rs`。小巧和可读性正是意义所在。

<sub>来源: [lib.rs:1-9](../../src/lib.rs#L1-L9) · [main.rs:165-168](../../src/main.rs#L165-L168) · [tests.rs:1-2](../../src/tests.rs#L1-L2) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c) · [commit 740555b1](https://github.com/lhwzds/restloop/commit/740555b1) · +1</sub>

## 如何发布一个版本

推送 `vX.Y.Z` 形式的 tag（预发布 tag 不匹配）会运行 `.github/workflows/release.yml`：

1. **build** 编译五个目标：macOS arm64 和 x86_64、Linux x86_64 和 arm64（静态 musl）、Windows x86_64，命名为 `restloop-<target>[.exe]`。
2. **release** 写出 `SHA256SUMS` 以及每个二进制对应的一个 `.sha256`，并创建 GitHub release。如果该 release 已存在，它会拒绝运行：已发布的二进制永远不会被替换，因为 npm 包固定了它们的校验和。此时应只重跑失败的 job。
3. **homebrew** 重新生成 `lhwzds/homebrew-tap` 中的 `restloop` formula（需要 `HOMEBREW_TAP_TOKEN` secret），只有内容变化时才提交。
4. **npm** 同样在 **release** 之后运行，用 trusted publishing stamp 并发布 npm 包；下面的 npm 包一节会详细说明。

手动运行该 workflow（`workflow_dispatch`）会构建并计算校验和，但不发布任何东西，相当于一次演练。

<sub>来源: [release.yml:1-75](../../.github/workflows/release.yml#L1-L75) · [release.yml:76-95](../../.github/workflows/release.yml#L76-L95) · [release.yml:123-138](../../.github/workflows/release.yml#L123-L138) · [release.yml:140-164](../../.github/workflows/release.yml#L140-L164) · [commit b41c62a7](https://github.com/lhwzds/restloop/commit/b41c62a7) · +1</sub>

## npm 包

npm 包里不含二进制。`npm/bin/restloop.js` 在 `checksums.json` 中查找 `"<platform> <arch>"`，首次运行时从 `v<version>` GitHub release 下载对应的 asset，校验其 SHA-256，并存放在自己旁边（先写临时文件再重命名）。之后的运行跳过下载。无论哪种情况，launcher 都会运行该二进制，透传参数和 stdio，并返回其退出状态。

推送 tag 后由 CI 发布该包（`f76038cb`）：GitHub release 完成后，`npm` job 运行 `node npm/stamp.js X.Y.Z dist/SHA256SUMS`，它会设置包版本并为五个平台重写 `checksums.json`，然后在 `npm/` 下执行 `npm publish --access public`。它使用 npm trusted publishing（OIDC、`id-token: write`、npm 11.5.1 或更高版本），因此不涉及任何 npm token 或 secret。更早的版本是手动发布的（`a21f7c21`）。

stamp 后的 `package.json` 和 `checksums.json` 只存在于 runner 上，从不提交回仓库，因此仓库中的副本停留在最后一次手动 stamp（0.3.0），会落后于已发布的版本。在这个 commit 上，`checksums.json` 包含 v0.3.0 的五个条目：`darwin arm64`、`darwin x64`、`linux x64`、`linux arm64`（musl）和 `win32 x64`。

<sub>来源: [restloop.js:1-61](../../npm/bin/restloop.js#L1-L61) · [stamp.js:1-31](../../npm/stamp.js#L1-L31) · [checksums.json](../../npm/checksums.json) · [release.yml:140-164](../../.github/workflows/release.yml#L140-L164) · [commit a21f7c21](https://github.com/lhwzds/restloop/commit/a21f7c21) · +1</sub>
