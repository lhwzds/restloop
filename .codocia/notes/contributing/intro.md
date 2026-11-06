---
title: 
order: 0
evidence:
  - Cargo.toml:1-22 #29a495708046
  - src/tests.rs:1-2 #0cb9d8c7f1af
  - src/main.rs:170-171 #4ea5f3f63113
  - .github/workflows/ci.yml:1-37 #358d16c02af6
---

`cargo build --release` produces `target/release/restloop`. The crate uses edition 2024 and `let` chains inside `if`, so it needs a toolchain where both are stable (Rust 1.88 or newer). The release profile optimises for size (`opt-level = "z"`, LTO, one codegen unit, `panic = "abort"`, stripped).

The core's and the runner's tests live in `src/tests.rs`, the CLI's (settings, key file, session file) in `src/main.rs`; all run with `cargo test`. `.github/workflows/ci.yml` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the tests on macOS, Linux and Windows for every push to `main` and every pull request; on Unix it also runs `tests/cleanup.rs` in release. Run the same three commands before opening one. A `docs` job on Linux checks out the full history (`fetch-depth: 0`, so cited commits resolve), installs Codocia 0.2 and runs `codocia check`, so stale docs or `README_CN.md` fail CI.
