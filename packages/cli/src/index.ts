import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

function getPackageName(): string | null {
  const platform = process.platform;
  const arch = process.arch;

  const map: Record<string, string> = {
    "darwin-arm64": "cli-darwin-arm64",
    "darwin-x64": "cli-darwin-x64",
    "linux-x64": "cli-linux-x64-gnu",
    "linux-arm64": "cli-linux-arm64-gnu",
    "win32-x64": "cli-win32-x64-msvc",
  };

  return map[`${platform}-${arch}`] || null;
}

function getBinaryName(): string {
  return process.platform === "win32"
    ? "claude-dejavu.exe"
    : "claude-dejavu";
}

function findBinary(): string | null {
  const pkg = getPackageName();
  const bin = getBinaryName();

  if (pkg) {
    const npmPath = join(__dirname, "..", "node_modules", "@claude-dejavu", pkg, "bin", bin);
    if (existsSync(npmPath)) return npmPath;

    const hoisted = join(__dirname, "..", "..", "@claude-dejavu", pkg, "bin", bin);
    if (existsSync(hoisted)) return hoisted;

    const npx = join(__dirname, "..", "..", "..", "node_modules", "@claude-dejavu", pkg, "bin", bin);
    if (existsSync(npx)) return npx;

    const nested = join(__dirname, "..", "node_modules", ".pnpm", "node_modules", "@claude-dejavu", pkg, "bin", bin);
    if (existsSync(nested)) return nested;
  }

  const cargoRelease = join(__dirname, "..", "..", "..", "target", "release", bin);
  if (existsSync(cargoRelease)) return cargoRelease;

  const cargoDebug = join(__dirname, "..", "..", "..", "target", "debug", bin);
  if (existsSync(cargoDebug)) return cargoDebug;

  if (pkg) {
    const monorepo = join(__dirname, "..", "..", pkg, "bin", bin);
    if (existsSync(monorepo)) return monorepo;
  }

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
      "Run `cargo build -p dejavu-cli` to build from source."
  );
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  env: process.env,
});

process.exit(result.status ?? 1);
