// Prepare exact source materials without copying untracked/private workspace files.
import { readFileSync, writeFileSync, mkdirSync, readdirSync, copyFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { homedir } from 'node:os';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';
import { verifyPackage } from './verify-audio-encoders.mjs';

const [revision, encoderDirectory, target, outputDirectory] = process.argv.slice(2);
assert(revision && encoderDirectory && target && outputDirectory,
  'Usage: node scripts/prepare-redistribution.mjs REVISION ENCODER_PACKAGE TARGET OUTPUT');
function git(args) {
  const result = spawnSync('git', args, { maxBuffer: 16 * 1024 * 1024, encoding: 'utf8', windowsHide: true });
  assert.equal(result.status, 0, result.stderr); return result.stdout.trim();
}
const commit = git(['rev-parse', '--verify', `${revision}^{commit}`]);
const output = resolve(outputDirectory); mkdirSync(output, { recursive: false });
const encoder = verifyPackage(resolve(encoderDirectory), target);
const sha = data => createHash('sha256').update(data).digest('hex');
const files = {};
function retain(source, name) {
  const destination = join(output, name); copyFileSync(source, destination);
  files[name] = sha(readFileSync(destination));
}
git(['archive', '--format=zip', '--output', join(output, 'yes-master-source.zip'), commit]);
files['yes-master-source.zip'] = sha(readFileSync(join(output, 'yes-master-source.zip')));
const lock = git(['show', `${commit}:src-tauri/Cargo.lock`]);
const cargoHome = process.env.CARGO_HOME ?? join(homedir(), '.cargo');
const caches = readdirSync(join(cargoHome, 'registry/cache')).map(name => join(cargoHome, 'registry/cache', name));
for (const name of ['mp3lame-encoder', 'mp3lame-sys']) {
  const block = lock.split('[[package]]').find(block => block.includes(`name = "${name}"`));
  assert(block, `Missing ${name} in candidate lock`);
  const version = /version = "([^"]+)"/.exec(block)[1];
  const checksum = /checksum = "([^"]+)"/.exec(block)[1];
  const filename = `${name}-${version}.crate`;
  const directory = caches.find(dir => readdirSync(dir).includes(filename));
  assert(directory, `Run cargo fetch --locked for candidate ${commit}; missing ${filename}`);
  const source = join(directory, filename);
  assert.equal(sha(readFileSync(source)), checksum, `Cargo source hash mismatch: ${filename}`);
  retain(source, filename);
}
for (const name of Object.keys(encoder.manifest.files).filter(name => name.startsWith('sources/') || name.startsWith('licenses/') || name.startsWith('build/'))) {
  retain(join(resolve(encoderDirectory), name), `encoder-${name.replaceAll('/', '-')}`);
}
writeFileSync(join(output, 'manifest.json'), JSON.stringify({
  commit, encoder: encoder.manifest, files,
  releaseReady: false,
  pending: ['Owner-approved LGPL relinking permission and actual rebuild verification', 'Final candidate installed/CI/signing evidence'],
}, null, 2) + '\n');
console.log(`Prepared source materials for ${commit}: ${output}; releaseReady=false`);
