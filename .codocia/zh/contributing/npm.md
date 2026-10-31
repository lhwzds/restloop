---
title: npm 包
source: #2d8c30d12486
---

npm 包里不含二进制。`npm/bin/restloop.js` 在 `checksums.json` 中查找 `"<platform> <arch>"`，首次运行时从 `v<version>` GitHub release 下载对应的 asset，校验其 SHA-256，并存放在自己旁边（先写临时文件再重命名）。之后的运行跳过下载。无论哪种情况，launcher 都会运行该二进制，透传参数和 stdio，并返回其退出状态。

发布是手动的，因为 npm trusted publishing 需要修改账号设置（`a21f7c21`）：GitHub release 完成后，运行 `node npm/stamp.js X.Y.Z SHA256SUMS`，它会设置包版本并为五个平台重写 `checksums.json`，然后在 `npm/` 下执行 `npm publish --access public`。在这个 commit 上，`checksums.json` 包含 v0.3.0 的五个条目：`darwin arm64`、`darwin x64`、`linux x64`、`linux arm64`（musl）和 `win32 x64`。
