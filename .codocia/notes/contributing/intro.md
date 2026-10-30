---
title: 
order: 0
evidence:
  - Cargo.toml:1-18 #bba2d10ce9b9
  - src/main.rs:577-584 #1c89b56451ba
  - .github/workflows/ci.yml:1-25 #6884bd7bae08
---

`cargo build --release` produces `target/release/restloop`. The crate uses edition 2024 and `let` chains inside `if` (in `shell()`), so it needs a toolchain where both are stable (Rust 1.88 or newer). The release profile optimises for size (`opt-level = "z"`, LTO, one codegen unit, `panic = "abort"`, stripped).

Tests live in `src/tests.rs` and run with `cargo test`; `.github/workflows/ci.yml` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the tests on macOS, Linux and Windows for every push to `main` and every pull request. Run the same three commands before opening one.
