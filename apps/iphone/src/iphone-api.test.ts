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
    const preview = await backend.prepareMasterPreview({
      trackId: imported.id,
      trackPath: imported.path,
      settings: { volume_match: true } as MasteringSettings,
    });

    expect(imported.display_name).toBe("rough mix.wav");
    expect(imported.path).toBe("blob:yes-master/rough-mix");
    expect(analysis.track_id).toBe(imported.id);
    expect(preview.output_paths).toEqual(["blob:yes-master/rough-mix"]);
  });

  it("uses the suggested iPhone export filename", async () => {
    setNativeRuntime();
    dialogMocks.save.mockResolvedValue("/private/rough mix - YES Master.wav");

    await pickIphoneOutputPath("rough mix - YES Master.wav");

    expect(dialogMocks.save).toHaveBeenCalledWith(
      expect.objectContaining({
        defaultPath: "rough mix - YES Master.wav",
      }),
    );
  });

  it("keeps Chrome preview exports inside the browser flow", async () => {
    await expect(pickIphoneOutputPath("rough mix - YES Master.wav")).resolves.toBe(
      "rough mix - YES Master.wav",
    );
    expect(dialogMocks.save).not.toHaveBeenCalled();
  });
});
