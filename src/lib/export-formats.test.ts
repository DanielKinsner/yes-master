import { describe, expect, it } from "vitest";
import { EXPORT_FORMATS, exportEncoding } from "./export-formats";
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
});
