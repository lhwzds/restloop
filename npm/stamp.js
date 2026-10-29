// Release helper: node npm/stamp.js <version> <SHA256SUMS>
// Sets the npm package version and writes npm/checksums.json from the release checksums.
"use strict";
const fs = require("fs");
const path = require("path");

const [version, sumsFile] = process.argv.slice(2);
if (!version || !sumsFile) throw new Error("usage: node npm/stamp.js <version> <SHA256SUMS>");
const sums = {};
for (const line of fs.readFileSync(sumsFile, "utf8").trim().split("\n")) {
  const [hash, file] = line.trim().split(/\s+/);
  sums[file.replace(/^\*/, "")] = hash;
}
const assets = {
  "darwin arm64": "restloop-aarch64-apple-darwin",
  "darwin x64": "restloop-x86_64-apple-darwin",
  "linux x64": "restloop-x86_64-unknown-linux-musl",
  "linux arm64": "restloop-aarch64-unknown-linux-musl",
  "win32 x64": "restloop-x86_64-pc-windows-msvc.exe",
};
const out = {};
for (const [platform, file] of Object.entries(assets)) {
  if (!sums[file]) throw new Error(`no checksum for ${file}`);
  out[platform] = [file, sums[file]];
}
fs.writeFileSync(path.join(__dirname, "checksums.json"), JSON.stringify(out, null, 2) + "\n");
const pkgPath = path.join(__dirname, "package.json");
const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
pkg.version = version;
fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
console.log(`npm package set to ${version} with ${Object.keys(out).length} binaries`);
