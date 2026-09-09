import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

export const sources = {
  'ffmpeg-8.0.3.tar.xz': '6136812ea6d4e68bdba27e33c2a94382711cdf4f8602ffef056ff792bd6f9818',
  'libogg-1.3.5.tar.xz': 'c4d91be36fc8e54deae7575241e03f4211eb102afb3fc0775fbbc1b740016705',
  'libvorbis-1.3.7.tar.xz': 'b33cc4934322bcbf6efcbacf49e3ca01aadbea4114ec9589d1b1e9d20f72954b',
};
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
export function binaryArchitecture(bytes) {
  if (bytes.toString('ascii', 0, 2) === 'MZ') {
    const pe = bytes.readUInt32LE(60);
    assert.equal(bytes.toString('ascii', pe, pe + 4), 'PE\0\0', 'Invalid PE signature');
    return bytes.readUInt16LE(pe + 4) === 0x8664 ? 'x86_64-pc-windows-msvc' : 'unsupported-pe';
  }
  if (bytes.readUInt32LE(0) === 0xfeedfacf) {
    return bytes.readUInt32LE(4) === 0x100000c ? 'aarch64-apple-darwin' :
      bytes.readUInt32LE(4) === 0x1000007 ? 'x86_64-apple-darwin' : 'unsupported-macho';
  }
  throw new Error('Unrecognized encoder executable architecture');
}
export function validateDependencies(text, target) {
  const lines = text.trim().split(/\r?\n/).map(s => s.trim()).filter(Boolean);
  assert(lines.length, 'Missing runtime dependency inventory');
  if (target.endsWith('windows-msvc')) {
    for (const lib of lines) assert(/^(KERNEL32|msvcrt|ucrtbase|ADVAPI32|SHELL32|USER32|ole32|WS2_32|bcrypt)\.dll$/i.test(lib)
      || /^api-ms-win-crt-[\w-]+\.dll$/i.test(lib), `Unpackaged runtime library: ${lib}`);
  } else {
    for (const lib of lines.slice(1)) assert(/^\/(usr\/lib|System\/Library)\//.test(lib), `Unpackaged runtime library: ${lib}`);
  }
}
function run(exe, args, isolated = false) {
  const result = spawnSync(exe, args, {
    windowsHide: true, timeout: 120_000, maxBuffer: 32 * 1024 * 1024,
    env: isolated ? { ...process.env, PATH: process.platform === 'win32' ? `${process.env.SystemRoot}\\System32` : '/usr/bin:/bin' } : process.env,
  });
  assert.equal(result.error, undefined, `Cannot execute ${exe}: ${result.error?.message}`);
  assert.equal(result.status, 0, `${exe} failed: ${result.stderr?.toString()}`);
  return result.stdout;
}
function wav(frames, rate, channels, bits) {
  const bytes = Buffer.alloc(44 + frames * channels * bits / 8);
  bytes.write('RIFF'); bytes.writeUInt32LE(bytes.length - 8, 4); bytes.write('WAVEfmt ', 8);
  bytes.writeUInt32LE(16, 16); bytes.writeUInt16LE(1, 20); bytes.writeUInt16LE(channels, 22);
  bytes.writeUInt32LE(rate, 24); bytes.writeUInt32LE(rate * channels * bits / 8, 28);
  bytes.writeUInt16LE(channels * bits / 8, 32); bytes.writeUInt16LE(bits, 34);
  bytes.write('data', 36); bytes.writeUInt32LE(bytes.length - 44, 40);
  for (let f = 0; f < frames; f++) for (let ch = 0; ch < channels; ch++) {
    const value = 0.36 * Math.sin(2 * Math.PI * (431 + ch * 173) * f / rate)
      + 0.12 * Math.sin(2 * Math.PI * (1793 + ch * 293) * f / rate);
    bytes.writeIntLE(Math.round(value * 2 ** (bits - 1)), 44 + (f * channels + ch) * bits / 8, bits / 8);
  }
  return bytes;
}
function floats(bytes) {
  return Array.from({ length: bytes.length / 4 }, (_, i) => bytes.readFloatLE(i * 4));
}
function correlation(a, b, channels, start, end, delay) {
  let ab = 0, aa = 0, bb = 0;
  for (let f = start; f < end; f++) for (let c = 0; c < channels; c++) {
    const x = a[f * channels + c], y = b[(f + delay) * channels + c] ?? 0;
    ab += x * y; aa += x * x; bb += y * y;
  }
  return ab / Math.sqrt(aa * bb);
}
export function verifyPackage(dir, target, { seal = false } = {}) {
  const name = `yes-master-encoder-${target}${target.endsWith('windows-msvc') ? '.exe' : ''}`;
  const exe = join(dir, name);
  assert(existsSync(exe), `Missing packaged encoder: ${exe}`);
  const bytes = readFileSync(exe);
  assert.equal(binaryArchitecture(bytes), target, 'Wrong encoder architecture');
  const dependencies = readFileSync(join(dir, 'build/runtime-dependencies.txt'), 'utf8');
  validateDependencies(dependencies, target);
  const configuration = readFileSync(join(dir, 'build/configure-arguments.txt'), 'utf8');
  assert(!/--enable-(gpl|nonfree|version3|network)/.test(configuration), 'Unqualified license/network configuration');
  assert(configuration.includes('--disable-autodetect'), 'Autodetection must be disabled');
  const files = {};
  for (const [name, expected] of Object.entries(sources)) {
    assert.equal(sha(readFileSync(join(dir, 'sources', name))), expected, `Source hash mismatch: ${name}`);
    files[`sources/${name}`] = expected;
  }
  for (const name of ['COPYING.LGPLv2.1', 'LICENSE.md', 'libogg-1.3.5-COPYING', 'libvorbis-1.3.7-COPYING']) {
    files[`licenses/${name}`] = sha(readFileSync(join(dir, 'licenses', name)));
  }
  for (const name of ['configure-arguments.txt', 'runtime-dependencies.txt', 'version.txt', 'toolchain.txt', 'build-audio-encoders.sh']) {
    files[`build/${name}`] = sha(readFileSync(join(dir, 'build', name)));
  }
  files[name] = sha(bytes);
  const manifest = { schema: 1, target, files };
  const path = join(dir, 'manifest.json');
  if (seal) {
    assert(!existsSync(path), 'Refusing to replace an existing package manifest');
    writeFileSync(path, JSON.stringify(manifest, null, 2) + '\n', { flag: 'wx' });
  } else assert.deepEqual(manifest, JSON.parse(readFileSync(path, 'utf8')), 'Encoder package hash mismatch');
  const version = run(exe, ['-version'], true).toString();
  assert(version.includes('ffmpeg version 8.0.3'), 'Wrong encoder version');
  assert(!/--enable-(gpl|nonfree|version3|network)/.test(version), 'Binary has unqualified build flags');
  return { exe, manifest, version };
}
function qualify(dir, target, decoder, probe, seal) {
  const pkg = verifyPackage(dir, target, { seal });
  const work = join(dir, 'qualification'); mkdirSync(work, { recursive: true });
  const rows = [];
  const formats = [
    { ext: 'flac', codec: 'flac', container: 'flac', args: ['-c:a', 'flac', '-compression_level', '5'], lossless: true },
    { ext: 'aiff', codec: 'pcm_s24be', container: 'aiff', args: ['-c:a', 'pcm_s24be'], lossless: true },
    { ext: 'm4a', codec: 'aac', container: 'mov,mp4,m4a,3gp,3g2,mj2', args: ['-c:a', 'aac', '-profile:a', 'aac_low', '-b:a', '256k'], delay: 0, padding: 1023 },
    { ext: 'aac', codec: 'aac', container: 'aac', args: ['-c:a', 'aac', '-profile:a', 'aac_low', '-b:a', '256k'], delay: 1024, padding: 3071 },
    // Flush each packet's Ogg page: a single first-and-last audio page makes
    // FFmpeg's demuxer discard 128 valid tail frames on very short programmes.
    { ext: 'ogg', codec: 'vorbis', container: 'ogg', args: ['-c:a', 'libvorbis', '-q:a', '6', '-page_duration', '1'], delay: 0, padding: 0 },
  ];
  for (const rate of [44100, 48000]) for (const channels of [1, 2]) for (const frames of [257, rate * 2 + 137]) {
    const input = join(work, `signal-${rate}-${channels}-${frames}.wav`);
    writeFileSync(input, wav(frames, rate, channels, 24));
    const original = run(decoder, ['-v', 'error', '-i', input, '-f', 'f32le', '-c:a', 'pcm_f32le', '-']);
    for (const format of formats) {
      const output = `${input}.${format.ext}`;
      let encoderInput = input;
      if (format.ext === 'aac' && frames < 2048) {
        // ADTS has no sample-accurate duration metadata. At least three AAC
        // packets (including priming) are needed for reliable auto-probing.
        const padded = Buffer.alloc(44 + 2048 * channels * 3);
        readFileSync(input).copy(padded);
        padded.writeUInt32LE(padded.length - 8, 4); padded.writeUInt32LE(padded.length - 44, 40);
        encoderInput = `${input}.adts-input.wav`; writeFileSync(encoderInput, padded);
      }
      run(pkg.exe, ['-v', 'error', '-nostdin', '-y', '-i', encoderInput, '-map', '0:a:0', ...format.args, output], true);
      const info = JSON.parse(run(probe, ['-v', 'error', '-show_streams', '-show_format', '-of', 'json', output]).toString());
      assert.equal(info.streams.length, 1);
      assert.equal(info.streams[0].codec_name, format.codec);
      assert.equal(info.format.format_name, format.container);
      assert.equal(Number(info.streams[0].sample_rate), rate);
      assert.equal(info.streams[0].channels, channels);
      const decoded = run(decoder, ['-v', 'error', '-i', output, '-f', 'f32le', '-c:a', 'pcm_f32le', '-']);
      const samples = floats(decoded);
      assert(samples.every(Number.isFinite), `Nonfinite ${output}`);
      assert(samples.some(s => Math.abs(s) > 0.1), `Silent ${output}`);
      const deliveredFrames = samples.length / channels;
      let beginningCorrelation = null, tailCorrelation = null;
      if (format.lossless) assert.deepEqual(decoded, original, `Lossless PCM differs: ${output}`);
      else {
        assert(deliveredFrames >= frames + format.delay, `Lost frames: ${output} ${deliveredFrames}/${frames}`);
        assert(deliveredFrames <= frames + format.padding, `Excess padding: ${output} ${deliveredFrames}/${frames}`);
        const reference = floats(original);
        const window = Math.min(frames, 2048);
        beginningCorrelation = correlation(reference, samples, channels, 0, window, format.delay);
        tailCorrelation = correlation(reference, samples, channels, frames - window, frames, format.delay);
        assert(beginningCorrelation > 0.90 && tailCorrelation > 0.90,
          `Beginning/tail mismatch: ${output} ${beginningCorrelation}/${tailCorrelation}`);
      }
      rows.push({ format: format.ext, rate, channels, frames, deliveredFrames, beginningCorrelation, tailCorrelation });
    }
  }
  const evidence = { target, encoderSha256: pkg.manifest.files[Object.keys(pkg.manifest.files).at(-1)],
    independentDecoder: run(decoder, ['-version']).toString().split('\n')[0], rows };
  writeFileSync(join(work, 'results.json'), JSON.stringify(evidence, null, 2) + '\n');
  console.log(`PASS: ${rows.length} independently decoded encoder cases; ${join(work, 'results.json')}`);
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [dir, target, decoder, probe, flag] = process.argv.slice(2);
  assert(dir && target && decoder && probe, 'Usage: node scripts/verify-audio-encoders.mjs DIR TARGET INDEPENDENT_FFMPEG FFPROBE [--seal]');
  qualify(resolve(dir), target, resolve(decoder), resolve(probe), flag === '--seal');
}
