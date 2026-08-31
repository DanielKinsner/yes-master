import { afterEach, describe, expect, it, vi } from "vitest";
import type { ImportedTrack, PlaybackTick, RenderJob } from "../bindings";
import type { AlbumRenderReport } from "./api";
import { mockInvoke, mockListen, mockOpen } from "./preview-mock";

interface PreviewProjectShape {
  tracks: ImportedTrack[];
  track_order: string[];
  track_override_album: string[];
}

function setScenario(name: string | null): void {
  window.history.replaceState(null, "", name ? `/app?scenario=${name}` : "/app");
}

function albumRenderRequestFrom(project: PreviewProjectShape, overrides?: boolean[]) {
  return {
    plan: { title: "Preview Album", tracks: [] },
    tracks: project.track_order.map((id, index) => {
      const track = project.tracks.find((t) => t.id === id);
      if (!track) throw new Error(`seeded track missing for ${id}`);
      return {
        track_id: id,
        source_path: track.path,
        settings: {},
        override_album:
          overrides?.[index] ?? project.track_override_album.includes(id),
      };
    }),
  };
}

afterEach(async () => {
  vi.useRealTimers();
  setScenario(null);
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

// ---------------------------------------------------------------------------
// Audit T-02 — the album-warning scenario must be able to reach the REAL
// post-export advisory surface: a directory pick that resolves, seeded mixed
// source formats, and a render report derived from the seed + the actual
// request (never fabricated).
// ---------------------------------------------------------------------------

describe("album-warning scenario delivers real post-export advisories (audit T-02)", () => {
  it("resolves the album directory picker so Export Album can proceed", async () => {
    setScenario("album-warning");
    await expect(mockOpen({ directory: true })).resolves.toBe("/preview/exports");
  });

  it("keeps the base scenarios' directory pick conservative (cancelled)", async () => {
    setScenario("album-4");
    await expect(mockOpen({ directory: true })).resolves.toBeNull();
  });

  it("seeds mixed source formats: 44.1k mono, 48k stereo, 48k 4ch, 48k stereo; override on 3", async () => {
    setScenario("album-warning");
    const project = await mockInvoke<PreviewProjectShape>("load_recent_session");
    expect(project.tracks.map((t) => t.sample_rate)).toEqual([
      44_100, 48_000, 48_000, 48_000,
    ]);
    expect(project.tracks.map((t) => t.channels)).toEqual([1, 2, 4, 2]);
    expect(project.track_override_album).toEqual([project.track_order[2]]);
  });

  it("renders 48k stereo with four records and track 3 override", async () => {
    setScenario("album-warning");
    const project = await mockInvoke<PreviewProjectShape>("load_recent_session");
    const report = await mockInvoke<AlbumRenderReport>("render_album_plan", {
      request: albumRenderRequestFrom(project),
      outputDir: "/preview/exports",
    });
    expect(report.status).toEqual({ status: "done" });
    expect(report.rendered_sample_rate).toBe(48_000);
    expect(report.rendered_channels).toBe(2);
    expect(report.tracks).toHaveLength(4);
    expect(report.source_sample_rates).toEqual([44_100, 48_000, 48_000, 48_000]);
    expect(report.source_channels).toEqual([1, 2, 4, 2]);
    expect(report.tracks.map((t) => t.override_album)).toEqual([
      false,
      false,
      true,
      false,
    ]);
    expect(report.tracks.map((t) => t.position)).toEqual([1, 2, 3, 4]);
  });

  it("derives override from the REQUEST, not the seed — broken wiring cannot be masked", async () => {
    setScenario("album-warning");
    const project = await mockInvoke<PreviewProjectShape>("load_recent_session");
    // The seed marks position 3; this request marks position 2 instead. The
    // report must follow the request, or the UI-to-render wiring could break
    // silently while the scenario stayed green off the seed.
    const report = await mockInvoke<AlbumRenderReport>("render_album_plan", {
      request: albumRenderRequestFrom(project, [false, true, false, false]),
      outputDir: "/preview/exports",
    });
    expect(report.tracks.map((t) => t.override_album)).toEqual([
      false,
      true,
      false,
      false,
    ]);
  });
});
