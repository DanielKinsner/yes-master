import { describe, expect, it } from "vitest";
import {
  AUDIO_DIALOG_FILTER,
  AUDIO_EXTENSIONS,
  SUPPORTED_FORMATS_COPY,
  supportedAudioExtensionFromName,
} from "./supported-formats";

describe("supported audio formats", () => {
  it("derives dialog filters and display copy from one extension list", () => {
    expect(AUDIO_DIALOG_FILTER.extensions).toEqual([...AUDIO_EXTENSIONS]);
    expect(SUPPORTED_FORMATS_COPY).toBe(
      AUDIO_EXTENSIONS.map((ext) => ext.toUpperCase()).join(" · "),
    );
  });

  it("keeps unsupported desktop formats out of the public contract", () => {
    expect(AUDIO_EXTENSIONS).not.toContain("opus");
    expect(AUDIO_EXTENSIONS).toContain("aiff");
    expect(AUDIO_EXTENSIONS).toContain("aif");
    expect(SUPPORTED_FORMATS_COPY).not.toMatch(/Opus/i);
  });

  it("normalizes supported file names without accepting unknown extensions", () => {
    expect(supportedAudioExtensionFromName("Track.WAV")).toBe("wav");
    expect(supportedAudioExtensionFromName("mix.final.m4a")).toBe("m4a");
    expect(supportedAudioExtensionFromName("spoken-word.opus")).toBeNull();
    expect(supportedAudioExtensionFromName("master.aiff")).toBe("aiff");
    expect(supportedAudioExtensionFromName("master.AIF")).toBe("aif");
    expect(supportedAudioExtensionFromName("no-extension")).toBeNull();
  });
});
