import { describe, expect, it, vi } from "vitest";
import {
  createIphoneBackend,
  pickIphoneAudioPath,
  pickIphoneOutputPath,
} from "./iphone-api";
import type { MasteringSettings } from "../../../src/bindings";

const dialogMocks = vi.hoisted(() => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: dialogMocks.open,
  save: dialogMocks.save,
}));

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

  it("calls the separate iPhone mastered preview command", async () => {
    const invoke = vi.fn().mockResolvedValue({
      output_paths: ["/private/preview/track-1-mastered.wav"],
    });
    const backend = createIphoneBackend(invoke);
    const settings = { volume_match: true } as MasteringSettings;

    await backend.prepareMasterPreview({
      trackId: "track-1",
      trackPath: "/private/song.wav",
      settings,
    });

    expect(invoke).toHaveBeenCalledWith("iphone_prepare_master_preview", {
      trackId: "track-1",
      trackPath: "/private/song.wav",
      settings,
    });
  });

  it("opens iPhone audio as a copied document file", async () => {
    dialogMocks.open.mockResolvedValue("/private/song.wav");

    await pickIphoneAudioPath();

    expect(dialogMocks.open).toHaveBeenCalledWith(
      expect.objectContaining({
        directory: false,
        multiple: false,
        pickerMode: "document",
        fileAccessMode: "copy",
      }),
    );
  });

  it("uses the suggested iPhone export filename", async () => {
    dialogMocks.save.mockResolvedValue("/private/rough mix - YES Master.wav");

    await pickIphoneOutputPath("rough mix - YES Master.wav");

    expect(dialogMocks.save).toHaveBeenCalledWith(
      expect.objectContaining({
        defaultPath: "rough mix - YES Master.wav",
      }),
    );
  });
});
