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
// Platform -> [release asset, SHA-256], written by the release workflow for each version.
const ASSETS = require("../checksums.json");

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
