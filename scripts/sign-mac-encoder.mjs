import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

// Tauri 2.11 signs executable sidecars with hardened runtime enabled. Apply
// that same ad-hoc signature before Rust embeds the final file hash. A stable
// identifier also survives the target-suffixed file being renamed in the app.
export function signMacEncoder(path) {
  assert.equal(process.platform, 'darwin', 'Mac encoder signing requires macOS');
  execFileSync('codesign', [
    '--force', '--sign', '-', '--options', 'runtime',
    '--identifier', 'com.yesmaster.encoder', path,
  ]);
  execFileSync('codesign', ['--verify', '--strict', path]);
  return createHash('sha256').update(readFileSync(path)).digest('hex');
}
