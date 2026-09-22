import { execFileSync } from 'node:child_process';
import { readFileSync, copyFileSync, mkdtempSync, rmSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, join, resolve } from 'node:path';
import assert from 'node:assert/strict';

// Tauri 2.11 signs executable sidecars with hardened runtime enabled. Apply
// that same ad-hoc signature before Rust embeds the final file hash. codesign
// derives its identifier from the filename, so sign using the packaged name.
export function signMacEncoder(path) {
  assert.equal(process.platform, 'darwin', 'Mac encoder signing requires macOS');
  const parent = dirname(resolve(path));
  const temporary = mkdtempSync(join(parent, '.sign-encoder-'));
  try {
    const packagedName = join(temporary, 'yes-master-encoder');
    copyFileSync(path, packagedName);
    execFileSync('codesign', ['--force', '-s', '-', '--options', 'runtime', packagedName]);
    execFileSync('codesign', ['--verify', '--strict', packagedName]);
    const sha256 = createHash('sha256').update(readFileSync(packagedName)).digest('hex');
    copyFileSync(packagedName, path);
    return sha256;
  } finally {
    assert.equal(dirname(resolve(temporary)), parent);
    rmSync(temporary, { recursive: true, force: true });
  }
}
