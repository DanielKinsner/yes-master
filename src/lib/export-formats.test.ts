import { describe, expect, it } from "vitest";
import { EXPORT_FORMATS, exportEncoding, ensureExportExtension, exportApiArgs, type ExportFormat } from "./export-formats";
describe("export encoding contracts", () => {
  it("names codec/container choices explicitly and retains quality defaults", () => {
    expect(Object.keys(EXPORT_FORMATS)).toEqual(["wav", "mp3", "flac", "m4a", "aac", "ogg", "aiff"]);
    expect(exportEncoding("mp3")).toEqual({ format: "mp3", bitrate_kbps: 320 });
    expect(exportEncoding("m4a")).toEqual({ format: "m4a", bitrate_kbps: 256 });
    expect(exportEncoding("ogg")).toEqual({ format: "ogg", quality: 6 });
    expect(exportEncoding("flac")).toEqual({ format: "flac" });
  });
  it("rejects unsupported quality before export", () => {
    expect(() => exportEncoding("aac", 96)).toThrow();
    expect(() => exportEncoding("ogg", 256, 5)).toThrow();
  });
  it("normalizes stale known suffixes and preserves meaningful dots in names", () => {
    for (const format of Object.keys(EXPORT_FORMATS) as ExportFormat[]) {
      expect(ensureExportExtension("Song.final.WAV", format)).toBe(`Song.final.${format}`);
      expect(ensureExportExtension("Song.final", format)).toBe(`Song.final.${format}`);
    }
    expect(ensureExportExtension("Song.aif", "aiff")).toBe("Song.aiff");
    expect(exportApiArgs(exportEncoding("wav"))).toEqual([]);
    expect(exportApiArgs(exportEncoding("mp3"))).toEqual([320]);
    expect(exportApiArgs(exportEncoding("m4a"))).toEqual([undefined, { format: "m4a", bitrate_kbps: 256 }]);
  });
});
