// Independent validation of retained engine outputs, not the packaged decoder.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
const [directory, ffmpeg, ffprobe] = process.argv.slice(2);
assert(directory && ffmpeg && ffprobe, 'Usage: node scripts/verify-export-delivery.mjs OUTPUT INDEPENDENT_FFMPEG FFPROBE');
const dir = resolve(directory);
const reports = JSON.parse(readFileSync(join(dir, 'reports.json'), 'utf8'));
function run(exe, args) {
  const result = spawnSync(resolve(exe), args, {windowsHide:true, maxBuffer:32*1024*1024, timeout:120_000});
  assert.equal(result.status,0,result.stderr?.toString()); return result;
}
const rows = [];
let wav;
for (const job of reports.jobs) {
  const path = job.output_paths[0], facts = job.delivered_format;
  const info = JSON.parse(run(ffprobe,['-v','error','-show_streams','-show_format','-of','json',path]).stdout);
  assert.equal(info.streams.length,1);
  const stream = info.streams[0];
  assert.equal(stream.codec_name,facts.codec === 'aac_lc' ? 'aac' : facts.codec);
  if (facts.codec === 'aac_lc') assert.equal(stream.profile,'LC');
  assert.equal(Number(stream.sample_rate),facts.sample_rate);
  assert.equal(stream.channels,facts.channels);
  const raw = run(ffmpeg,['-v','error','-i',path,'-c:a','pcm_f32le','-f','f32le','-']).stdout;
  let nonSilent = false;
  for (let i=0;i<raw.length;i+=4) {const value=raw.readFloatLE(i);assert(Number.isFinite(value));nonSilent ||= Math.abs(value)>0.01;}
  assert(nonSilent);
  if (facts.encoding.format === 'wav') wav = raw;
  if (['flac','aiff'].includes(facts.encoding.format)) assert.deepEqual(raw,wav,'Lossless master PCM differs from WAV');
  const result = run(ffmpeg,['-hide_banner','-nostats','-i',path,'-af','ebur128=peak=true','-f','null','-']);
  const summary = result.stderr.toString().split('Summary:').at(-1);
  const lufs = Number(/I:\s+(-?[\d.]+) LUFS/.exec(summary)?.[1]);
  const peak = Number(/Peak:\s+(-?[\d.]+) dBFS/.exec(summary)?.[1]);
  const lra = Number(/LRA:\s+([\d.]+) LU/.exec(summary)?.[1]);
  assert(Math.abs(lufs-job.measurements.lufs_integrated)<=0.15,`LUFS ${path}`);
  assert(Math.abs(peak-job.measurements.true_peak_dbtp)<=0.2,`True peak ${path}`);
  assert(Math.abs(lra-job.measurements.dynamic_range_lu)<=0.5,`LRA ${path}`);
  rows.push({format:facts.encoding.format,path,sha256:createHash('sha256').update(readFileSync(path)).digest('hex'),
    frames:raw.length/4/facts.channels,independentLufs:lufs,independentTruePeak:peak,independentLra:lra});
}
writeFileSync(join(dir,'independent-results.json'),JSON.stringify({buildStamp:reports.build_stamp,rows},null,2)+'\n');
console.log(`PASS: ${rows.length} actual engine files independently decoded and measured`);
