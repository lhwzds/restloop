---
title: Building, releasing and contributing
---

# Building, releasing and contributing

<sub>**English** · [中文](zh/04-contributing.md)</sub>

`cargo build --release` produces `target/release/restloop`. The crate uses edition 2024 and `let` chains inside `if` (in `shell()`), so it needs a toolchain where both are stable (Rust 1.88 or newer). The release profile optimises for size (`opt-level = "z"`, LTO, one codegen unit, `panic = "abort"`, stripped).

Tests live in `src/tests.rs` and run with `cargo test`; `.github/workflows/ci.yml` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the tests on macOS, Linux and Windows for every push to `main` and every pull request. Run the same three commands before opening one.

<sub>Sources: [Cargo.toml:1-18](../Cargo.toml#L1-L18) · [main.rs:577-584](../src/main.rs#L577-L584) · [ci.yml:1-25](../.github/workflows/ci.yml#L1-L25)</sub>

## What the tests cover

The unit tests pin what other pages describe: `too_long()` accepts the recorded provider wordings and rejects near misses (an overlong `tool_call_id`, a 500, a network error); `drop_old_output()` halves the remaining outputs until it returns 0; `trim_thinking()` spares the last two replies, empties rather than removes `reasoning_content`, and drops only the `reasoning` items of `output_items`; `truncate()` keeps head and tail. The `shell()` tests run on each CI platform: the exit line with both streams, a kill at the timeout, and a background `sleep 30 &` not blocking.

One end-to-end test drives `run()` against a local server (`serve()`) replaying canned replies: a tool call, a context-overflow 400 and its retry, a resume from the saved session, a retried 200 without `choices`. Then, over the Responses API: the key file, output items sent back in order (message and function-call ids dropped; reasoning ids and `call_id` kept), retried server and unavailable-plan failures, overflow recovery, and immediate failure for a used-up plan, a content filter, an invalid request or an unreadable key file. Only this test touches `RESTLOOP_*` variables; it clears them and any `*_PROXY` first.

<sub>Sources: [tests.rs:8-97](../src/tests.rs#L8-L97) · [tests.rs:96-139](../src/tests.rs#L96-L139) · [tests.rs:141-370](../src/tests.rs#L141-L370) · [commit 4c468f27](https://github.com/lhwzds/restloop/commit/4c468f27) · [commit 333b7b64](https://github.com/lhwzds/restloop/commit/333b7b64)</sub>

## Changing the loop

Conventions the history suggests:

- **Options are environment variables with the old behaviour as default.** Read numbers with `env_num` so an invalid value falls back to the default, and add the variable to the `--help` text in `main()` and to the README.
- **Keep the model-facing surface fixed** (see [Overview](01-overview.md)). Changes to `system_prompt()` or the tool schema should come with benchmark evidence.
- **Keep the prompt prefix stable.** Anything that rewrites earlier messages breaks provider prompt caching; RestLoop does it only when the provider reports an overflow. A threshold-based trim was removed after an A/B showed no benefit (`f09603f4`).
- **Keep the program in one file with two dependencies.** The program is `src/main.rs`; its tests live in `src/tests.rs`. Its size and readability are the point.

<sub>Sources: [main.rs:118-124](../src/main.rs#L118-L124) · [tests.rs:1-6](../src/tests.rs#L1-L6) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c) · [commit 740555b1](https://github.com/lhwzds/restloop/commit/740555b1) · [commit f09603f4](https://github.com/lhwzds/restloop/commit/f09603f4)</sub>

## How a release is made

Pushing a tag `vX.Y.Z` (prerelease tags do not match) runs `.github/workflows/release.yml`:

1. **build** compiles five targets: macOS arm64 and x86_64, Linux x86_64 and arm64 (static musl), Windows x86_64, named `restloop-<target>[.exe]`.
2. **release** writes `SHA256SUMS` plus one `.sha256` per binary and creates the GitHub release. It refuses to run if the release already exists: published binaries are never replaced, because the npm package pins their checksums. Re-run only failed jobs instead.
3. **homebrew** regenerates the `restloop` formula in `lhwzds/homebrew-tap` (needs the `HOMEBREW_TAP_TOKEN` secret) and commits only if it changed.

Running the workflow by hand (`workflow_dispatch`) builds and checksums but publishes nothing, which makes it a dry run.

<sub>Sources: [release.yml:1-75](../.github/workflows/release.yml#L1-L75) · [release.yml:77-96](../.github/workflows/release.yml#L77-L96) · [release.yml:124-139](../.github/workflows/release.yml#L124-L139) · [commit b41c62a7](https://github.com/lhwzds/restloop/commit/b41c62a7) · [commit e077f5e0](https://github.com/lhwzds/restloop/commit/e077f5e0)</sub>

## The npm package

The npm package contains no binary. `npm/bin/restloop.js` looks up `"<platform> <arch>"` in `checksums.json`, and on first run downloads that asset from the `v<version>` GitHub release, verifies its SHA-256, and stores it next to itself (temporary file and rename). Later runs skip the download. Either way the launcher runs the binary, passing arguments and stdio through and returning its exit status.

Publishing is manual, since npm trusted publishing would need an account setting (`a21f7c21`): after the GitHub release, run `node npm/stamp.js X.Y.Z SHA256SUMS`, which sets the package version and rewrites `checksums.json` for five platforms, then `npm publish --access public` from `npm/`. At this commit `checksums.json` holds the five v0.2.0 entries: `darwin arm64`, `darwin x64`, `linux x64`, `linux arm64` (musl) and `win32 x64`.

<sub>Sources: [restloop.js:1-61](../npm/bin/restloop.js#L1-L61) · [stamp.js:1-31](../npm/stamp.js#L1-L31) · [checksums.json](../npm/checksums.json) · [release.yml:5-6](../.github/workflows/release.yml#L5-L6) · [commit a21f7c21](https://github.com/lhwzds/restloop/commit/a21f7c21)</sub>
