---
title: 
source: #89f685b4f75b
---

`cargo build --release` 生成 `target/release/restloop`。这个 crate 使用 edition 2024，并在 `if` 中使用 `let` 链（在 `shell()` 里），所以需要两者都已稳定的工具链（Rust 1.88 或更新）。release profile 针对体积优化（`opt-level = "z"`、LTO、单个 codegen unit、`panic = "abort"`、strip）。

测试位于 `src/tests.rs`，用 `cargo test` 运行；`.github/workflows/ci.yml` 会在每次 push 到 `main` 和每个 pull request 时，在 macOS、Linux 和 Windows 上运行 `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings` 和测试。提 pull request 之前，请先在本地跑一遍这三条命令。
