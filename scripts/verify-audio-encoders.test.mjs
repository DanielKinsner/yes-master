import test from 'node:test';
import assert from 'node:assert/strict';
import { binaryArchitecture, validateDependencies } from './verify-audio-encoders.mjs';

test('package gate diagnoses wrong architecture and corrupt executable headers', () => {
  const pe = Buffer.alloc(128);
  pe.write('MZ'); pe.writeUInt32LE(64, 60); pe.write('PE\0\0', 64); pe.writeUInt16LE(0x8664, 68);
  assert.equal(binaryArchitecture(pe), 'x86_64-pc-windows-msvc');
  pe.writeUInt16LE(0xaa64, 68);
  assert.equal(binaryArchitecture(pe), 'unsupported-pe');
  pe.write('bad!', 64);
  assert.throws(() => binaryArchitecture(pe), /Invalid PE signature/);
});
test('package gate rejects unbundled Windows and Mac runtime libraries', () => {
  validateDependencies('KERNEL32.dll\napi-ms-win-crt-runtime-l1-1-0.dll', 'x86_64-pc-windows-msvc');
  assert.throws(() => validateDependencies('libgcc_s_seh-1.dll', 'x86_64-pc-windows-msvc'), /Unpackaged/);
  validateDependencies('encoder:\n/usr/lib/libSystem.B.dylib (compatibility version 1)', 'aarch64-apple-darwin');
  assert.throws(() => validateDependencies('encoder:\n/opt/homebrew/lib/libvorbis.dylib', 'aarch64-apple-darwin'), /Unpackaged/);
});
