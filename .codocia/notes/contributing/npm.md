---
title: The npm package
order: 30
evidence:
  - npm/bin/restloop.js:1-61 #e24057e5e84e
  - npm/stamp.js:1-31 #bd690607ff3b
  - npm/checksums.json #1fd11eeb6110
  - .github/workflows/release.yml:5-6 #79541f084580
  - commit:a21f7c21
---

The npm package contains no binary. `npm/bin/restloop.js` looks up `"<platform> <arch>"` in `checksums.json`, and on first run downloads that asset from the `v<version>` GitHub release, verifies its SHA-256, and stores it next to itself (temporary file and rename). Later runs skip the download. Either way the launcher runs the binary, passing arguments and stdio through and returning its exit status.

Publishing is manual, since npm trusted publishing would need an account setting (`a21f7c21`): after the GitHub release, run `node npm/stamp.js X.Y.Z SHA256SUMS`, which sets the package version and rewrites `checksums.json` for five platforms, then `npm publish --access public` from `npm/`. At this commit `checksums.json` holds the five v0.3.0 entries: `darwin arm64`, `darwin x64`, `linux x64`, `linux arm64` (musl) and `win32 x64`.
