import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';

test('final bundle must retain the encoder bytes trusted by the Rust build', t => {
  const directory = mkdtempSync(join(tmpdir(), 'yes-master-bundle-check-'));
  t.after(() => {
    assert.equal(dirname(resolve(directory)), resolve(tmpdir()));
    rmSync(directory, { recursive: true, force: true });
  });
  const target = 'aarch64-apple-darwin';
  const staged = join(directory, 'src-tauri/binaries');
  mkdirSync(join(staged, 'licenses'), { recursive: true });
  const bytes = Buffer.from('qualified encoder fixture');
  writeFileSync(join(staged, `yes-master-encoder-${target}`), bytes);
  writeFileSync(join(staged, `manifest-${target}.json`), JSON.stringify({
    target, sha256: createHash('sha256').update(bytes).digest('hex'),
  }));
  for (const name of ['COPYING.LGPLv2.1', 'LICENSE.md', 'libogg-1.3.5-COPYING', 'libvorbis-1.3.7-COPYING']) {
    writeFileSync(join(staged, 'licenses', name), 'fixture notice');
  }
  const bundled = join(directory, 'YES Master.app/Contents/MacOS/yes-master-encoder');
  mkdirSync(dirname(bundled), { recursive: true });
  writeFileSync(bundled, bytes);
  const script = fileURLToPath(new URL('./check-staged-encoder.mjs', import.meta.url));
  const run = (...args) => spawnSync(process.execPath, [script, target, ...args], {
    cwd: directory, encoding: 'utf8', windowsHide: true,
  });
  assert.equal(run().status, 0, 'the staged control must pass');
  assert.equal(run(bundled).status, 0, 'an unchanged bundled copy must pass');
  writeFileSync(bundled, Buffer.from('mutated encoder fixture'));
  assert.notEqual(run(bundled).status, 0, 'post-bundle mutation must fail even when staging is intact');
  assert.equal(run().status, 0, 'the negative control must not alter staging');
  assert.notEqual(run(join(directory, 'missing-encoder')).status, 0, 'a missing bundled encoder must fail');
});
