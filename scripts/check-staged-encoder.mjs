import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import assert from 'node:assert/strict';
const target = process.argv[2] ?? execFileSync('rustc', ['-vV'], {encoding:'utf8'}).match(/^host: (.+)$/m)?.[1];
assert(target, 'Cannot identify the desktop build target');
const root = 'src-tauri/binaries';
const suffix = target.endsWith('windows-msvc') ? '.exe' : '';
try {
  const manifest = JSON.parse(readFileSync(`${root}/manifest-${target}.json`, 'utf8'));
  const bytes = readFileSync(`${root}/yes-master-encoder-${target}${suffix}`);
  assert.equal(manifest.target, target);
  assert.equal(createHash('sha256').update(bytes).digest('hex'), manifest.sha256, 'Staged encoder hash mismatch');
  for (const name of ['COPYING.LGPLv2.1','LICENSE.md','libogg-1.3.5-COPYING','libvorbis-1.3.7-COPYING']) readFileSync(`${root}/licenses/${name}`);
  console.log(`Verified staged encoder ${target}: ${manifest.sha256}`);
} catch (error) {
  throw new Error(`Build needs a qualified encoder. Run scripts/stage-audio-encoder.mjs PACKAGE ${target}. ${error.message}`);
}
