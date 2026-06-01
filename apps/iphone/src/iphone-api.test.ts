import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createBrowserPreviewIphoneBackend,
  createIphoneBackend,
  pickIphoneAudioPath,
  pickIphoneOutputPath,
} from "./iphone-api";
import type { MasteringSettings } from "../../../src/bindings";

const dialogMocks = vi.hoisted(() => ({
  open: vi.fn(),
  save: vi.fn(),
}));
const originalCreateObjectUrl = Object.getOwnPropertyDescriptor(
  URL,
  "createObjectURL",
);

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: dialogMocks.open,
  save: dialogMocks.save,
}));

afterEach(() => {
  dialogMocks.open.mockReset();
  dialogMocks.save.mockReset();
  delete (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  delete (globalThis as typeof globalThis & { isTauri?: boolean }).isTauri;
  if (originalCreateObjectUrl) {
    Object.defineProperty(URL, "createObjectURL", originalCreateObjectUrl);
  } else {
    delete (URL as unknown as { createObjectURL?: unknown }).createObjectURL;
  }
  document.body.innerHTML = "";
});

function setNativeRuntime() {
  Object.defineProperty(window, "__TAURI_INTERNALS__", {
    configurable: true,
    value: {},
  });
}

function setTauriRuntimeMarker() {
  Object.defineProperty(globalThis, "isTauri", {
    configurable: true,
    value: true,
  });
}

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

  it("reactivates the iPhone audio session", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const backend = createIphoneBackend(invoke);

    await backend.reactivateAudioSession();

    expect(invoke).toHaveBeenCalledWith("iphone_reactivate_audio_session");
  });

  it("calls the separate iPhone waveform command", async () => {
    const invoke = vi.fn().mockResolvedValue({ track_id: "track-1", channels: [[]] });
    const backend = createIphoneBackend(invoke);

    await backend.prepareWaveform("track-1", "/private/song.wav", 140);

    expect(invoke).toHaveBeenCalledWith("iphone_prepare_waveform", {
      trackId: "track-1",
      trackPath: "/private/song.wav",
      targetPixels: 140,
    });
  });

  it("calls the native iPhone Original playback command", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const backend = createIphoneBackend(invoke);

    await backend.playOriginal("track-1", "/private/song.wav", 12.5);

    expect(invoke).toHaveBeenCalledWith("iphone_play_track", {
      trackId: "track-1",
      trackPath: "/private/song.wav",
      startPositionSec: 12.5,
    });
  });

  it("calls the native iPhone Mastered playback command", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const backend = createIphoneBackend(invoke);
    const settings = { volume_match: true } as MasteringSettings;

    await backend.playMastered("track-1", "/private/song.wav", settings, 7, false);

    expect(invoke).toHaveBeenCalledWith("iphone_play_master", {
      trackId: "track-1",
      trackPath: "/private/song.wav",
      settings,
      startPositionSec: 7,
      previewLufsLanding: false,
    });
  });

  it("calls the native iPhone transport commands", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    const backend = createIphoneBackend(invoke);

    await backend.updateMasteringChain({ volume_match: false } as MasteringSettings);
    await backend.pausePlayback();
    await backend.resumePlayback();
    await backend.seekPlayback(4);
    await backend.stopPlayback();

    expect(invoke).toHaveBeenCalledWith("iphone_update_chain", {
      settings: { volume_match: false },
      previewLufsLanding: false,
    });
    expect(invoke).toHaveBeenCalledWith("iphone_pause_playback");
    expect(invoke).toHaveBeenCalledWith("iphone_resume_playback");
    expect(invoke).toHaveBeenCalledWith("iphone_seek_playback", {
      positionSec: 4,
    });
    expect(invoke).toHaveBeenCalledWith("iphone_stop_playback");
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
    setNativeRuntime();
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

  it("uses the native picker when the Tauri runtime marker is present", async () => {
    setTauriRuntimeMarker();
    dialogMocks.open.mockResolvedValue("/private/song.wav");

    const selectedPath = pickIphoneAudioPath();
    await Promise.resolve();

    expect(dialogMocks.open).toHaveBeenCalledWith(
      expect.objectContaining({
        pickerMode: "document",
        fileAccessMode: "copy",
      }),
    );

    await expect(selectedPath).resolves.toBe("/private/song.wav");
  });

  it("opens a browser file picker when Chrome previews the dev server", async () => {
    const createObjectUrl = vi.fn().mockReturnValue("blob:yes-master/rough-mix");
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: createObjectUrl,
    });

    const selectedPath = pickIphoneAudioPath();
    const input = document.querySelector<HTMLInputElement>("input[type='file']");
    const file = new File(["audio"], "rough mix.wav", { type: "audio/wav" });

    expect(input?.accept).toContain(".wav");
    Object.defineProperty(input, "files", {
      configurable: true,
      value: [file],
    });
    input?.dispatchEvent(new Event("change"));

    await expect(selectedPath).resolves.toBe("blob:yes-master/rough-mix");
    expect(createObjectUrl).toHaveBeenCalledWith(file);
    expect(document.querySelector("input[type='file']")).toBeNull();
  });

  it("imports Chrome preview files without native Tauri APIs", async () => {
    const createObjectUrl = vi.fn().mockReturnValue("blob:yes-master/rough-mix");
    Object.defineProperty(URL, "createObjectURL", {
      configurable: true,
      value: createObjectUrl,
    });
    const selectedPath = pickIphoneAudioPath();
    const input = document.querySelector<HTMLInputElement>("input[type='file']");
    const file = new File(["audio"], "rough mix.wav", { type: "audio/wav" });
    Object.defineProperty(input, "files", {
      configurable: true,
      value: [file],
    });
    input?.dispatchEvent(new Event("change"));

    const backend = createBrowserPreviewIphoneBackend();
    const path = await selectedPath;
    expect(path).toBe("blob:yes-master/rough-mix");
    if (!path) throw new Error("Expected browser import path");

    const imported = await backend.importTrack(path);
    const analysis = await backend.analyzeTrack(imported.id, imported.path);
    const waveform = await backend.prepareWaveform(imported.id, imported.path, 64);
    const preview = await backend.prepareMasterPreview({
      trackId: imported.id,
      trackPath: imported.path,
      settings: { volume_match: true } as MasteringSettings,
    });

    expect(imported.display_name).toBe("rough mix.wav");
    expect(imported.path).toBe("blob:yes-master/rough-mix");
    expect(analysis.track_id).toBe(imported.id);
    expect(waveform.track_id).toBe(imported.id);
    expect(waveform.channels[0]?.length).toBeGreaterThan(0);
    expect(preview.output_paths).toEqual(["blob:yes-master/rough-mix"]);
  });

  it("returns the suggested filename for the iPhone export folder", async () => {
    setNativeRuntime();

    await expect(
      pickIphoneOutputPath("rough mix - YES Master.wav"),
    ).resolves.toBe("rough mix - YES Master.wav");

    // No save() dialog on iOS — iphone_render_master lands the file in the
    // app's Documents/YES Master folder, so the dialog must not be invoked.
    expect(dialogMocks.save).not.toHaveBeenCalled();
  });

  it("keeps Chrome preview exports inside the browser flow", async () => {
    await expect(pickIphoneOutputPath("rough mix - YES Master.wav")).resolves.toBe(
      "rough mix - YES Master.wav",
    );
    expect(dialogMocks.save).not.toHaveBeenCalled();
  });
});
