import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

function getBinaryName(): string {
  return process.platform === "win32"
    ? "claude-dejavu.exe"
    : "claude-dejavu";
}

function findBinary(): string | null {
  const bin = getBinaryName();

  // 1. Postinstall download location (primary)
  const downloaded = join(__dirname, "..", "bin", bin);
  if (existsSync(downloaded)) return downloaded;

  // 2. Cargo build output (dev)
  const cargoRelease = join(__dirname, "..", "..", "..", "target", "release", bin);
  if (existsSync(cargoRelease)) return cargoRelease;

  const cargoDebug = join(__dirname, "..", "..", "..", "target", "debug", bin);
  if (existsSync(cargoDebug)) return cargoDebug;

  // 3. System PATH
  const which = spawnSync("which", [bin], { encoding: "utf-8" });
  if (which.status === 0 && which.stdout.trim()) {
    return which.stdout.trim();
  }

  return null;
}

const binary = findBinary();

if (!binary) {
  console.error(
    "Error: Could not find claude-dejavu binary.\n" +
      "Try reinstalling: npm install -g claude-dejavu\n" +
      "Or build from source: cargo build --release -p dejavu-cli"
  );
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

process.exit(result.status ?? 1);
