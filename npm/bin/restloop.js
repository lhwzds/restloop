#!/usr/bin/env node
// Runs the restloop binary for this platform. On first use it downloads the binary
// from the matching GitHub release, checks its SHA-256 and keeps it next to this file.
"use strict";
const crypto = require("crypto");
const fs = require("fs");
const https = require("https");
const path = require("path");
const { spawnSync } = require("child_process");

const VERSION = require("../package.json").version;
const ASSETS = {
  "darwin arm64": ["restloop-aarch64-apple-darwin", "c02d35077979e9cbef281929939fde3c921bc1b2b9e7b551e423fbf2f593544d"],
  "darwin x64": ["restloop-x86_64-apple-darwin", "d096b991173eb97b76cb5e4586c18103f279e4f4fd2a899a457573c312bef763"],
  "linux x64": ["restloop-x86_64-unknown-linux-musl", "a987f3507c8a8e4b7a6e0faaacdbc81cd8f457ac77d484934548d3fd1b389d17"],
  "win32 x64": ["restloop-x86_64-pc-windows-msvc.exe", "8303882528dafe520c936cba2a3b7274ce81e27c16a202e74570e09116817c6a"],
};

function fail(msg) {
  console.error(`restloop: ${msg}`);
  process.exit(1);
}

function download(url, redirects = 5) {
  return new Promise((resolve, reject) => {
    https.get(url, { headers: { "User-Agent": "restloop-npm" } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location && redirects > 0) {
        res.resume();
        return resolve(download(res.headers.location, redirects - 1));
      }
      if (res.statusCode !== 200) {
        res.resume();
        return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
      }
      const chunks = [];
      res.on("data", (c) => chunks.push(c));
      res.on("end", () => resolve(Buffer.concat(chunks)));
    }).on("error", reject);
  });
}

async function main() {
  const asset = ASSETS[`${process.platform} ${process.arch}`];
  if (!asset) fail(`no binary for ${process.platform} ${process.arch}; see https://github.com/lhwzds/restloop`);
  const [name, sha256] = asset;
  const bin = path.join(__dirname, name);
  if (!fs.existsSync(bin)) {
    const url = `https://github.com/lhwzds/restloop/releases/download/v${VERSION}/${name}`;
    let data;
    try {
      data = await download(url);
    } catch (e) {
      fail(`cannot download ${url}: ${e.message}`);
    }
    if (crypto.createHash("sha256").update(data).digest("hex") !== sha256) fail(`checksum mismatch for ${url}`);
    const tmp = `${bin}.${process.pid}.tmp`;
    fs.writeFileSync(tmp, data, { mode: 0o755 });
    fs.renameSync(tmp, bin);
  }
  const r = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
  if (r.error) fail(r.error.message);
  process.exit(r.status === null ? 1 : r.status);
}

main();
