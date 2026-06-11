import { afterEach, describe, expect, it, vi } from "vitest";
import type { ImportedTrack, PlaybackTick, RenderJob } from "../bindings";
import { mockInvoke, mockListen } from "./preview-mock";

afterEach(async () => {
  vi.useRealTimers();
  await mockInvoke("stop_playback");
});

describe("preview-mock playback identity", () => {
  it("emits playback ticks for the imported track that started playback", async () => {
    vi.useFakeTimers();
    const [track] = await mockInvoke<ImportedTrack[]>("import_tracks", {
      paths: ["C:/audio/fresh.wav"],
    });
    const ticks: PlaybackTick[] = [];
    const unlisten = await mockListen<PlaybackTick>("playback:tick", (event) => {
      ticks.push(event.payload);
    });

    await mockInvoke("play_track", {
      trackId: track.id,
      trackPath: track.path,
      startPositionSec: 12,
    });
    vi.advanceTimersByTime(50);
    unlisten();

    expect(ticks.at(-1)?.track_id).toBe(track.id);
    expect(ticks.at(-1)?.is_loaded).toBe(true);
    expect(ticks.at(-1)?.position_sec).toBeGreaterThan(12);
  });

  it("echoes render_track_master identity in browser preview render jobs", async () => {
    const [track] = await mockInvoke<ImportedTrack[]>("import_tracks", {
      paths: ["C:/audio/render-me.wav"],
    });

    const job = await mockInvoke<RenderJob>("render_track_master", {
      trackId: track.id,
      trackPath: track.path,
      settings: {},
      outputPath: "C:/renders/render-me.wav",
    });

    expect(job.kind).toBe("master");
    expect(job.target_tracks).toEqual([track.id]);
    expect(job.output_paths).toEqual(["C:/renders/render-me.wav"]);
  });
});
