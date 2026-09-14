// Decode at the file's OWN sample rate, the way the desktop app renders
// ("preserve source rate" unless a delivery profile says otherwise).
//
// `decodeAudioData` always resamples to the rate of the context it runs on,
// and a default `AudioContext` runs at whatever the user's device is set to.
// So the container header is read first to learn the file's rate, and the
// decode runs on an OfflineAudioContext at that rate. Formats whose header
// can't be read here fall back to 48 kHz.

export const DEFAULT_SAMPLE_RATE = 48_000;
const PLAUSIBLE = (rate: number) => (rate >= 8_000 && rate <= 384_000 ? rate : null);

const ascii = (view: DataView, at: number, length: number) => {
  let s = "";
  for (let i = 0; i < length && at + i < view.byteLength; i++) s += String.fromCharCode(view.getUint8(at + i));
  return s;
};

function wav(view: DataView): number | null {
  const head = ascii(view, 0, 4);
  if ((head !== "RIFF" && head !== "RF64") || ascii(view, 8, 4) !== "WAVE") return null;
  let at = 12;
  while (at + 8 <= view.byteLength) {
    const id = ascii(view, at, 4);
    const size = view.getUint32(at + 4, true);
    if (id === "fmt " && at + 12 <= view.byteLength) return PLAUSIBLE(view.getUint32(at + 12, true));
    at += 8 + size + (size & 1);
  }
  return null;
}

function aiff(view: DataView): number | null {
  if (ascii(view, 0, 4) !== "FORM") return null;
  const form = ascii(view, 8, 4);
  if (form !== "AIFF" && form !== "AIFC") return null;
  let at = 12;
  while (at + 8 <= view.byteLength) {
    const id = ascii(view, at, 4);
    const size = view.getUint32(at + 4, false);
    if (id === "COMM" && at + 26 <= view.byteLength) {
      // 80-bit IEEE 754 extended: 1 sign + 15 exponent bits, 64-bit mantissa with explicit integer bit.
      const p = at + 16;
      const exponent = ((view.getUint8(p) & 0x7f) << 8) | view.getUint8(p + 1);
      const mantissa = view.getUint32(p + 2, false) * 2 ** 32 + view.getUint32(p + 6, false);
      return PLAUSIBLE(Math.round(mantissa * 2 ** (exponent - 16383 - 63)));
    }
    at += 8 + size + (size & 1);
  }
  return null;
}

function flac(view: DataView): number | null {
  if (ascii(view, 0, 4) !== "fLaC" || view.byteLength < 21) return null;
  if ((view.getUint8(4) & 0x7f) !== 0) return null; // first block must be STREAMINFO
  return PLAUSIBLE((view.getUint8(18) << 12) | (view.getUint8(19) << 4) | (view.getUint8(20) >> 4));
}

function mp3(view: DataView): number | null {
  let at = 0;
  if (ascii(view, 0, 3) === "ID3" && view.byteLength >= 10) {
    const size = ((view.getUint8(6) & 0x7f) << 21) | ((view.getUint8(7) & 0x7f) << 14) | ((view.getUint8(8) & 0x7f) << 7) | (view.getUint8(9) & 0x7f);
    at = 10 + size + ((view.getUint8(5) & 0x10) ? 10 : 0);
  }
  const RATES = { 3: [44_100, 48_000, 32_000], 2: [22_050, 24_000, 16_000], 0: [11_025, 12_000, 8_000] } as Record<number, number[]>;
  const end = Math.min(view.byteLength - 3, at + 65_536);
  for (let i = at; i < end; i++) {
    if (view.getUint8(i) !== 0xff || (view.getUint8(i + 1) & 0xe0) !== 0xe0) continue;
    const b1 = view.getUint8(i + 1), b2 = view.getUint8(i + 2);
    const version = (b1 >> 3) & 3, layer = (b1 >> 1) & 3, index = (b2 >> 2) & 3;
    if (version === 1 || layer === 0 || index === 3 || !RATES[version]) continue;
    return RATES[version][index];
  }
  return null;
}

function ogg(view: DataView): number | null {
  if (ascii(view, 0, 4) !== "OggS") return null;
  const end = Math.min(view.byteLength - 16, 8_192);
  for (let i = 0; i < end; i++) {
    if (view.getUint8(i) === 0x01 && ascii(view, i + 1, 6) === "vorbis") return PLAUSIBLE(view.getUint32(i + 12, true));
    if (ascii(view, i, 8) === "OpusHead") return 48_000; // Opus always decodes at 48 kHz
  }
  return null;
}

function mp4(view: DataView): number | null {
  if (ascii(view, 4, 4) !== "ftyp") return null;
  // Find the AudioSampleEntry ("mp4a"); its sample rate is a 16.16 fixed-point
  // at byte 32 of the box. A moov placed after a large mdat isn't scanned.
  const end = Math.min(view.byteLength - 36, 4 * 1024 * 1024);
  for (let i = 4; i < end; i++) {
    if (ascii(view, i, 4) !== "mp4a") continue;
    const rate = PLAUSIBLE(view.getUint16(i - 4 + 32, false));
    if (rate) return rate;
  }
  return null;
}

/** The container's declared sample rate, or null when it can't be read. */
export function sniffSampleRate(buffer: ArrayBuffer): number | null {
  const view = new DataView(buffer);
  if (view.byteLength < 12) return null;
  for (const probe of [wav, aiff, flac, ogg, mp4, mp3]) {
    try { const rate = probe(view); if (rate) return rate; } catch { /* keep probing */ }
  }
  return null;
}

export type Decoded = { audio: AudioBuffer; fileRate: number | null; decodedRate: number };

/** Decode `buffer` at its own rate (or 48 kHz when unknown/unsupported). */
export async function decodeAtSourceRate(buffer: ArrayBuffer): Promise<Decoded> {
  const fileRate = sniffSampleRate(buffer);
  let context: OfflineAudioContext;
  try { context = new OfflineAudioContext(1, 1, fileRate ?? DEFAULT_SAMPLE_RATE); }
  catch { context = new OfflineAudioContext(1, 1, DEFAULT_SAMPLE_RATE); }
  const audio = await context.decodeAudioData(buffer);
  return { audio, fileRate, decodedRate: audio.sampleRate };
}

/** A playback context at the track's rate when the browser allows it. */
export function playbackContext(sampleRate: number): AudioContext {
  try { return new AudioContext({ sampleRate }); }
  catch { return new AudioContext(); }
}
