import { copyFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';
import { verifyPackage } from './verify-audio-encoders.mjs';
import { signMacEncoder } from './sign-mac-encoder.mjs';

const [source, target, destination = 'src-tauri/binaries'] = process.argv.slice(2);
assert(source && target, 'Usage: node scripts/stage-audio-encoder.mjs PACKAGE TARGET [DESTINATION]');
const { exe, manifest } = verifyPackage(resolve(source), target);
const dir = resolve(destination); mkdirSync(dir, { recursive: true });
const name = `yes-master-encoder-${target}${target.endsWith('windows-msvc') ? '.exe' : ''}`;
copyFileSync(exe, join(dir, name));
const sha256 = target.endsWith('apple-darwin')
  ? signMacEncoder(join(dir, name))
  : manifest.files[name];
writeFileSync(join(dir, `manifest-${target}.json`), JSON.stringify({ target, sha256 }, null, 2) + '\n');
const notices = join(dir, 'licenses'); mkdirSync(notices, { recursive: true });
for (const path of Object.keys(manifest.files).filter(path => path.startsWith('licenses/'))) {
  copyFileSync(join(resolve(source), path), join(dir, path));
}
console.log(`Staged verified ${target}: ${sha256}`);
