import { describe, expect, it } from "vitest";
import { sniffSampleRate } from "./decode";

const bytes = (...parts: (string | number[])[]) => {
  const out: number[] = [];
  for (const part of parts) {
    if (typeof part === "string") for (const ch of part) out.push(ch.charCodeAt(0));
    else out.push(...part);
  }
  return new Uint8Array(out).buffer;
};
const le32 = (n: number) => [n & 255, (n >> 8) & 255, (n >> 16) & 255, (n >>> 24) & 255];
const be32 = (n: number) => [(n >>> 24) & 255, (n >> 16) & 255, (n >> 8) & 255, n & 255];

describe("sniffSampleRate", () => {
  it("reads WAV (RIFF and RF64) fmt chunks, skipping earlier chunks", () => {
    const junk = ["JUNK", le32(4), [0, 0, 0, 0]];
    const fmt = ["fmt ", le32(16), [1, 0, 2, 0], le32(44_100), le32(176_400), [4, 0, 16, 0]];
    expect(sniffSampleRate(bytes("RIFF", le32(100), "WAVE", ...junk, ...fmt))).toBe(44_100);
    expect(sniffSampleRate(bytes("RF64", le32(100), "WAVE", "fmt ", le32(16), [1, 0, 2, 0], le32(96_000), le32(0), [0, 0, 0, 0]))).toBe(96_000);
  });
  it("reads AIFF's 80-bit extended sample rate", () => {
    const comm = ["COMM", be32(18), [0, 2], be32(1000), [0, 16], [0x40, 0x0e, 0xbb, 0x80, 0, 0, 0, 0, 0, 0]];
    expect(sniffSampleRate(bytes("FORM", be32(100), "AIFF", ...comm))).toBe(48_000);
  });
  it("reads FLAC STREAMINFO", () => {
    const streaminfo = new Array(34).fill(0);
    streaminfo[10] = 0x17; streaminfo[11] = 0x70; streaminfo[12] = 0x00; // 96000 << 4 at bytes 18..20 of the file
    expect(sniffSampleRate(bytes("fLaC", [0x00, 0, 0, 34], streaminfo))).toBe(96_000);
  });
  it("reads the first MP3 frame header after an ID3v2 tag", () => {
    const id3 = ["ID3", [3, 0, 0], [0, 0, 0, 5], [0, 0, 0, 0, 0]];
    expect(sniffSampleRate(bytes(...id3, [0xff, 0xfb, 0x90, 0x00]))).toBe(44_100);
    expect(sniffSampleRate(bytes(...id3, [0xff, 0xf3, 0x50, 0x00]))).toBe(22_050);
    expect(sniffSampleRate(bytes(...id3, [0xff, 0xfb, 0x94, 0x00]))).toBe(48_000);
  });
  it("reads Ogg Vorbis and treats Opus as 48 kHz", () => {
    const page = ["OggS", new Array(24).fill(0)];
    expect(sniffSampleRate(bytes(...page, [0x01], "vorbis", le32(0), [2], le32(44_100), new Array(16).fill(0)))).toBe(44_100);
    expect(sniffSampleRate(bytes(...page, "OpusHead", new Array(16).fill(0)))).toBe(48_000);
  });
  it("reads the mp4a sample entry in M4A", () => {
    const mp4a = [be32(36), "mp4a", new Array(24).fill(0), [0xac, 0x44], [0, 0], new Array(8).fill(0)];
    expect(sniffSampleRate(bytes(be32(24), "ftyp", "M4A ", new Array(12).fill(0), ...mp4a))).toBe(44_100);
  });
  it("returns null for unknown or tiny inputs", () => {
    expect(sniffSampleRate(bytes("hello world, not audio at all"))).toBeNull();
    expect(sniffSampleRate(bytes("RIFF"))).toBeNull();
    expect(sniffSampleRate(bytes("RIFF", le32(100), "WAVE", "fmt ", le32(16), [1, 0, 2, 0], le32(7_000), le32(0), [0, 0, 0, 0]))).toBeNull();
  });
});
