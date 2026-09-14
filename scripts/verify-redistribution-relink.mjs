// Prove recipients can rebuild the archived application with a modified LAME.
// This never installs the modified application or grants license permissions.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync, mkdirSync, readdirSync, openSync, closeSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const [materialsArg, packageArg, target, workArg] = process.argv.slice(2);
assert(materialsArg && packageArg && target && workArg,
  'Usage: node scripts/verify-redistribution-relink.mjs MATERIALS ENCODER_PACKAGE TARGET NEW_WORK_DIRECTORY');
const materials = resolve(materialsArg), pkg = resolve(packageArg), work = resolve(workArg);
assert(!existsSync(work), 'Relink workspace must be new');
const manifestPath = join(materials, 'manifest.json');
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
const hash = file => createHash('sha256').update(readFileSync(file)).digest('hex');
for (const [name, expected] of Object.entries(manifest.files)) {
  assert.equal(hash(join(materials, name)), expected, `Source material changed: ${name}`);
}
assert.equal(manifest.encoder.target, target, 'Encoder/source proof target mismatch');
mkdirSync(work);
const source = join(work, 'source'), vendor = join(work, 'vendor'), proof = join(materials, 'relink-proof');
mkdirSync(source); mkdirSync(vendor); mkdirSync(proof);
writeFileSync(join(proof, 'verify-redistribution-relink.mjs'), readFileSync(fileURLToPath(import.meta.url)));
function run(exe, args, cwd = source, logName = 'setup.log') {
  const fd = openSync(join(proof, logName), 'a');
  try {
    const result = spawnSync(exe, args, {
      cwd, windowsHide: true, stdio: ['ignore', fd, fd],
      env: { ...process.env, CARGO_TARGET_DIR: join(work, 'target'), CARGO_BUILD_JOBS: '4', GIT_CEILING_DIRECTORIES: work },
      timeout: 30 * 60_000,
    });
    assert.equal(result.error, undefined, result.error?.message);
    assert.equal(result.status, 0, `${exe} failed; see ${join(proof, logName)}`);
  } finally { closeSync(fd); }
}
run('tar', ['-xf', join(materials, 'yes-master-source.zip'), '-C', source]);
const crates = {};
for (const name of ['mp3lame-sys', 'mp3lame-encoder']) {
  const archive = readdirSync(materials).find(file => file.startsWith(`${name}-`) && file.endsWith('.crate'));
  assert(archive, `Missing ${name} source`);
  run('tar', ['-xf', join(materials, archive), '-C', vendor]);
  crates[name] = archive.slice(0, -6);
}
const header = join(vendor, crates['mp3lame-sys'], 'lame-3.100/libmp3lame/version.h');
const originalHeader = readFileSync(header, 'utf8');
assert.match(originalHeader, /#\s*define LAME_MINOR_VERSION\s+100\b/);
writeFileSync(header, originalHeader.replace(/(#\s*define LAME_MINOR_VERSION\s+)100\b/, (_, prefix) => `${prefix}101`));
mkdirSync(join(source, '.cargo'), { recursive: true });
writeFileSync(join(source, '.cargo/config.toml'), '[patch.crates-io]\n' +
  Object.entries(crates).map(([name, dir]) => `${name} = { path = "../vendor/${dir}" }`).join('\n') + '\n');
// npm's JavaScript entry point avoids cmd.exe/shell quoting on Windows.
const npmRoot = process.env.npm_execpath;
const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm';
if (process.platform === 'win32') {
  // Locate npm-cli beside the actual Node runtime, or use the explicit CLI path.
  const nodeDir = resolve(process.execPath, '..');
  const npmCli = npmRoot ?? join(nodeDir, 'node_modules/npm/bin/npm-cli.js');
  assert(existsSync(npmCli), 'Run through npm exec or provide npm_execpath for this Node distribution');
  run(process.execPath, [npmCli, 'ci'], source, 'frontend.log');
  run(process.execPath, [npmCli, 'run', 'build'], source, 'frontend.log');
} else {
  run(npmCommand, ['ci'], source, 'frontend.log');
  run(npmCommand, ['run', 'build'], source, 'frontend.log');
}
run(process.execPath, [join(source, 'scripts/stage-audio-encoder.mjs'), pkg, target]);
writeFileSync(join(source, 'src-tauri/tests/relink_proof.rs'), `#![cfg(feature = "app-runner")]
#[test]
fn modified_lame_is_linked_and_encodes() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../modified-lame-proof.mp3");
    let samples = (0..48_000 * 2).flat_map(|frame| {
        let v = (frame as f32 * 997.0 * std::f32::consts::TAU / 48_000.0).sin() * 0.2;
        [Ok(v), Ok(v * 0.7)]
    });
    let output = yes_master_lib::mp3::write(&path, samples, 48_000, 2, 320, None).unwrap();
    let bytes = std::fs::read(&output).unwrap();
    assert!(bytes.windows(b"LAME3.101".len()).any(|w| w == b"LAME3.101"));
    let decoded = yes_master_lib::decode::decode_full(&output).unwrap();
    assert_eq!((decoded.sample_rate, decoded.channels), (48_000, 2));
    assert!(decoded.samples.iter().all(|x| x.is_finite()));
    assert!(decoded.samples.iter().any(|x| x.abs() > 0.1));
}
`);
const cargoArgs = ['--manifest-path', 'src-tauri/Cargo.toml', '--target', target, '--release'];
run('cargo', ['build', ...cargoArgs], source, 'application-build.log');
run('cargo', ['test', ...cargoArgs, '--test', 'mp3_export', '--test', 'relink_proof'], source, 'encode-tests.log');
const app = join(work, 'target', target, 'release', `yes-master${target.includes('windows') ? '.exe' : ''}`);
assert(readFileSync(app).includes(Buffer.from('LAME3.101')), 'Full application lacks the modified library marker');
const record = {
  schema: 1, sourceCommit: manifest.commit, target,
  fullApplicationRebuild: 'pass', modifiedLibraryEncode: 'pass', existingMp3Tests: 3,
  applicationSha256: hash(app), modifiedHeaderSha256: hash(header),
  sourceArchiveSha256: hash(join(materials, 'yes-master-source.zip')),
  encodedMp3Sha256: hash(join(source, 'modified-lame-proof.mp3')),
  note: 'Modified library is a verification marker only; no modified application was installed.',
};
writeFileSync(join(proof, 'result.json'), JSON.stringify(record, null, 2) + '\n');
for (const name of readdirSync(proof)) manifest.files[`relink-proof/${name}`] = hash(join(proof, name));
manifest.relinkProof = record;
// The existing permission decision remains explicit. A successful rebuild
// supplies mechanical proof only and must never approve distribution by itself.
manifest.releaseReady = false;
manifest.pending = ['Owner-approved LGPL relinking permission', 'Final candidate release evidence'];
writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + '\n');
console.log(`PASS: archived ${manifest.commit} rebuilt for ${target} with modified LAME; releaseReady=false`);
