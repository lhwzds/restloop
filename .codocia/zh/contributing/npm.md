---
title: npm 包
source: #0c9a73e265f6
---

npm 包里不含二进制。`npm/bin/restloop.js` 在 `checksums.json` 中查找 `"<platform> <arch>"`，首次运行时从 `v<version>` GitHub release 下载对应的 asset，校验其 SHA-256，并存放在自己旁边（先写临时文件再重命名）。之后的运行跳过下载。无论哪种情况，launcher 都会运行该二进制，透传参数和 stdio，并返回其退出状态。

推送 tag 后由 CI 发布该包（`f76038cb`）：GitHub release 完成后，`npm` job 运行 `node npm/stamp.js X.Y.Z dist/SHA256SUMS`，它会设置包版本并为五个平台重写 `checksums.json`，然后在 `npm/` 下执行 `npm publish --access public`。它使用 npm trusted publishing（OIDC、`id-token: write`、npm 11.5.1 或更高版本），因此不涉及任何 npm token 或 secret。更早的版本是手动发布的（`a21f7c21`）。

stamp 后的 `package.json` 和 `checksums.json` 只存在于 runner 上，从不提交回仓库，因此仓库中的副本停留在最后一次手动 stamp（0.3.0），会落后于已发布的版本。在这个 commit 上，`checksums.json` 包含 v0.3.0 的五个条目：`darwin arm64`、`darwin x64`、`linux x64`、`linux arm64`（musl）和 `win32 x64`。
