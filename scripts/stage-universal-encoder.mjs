// Run on macOS after both thin packages have passed their execution matrices.
import { readFileSync, writeFileSync, copyFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';
import { verifyPackage } from './verify-audio-encoders.mjs';
const [armDir, intelDir, destination = 'src-tauri/binaries'] = process.argv.slice(2);
assert(process.platform === 'darwin' && armDir && intelDir, 'Usage on Mac: node scripts/stage-universal-encoder.mjs ARM_PACKAGE INTEL_PACKAGE [DESTINATION]');
const targets = ['aarch64-apple-darwin', 'x86_64-apple-darwin'];
const packages = [armDir,intelDir].map((dir,i) => verifyPackage(resolve(dir), targets[i]));
const dir = resolve(destination); mkdirSync(dir, {recursive:true});
const output = join(dir,'yes-master-encoder-universal-apple-darwin');
execFileSync('lipo', ['-create', ...packages.map(pkg=>pkg.exe), '-output', output]);
execFileSync('lipo', ['-verify_arch', 'arm64', 'x86_64', output]);
execFileSync('codesign', ['--force', '--sign', '-', output]);
for (const architecture of ['arm64','x86_64']) execFileSync('arch', [`-${architecture}`, output, '-version']);
const sha256 = createHash('sha256').update(readFileSync(output)).digest('hex');
// Both Rust slices must expect the same final universal executable bytes.
for (const target of [...targets, 'universal-apple-darwin']) {
  writeFileSync(join(dir,`manifest-${target}.json`),JSON.stringify({target,sha256},null,2)+'\n');
  if (target !== 'universal-apple-darwin') copyFileSync(output,join(dir,`yes-master-encoder-${target}`));
}
mkdirSync(join(dir,'licenses'),{recursive:true});
for (const file of Object.keys(packages[0].manifest.files).filter(file=>file.startsWith('licenses/'))) {
  assert.equal(packages[0].manifest.files[file],packages[1].manifest.files[file]);
  copyFileSync(join(resolve(armDir),file),join(dir,file));
}
console.log(`Staged universal encoder: ${sha256}. Final app signing and installed hash verification remain mandatory.`);
