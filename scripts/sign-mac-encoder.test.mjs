import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, writeFileSync, copyFileSync, readFileSync, mkdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname, resolve } from 'node:path';
import { signMacEncoder } from './sign-mac-encoder.mjs';

test('Mac encoder hash survives Tauri sidecar rename and hardened-runtime signing', { skip: process.platform !== 'darwin' }, () => {
  const root = mkdtempSync(join(tmpdir(), 'yes-master-signing-'));
  const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
  // Matches the installed Tauri CLI 2.11.1 bundler's executable-signing call.
  const bundleSign = path => execFileSync('codesign', ['--force', '-s', '-', '--options', 'runtime', path]);
  try {
    const source = join(root, 'encoder.c');
    writeFileSync(source, 'int main(void) { return 0; }\n');
    for (const [label, architectures] of [
      ['arm', ['arm64']], ['intel', ['x86_64']], ['universal', ['arm64', 'x86_64']],
    ]) {
      const dir = join(root, label); mkdirSync(dir);
      const original = join(dir, 'qualified-encoder');
      execFileSync('clang', [...architectures.flatMap(arch => ['-arch', arch]), source, '-o', original]);
      execFileSync('codesign', ['--force', '--sign', '-', original]);
      const bundled = join(dir, 'yes-master-encoder');
      copyFileSync(original, bundled);
      const oldHash = hash(bundled);
      bundleSign(bundled);
      assert.notEqual(hash(bundled), oldHash, `${label}: negative control must expose the old staging mismatch`);

      const stagedHash = signMacEncoder(original);
      copyFileSync(original, bundled);
      bundleSign(bundled);
      assert.equal(hash(bundled), stagedHash, `${label}: packaged bytes must match Rust's embedded trust hash`);
      execFileSync('codesign', ['--verify', '--strict', bundled]);
      execFileSync('lipo', [bundled, '-verify_arch', ...architectures]);
    }
  } finally {
    assert.equal(dirname(resolve(root)), resolve(tmpdir()));
    rmSync(root, { recursive: true, force: true });
  }
});
