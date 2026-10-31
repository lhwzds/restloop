---
title: How a release is made
order: 20
evidence:
  - .github/workflows/release.yml:1-75 #f62a673c7930
  - .github/workflows/release.yml:76-95 #a831c70ed854
  - .github/workflows/release.yml:123-138 #704b738c2a5c
  - .github/workflows/release.yml:140-164 #0aa29aa479e3
  - commit:b41c62a7
  - commit:e077f5e0
---

Pushing a tag `vX.Y.Z` (prerelease tags do not match) runs `.github/workflows/release.yml`:

1. **build** compiles five targets: macOS arm64 and x86_64, Linux x86_64 and arm64 (static musl), Windows x86_64, named `restloop-<target>[.exe]`.
2. **release** writes `SHA256SUMS` plus one `.sha256` per binary and creates the GitHub release. It refuses to run if the release already exists: published binaries are never replaced, because the npm package pins their checksums. Re-run only failed jobs instead.
3. **homebrew** regenerates the `restloop` formula in `lhwzds/homebrew-tap` (needs the `HOMEBREW_TAP_TOKEN` secret) and commits only if it changed.
4. **npm**, also after **release**, stamps and publishes the npm package with trusted publishing; the npm package section below explains it.

Running the workflow by hand (`workflow_dispatch`) builds and checksums but publishes nothing, which makes it a dry run.
