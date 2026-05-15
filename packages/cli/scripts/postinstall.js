#!/usr/bin/env node

import { execSync } from "node:child_process";
import { existsSync, mkdirSync, chmodSync, createWriteStream, unlinkSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { get } from "node:https";
import { createGunzip } from "node:zlib";

const __dirname = dirname(fileURLToPath(import.meta.url));
const BIN_DIR = join(__dirname, "..", "bin");
const IS_WINDOWS = process.platform === "win32";
const BINARY_NAME = IS_WINDOWS ? "claude-dejavu.exe" : "claude-dejavu";
const BINARY_PATH = join(BIN_DIR, BINARY_NAME);

if (existsSync(BINARY_PATH)) {
  process.exit(0);
}

const PLATFORM_MAP = {
  "darwin-arm64": "claude-dejavu-darwin-arm64.tar.gz",
  "darwin-x64": "claude-dejavu-darwin-x64.tar.gz",
  "linux-x64": "claude-dejavu-linux-x64.tar.gz",
  "linux-arm64": "claude-dejavu-linux-arm64.tar.gz",
  "win32-x64": "claude-dejavu-win-x64.zip",
};

const key = `${process.platform}-${process.arch}`;
const asset = PLATFORM_MAP[key];

if (!asset) {
  console.error(`claude-dejavu: unsupported platform ${key}`);
  console.error("Build from source: cargo build --release -p dejavu-cli");
  process.exit(0); // Don't fail install
}

// Read version from package.json
const pkg = JSON.parse(
  (await import("node:fs")).readFileSync(join(__dirname, "..", "package.json"), "utf-8")
);
const version = pkg.version;
const url = `https://github.com/cmg8431/claude-dejavu/releases/download/v${version}/${asset}`;

console.log(`claude-dejavu: downloading binary for ${key}...`);

mkdirSync(BIN_DIR, { recursive: true });

function download(url) {
  return new Promise((resolve, reject) => {
    get(url, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        download(res.headers.location).then(resolve).catch(reject);
        return;
      }
      if (res.statusCode !== 200) {
        reject(new Error(`HTTP ${res.statusCode}`));
        return;
      }
      resolve(res);
    }).on("error", reject);
  });
}

try {
  const res = await download(url);

  if (asset.endsWith(".tar.gz")) {
    // Extract tar.gz
    const tmpFile = join(BIN_DIR, asset);
    const ws = createWriteStream(tmpFile);
    await new Promise((resolve, reject) => {
      res.pipe(ws);
      ws.on("finish", resolve);
      ws.on("error", reject);
    });

    execSync(`tar xzf "${tmpFile}" -C "${BIN_DIR}"`, { stdio: "pipe" });
    unlinkSync(tmpFile);
  } else {
    // .zip for Windows
    const tmpFile = join(BIN_DIR, asset);
    const ws = createWriteStream(tmpFile);
    await new Promise((resolve, reject) => {
      res.pipe(ws);
      ws.on("finish", resolve);
      ws.on("error", reject);
    });

    execSync(`unzip -o "${tmpFile}" -d "${BIN_DIR}"`, { stdio: "pipe" });
    unlinkSync(tmpFile);
  }

  if (!IS_WINDOWS) {
    chmodSync(BINARY_PATH, 0o755);
  }

  console.log(`claude-dejavu: binary installed to ${BINARY_PATH}`);
} catch (err) {
  console.error(`claude-dejavu: failed to download binary: ${err.message}`);
  console.error("Build from source: cargo build --release -p dejavu-cli");
  process.exit(0); // Don't fail install
}
