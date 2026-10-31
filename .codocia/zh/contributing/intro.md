---
title: 
source: #2d7354041ea7
---

`cargo build --release` 生成 `target/release/restloop`。这个 crate 使用 edition 2024，并在 `if` 中使用 `let` 链，所以需要两者都已稳定的工具链（Rust 1.88 或更新）。release profile 针对体积优化（`opt-level = "z"`、LTO、单个 codegen unit、`panic = "abort"`、strip）。

测试位于 `src/tests.rs`，用 `cargo test` 运行；`.github/workflows/ci.yml` 会在每次 push 到 `main` 和每个 pull request 时，在 macOS、Linux 和 Windows 上运行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings` 和测试。提 pull request 之前，请先在本地跑一遍这三条命令。另有一个在 Linux 上运行的 `docs` job，检出完整历史（`fetch-depth: 0`，这样引用的 commit 才能找到），安装 Codocia 0.2 并运行 `codocia check`，所以过时的文档或 `README_CN.md` 会让 CI 失败。
