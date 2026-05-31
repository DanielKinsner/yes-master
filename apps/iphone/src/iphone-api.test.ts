import { describe, expect, it, vi } from "vitest";
import { createIphoneBackend } from "./iphone-api";
import type { MasteringSettings } from "../../../src/bindings";

describe("iPhone API facade", () => {
  it("calls the separate iPhone import command", async () => {
    const invoke = vi.fn().mockResolvedValue({ id: "track-1" });
    const backend = createIphoneBackend(invoke);

    await backend.importTrack("/private/song.wav");

    expect(invoke).toHaveBeenCalledWith("iphone_import_track", {
      path: "/private/song.wav",
    });
  });

  it("calls the separate iPhone analyze command", async () => {
    const invoke = vi.fn().mockResolvedValue({ track_id: "track-1" });
    const backend = createIphoneBackend(invoke);

    await backend.analyzeTrack("track-1", "/private/song.wav");

    expect(invoke).toHaveBeenCalledWith("iphone_analyze_track", {
      trackId: "track-1",
      path: "/private/song.wav",
    });
  });

  it("calls the separate iPhone render command with export settings", async () => {
    const invoke = vi.fn().mockResolvedValue({ output_paths: ["/private/master.wav"] });
    const backend = createIphoneBackend(invoke);
    const settings = { volume_match: false } as MasteringSettings;

    await backend.renderMaster({
      trackId: "track-1",
      trackPath: "/private/song.wav",
      settings,
      outputPath: "/private/master.wav",
    });

    expect(invoke).toHaveBeenCalledWith("iphone_render_master", {
      trackId: "track-1",
      trackPath: "/private/song.wav",
      settings,
      outputPath: "/private/master.wav",
    });
  });
});
