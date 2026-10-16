#!/usr/bin/env node
const { spawnSync } = require('node:child_process');
const path = require('node:path');

const isMac = process.platform === 'darwin';
const root = path.join(__dirname, '..');

if (isMac) {
  const result = spawnSync('./scripts/build.sh', {
    cwd: root,
    stdio: 'inherit',
    env: process.env,
    shell: true,
  });
  process.exit(result.status ?? 0);
} else {
  const result = spawnSync('npx', ['tauri', 'build'], {
    cwd: root,
    stdio: 'inherit',
    env: process.env,
  });
  process.exit(result.status ?? 0);
}
