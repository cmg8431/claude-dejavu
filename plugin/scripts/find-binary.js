import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { spawnSync } from 'child_process';
import { fileURLToPath } from 'url';
import { homedir } from 'os';

const __dirname = dirname(fileURLToPath(import.meta.url));
const IS_WINDOWS = process.platform === 'win32';

export function findBinary() {
  const bin = IS_WINDOWS ? 'claude-dejavu.exe' : 'claude-dejavu';

  // 1. Cargo build output (dev)
  const cargoRelease = join(__dirname, '..', '..', 'target', 'release', bin);
  if (existsSync(cargoRelease)) return cargoRelease;

  const cargoDebug = join(__dirname, '..', '..', 'target', 'debug', bin);
  if (existsSync(cargoDebug)) return cargoDebug;

  // 2. Bundled in plugin package
  const pluginBin = join(__dirname, '..', 'bin', bin);
  if (existsSync(pluginBin)) return pluginBin;

  // 3. npm global install
  const npmBin = join(__dirname, '..', '..', 'node_modules', '.bin', bin);
  if (existsSync(npmBin)) return npmBin;

  // 4. System PATH
  const which = spawnSync(IS_WINDOWS ? 'where' : 'which', [bin], {
    encoding: 'utf-8',
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  if (which.status === 0 && which.stdout.trim()) {
    return which.stdout.trim().split('\n')[0];
  }

  return null;
}

export function getDataDir() {
  const custom = process.env.CLAUDE_DEJAVU_DATA_DIR;
  if (custom) return custom;

  const platform = process.platform;
  if (platform === 'darwin') {
    return join(homedir(), 'Library', 'Application Support', 'claude-dejavu');
  }
  if (platform === 'win32') {
    return join(process.env.APPDATA || join(homedir(), 'AppData', 'Roaming'), 'claude-dejavu');
  }
  return join(process.env.XDG_DATA_HOME || join(homedir(), '.local', 'share'), 'claude-dejavu');
}

export function collectStdin() {
  return new Promise((resolve) => {
    if (process.stdin.isTTY) {
      resolve(null);
      return;
    }

    const chunks = [];
    process.stdin.on('data', (chunk) => chunks.push(chunk));
    process.stdin.on('end', () => {
      resolve(chunks.length > 0 ? Buffer.concat(chunks).toString('utf-8') : null);
    });
    process.stdin.on('error', () => resolve(null));

    setTimeout(() => {
      process.stdin.removeAllListeners();
      process.stdin.pause();
      resolve(chunks.length > 0 ? Buffer.concat(chunks).toString('utf-8') : null);
    }, 5000);
  });
}
