---
title: Building, releasing and contributing
---

# Building, releasing and contributing

<sub>**English** · [中文](zh/04-contributing.md)</sub>

`cargo build --release` produces `target/release/restloop`. The crate uses edition 2024 and `let` chains inside `if`, so it needs a toolchain where both are stable (Rust 1.88 or newer). The release profile optimises for size (`opt-level = "z"`, LTO, one codegen unit, `panic = "abort"`, stripped).

The core's and the runner's tests live in `src/tests.rs`, the CLI's (settings, key file, session file) in `src/main.rs`; all run with `cargo test`. `.github/workflows/ci.yml` runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the tests on macOS, Linux and Windows for every push to `main` and every pull request; on Unix it also runs `tests/cleanup.rs` in release. Run the same three commands before opening one. A `docs` job on Linux checks out the full history (`fetch-depth: 0`, so cited commits resolve), installs Codocia 0.2 and runs `codocia check`, so stale docs or `README_CN.md` fail CI.

<sub>Sources: [Cargo.toml:1-22](../Cargo.toml#L1-L22) · [tests.rs:1-2](../src/tests.rs#L1-L2) · [main.rs:170-171](../src/main.rs#L170-L171) · [ci.yml:1-37](../.github/workflows/ci.yml#L1-L37)</sub>

## What the tests cover

Unit tests in `src/tests.rs` pin: `too_long()` accepts recorded provider wordings and rejects near misses; `drop_old_output()` halves until it returns 0; `trim_thinking()` spares the last two replies; `truncate()` keeps head and tail. `shell()` tests: the exit line with both streams, a timeout kill, a background `sleep 30 &` not blocking.

One end-to-end test drives `run()` against `serve()`, a canned-reply server: a tool call, overflow recovery, a session resume, retried empty replies or replies without `choices`; over the Responses API, output items sent back in order, retried and fatal failures; under a deadline, hangs, 500s and empty replies stop early with `deadline of`. It alone sets restloop's settings and checks commands get none. Tests that use the environment hold one lock, tests that send requests clear `*_PROXY`, and changed variables are put back. Other core tests use a recording tool and check that a passed deadline runs no command and a failing key ends the run at once. The CLI's tests in `src/main.rs` cover `config()`, the key file and session files; `tests/cleanup.rs` (Unix) runs the binary and checks no process outlives a run.

<sub>Sources: [tests.rs:167-310](../src/tests.rs#L167-L310) · [tests.rs:30-96](../src/tests.rs#L30-L96) · [tests.rs:443-468](../src/tests.rs#L443-L468) · [tests.rs:832-908](../src/tests.rs#L832-L908) · [tests.rs:930-1077](../src/tests.rs#L930-L1077) · +6</sub>

## Changing the loop

Conventions the history suggests:

- **Options are environment variables with the old behaviour as default.** The CLI reads them in `config()` into a `Config` field; read numbers with `env_num` so an invalid value falls back to the default, and add the variable to the `--help` text in `main()` and to the README.
- **Keep the model-facing surface fixed** (see [Overview](01-overview.md)). Changes to `system_prompt()` or the tool schema should come with benchmark evidence.
- **Keep the prompt prefix stable.** Anything that rewrites earlier messages breaks provider prompt caching; RestLoop does it only when the provider reports an overflow. A threshold-based trim was removed after an A/B showed no benefit (`f09603f4`).
- **Keep the core's settings in `Config`, with few dependencies.** `src/lib.rs` makes no direct environment reads (its HTTP client, `ureq`, honours `*_PROXY`) and starts no process; that belongs to the CLI (`src/main.rs`) and the default command runner, the public module `src/shell.rs`. Its size and readability are the point.

<sub>Sources: [lib.rs:1-9](../src/lib.rs#L1-L9) · [main.rs:165-168](../src/main.rs#L165-L168) · [tests.rs:1-2](../src/tests.rs#L1-L2) · [commit 3b27682c](https://github.com/lhwzds/restloop/commit/3b27682c) · [commit 740555b1](https://github.com/lhwzds/restloop/commit/740555b1) · +1</sub>

## How a release is made

Pushing a tag `vX.Y.Z` (prerelease tags do not match) runs `.github/workflows/release.yml`:

1. **build** compiles five targets: macOS arm64 and x86_64, Linux x86_64 and arm64 (static musl), Windows x86_64, named `restloop-<target>[.exe]`.
2. **release** writes `SHA256SUMS` plus one `.sha256` per binary and creates the GitHub release. It refuses to run if the release already exists: published binaries are never replaced, because the npm package pins their checksums. Re-run only failed jobs instead.
3. **homebrew** regenerates the `restloop` formula in `lhwzds/homebrew-tap` (needs the `HOMEBREW_TAP_TOKEN` secret) and commits only if it changed.
4. **npm**, also after **release**, stamps and publishes the npm package with trusted publishing; the npm package section below explains it.

Running the workflow by hand (`workflow_dispatch`) builds and checksums but publishes nothing, which makes it a dry run.

<sub>Sources: [release.yml:1-75](../.github/workflows/release.yml#L1-L75) · [release.yml:76-95](../.github/workflows/release.yml#L76-L95) · [release.yml:123-138](../.github/workflows/release.yml#L123-L138) · [release.yml:140-164](../.github/workflows/release.yml#L140-L164) · [commit b41c62a7](https://github.com/lhwzds/restloop/commit/b41c62a7) · +1</sub>

## The npm package

The npm package contains no binary. `npm/bin/restloop.js` looks up `"<platform> <arch>"` in `checksums.json`, and on first run downloads that asset from the `v<version>` GitHub release, verifies its SHA-256, and stores it next to itself (temporary file and rename). Later runs skip the download. Either way the launcher runs the binary, passing arguments and stdio through and returning its exit status.

A tag publishes the package from CI (`f76038cb`): after the GitHub release, the `npm` job runs `node npm/stamp.js X.Y.Z dist/SHA256SUMS`, which sets the package version and rewrites `checksums.json` for five platforms, then `npm publish --access public` from `npm/`. It uses npm trusted publishing (OIDC, `id-token: write`, npm 11.5.1 or later), so no npm token or secret is involved. Earlier releases were published by hand (`a21f7c21`).

The stamped `package.json` and `checksums.json` exist only on the runner and are never committed back, so the repository copies stay at the last manual stamp (0.3.0) and will lag the published version. At this commit `checksums.json` holds the five v0.3.0 entries: `darwin arm64`, `darwin x64`, `linux x64`, `linux arm64` (musl) and `win32 x64`.

<sub>Sources: [restloop.js:1-61](../npm/bin/restloop.js#L1-L61) · [stamp.js:1-31](../npm/stamp.js#L1-L31) · [checksums.json](../npm/checksums.json) · [release.yml:140-164](../.github/workflows/release.yml#L140-L164) · [commit a21f7c21](https://github.com/lhwzds/restloop/commit/a21f7c21) · +1</sub>
