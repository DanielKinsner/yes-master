// src/hooks/useFirstRunGuide.test.tsx
//
// Behaviour coverage for the first-run guide hook, driving the real
// useFirstRunGuide → FirstRunOverlay pair through a tiny harness. This is the
// home of the silent-finish / audible-flip / send-off-timing / live-reset
// cases that used to be exercised through <StandardView> before the hint moved
// out of the rails into the App-root overlay (L9). The pure step machine and
// storage helpers stay covered by lib/first-run-guide.test.ts.

import { act } from "react";
import { createRoot } from "react-dom/client";
import { afterEach, describe, expect, it, vi } from "vitest";

import { useFirstRunGuide } from "./useFirstRunGuide";
import { FirstRunOverlay } from "../components/FirstRunOverlay";
import {
  FIRST_RUN_GUIDE_KEY,
  FIRST_RUN_GUIDE_RESET_EVENT,
} from "../lib/first-run-guide";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

afterEach(() => {
  document.body.innerHTML = "";
  globalThis.localStorage?.removeItem(FIRST_RUN_GUIDE_KEY);
});

function GuideHarness(props: {
  hasAnalyzedTrack: boolean;
  playbackKind: string;
  isPlaying: boolean;
}) {
  const guide = useFirstRunGuide(props);
  return <FirstRunOverlay step={guide.step} onDismiss={guide.dismiss} />;
}

const ON_ORIGINAL = {
  hasAnalyzedTrack: true,
  playbackKind: "source",
  isPlaying: false,
};
const ON_MASTER = {
  hasAnalyzedTrack: true,
  playbackKind: "master",
  isPlaying: false,
};

async function render(node: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  await act(async () => { root.render(node); });
  return { container, root };
}

describe("useFirstRunGuide", () => {
  it("shows the flip hint once a track is analyzed on Original", async () => {
    const { container, root } = await render(<GuideHarness {...ON_ORIGINAL} />);
    expect(container.querySelector(".hint-chip-flip")?.textContent).toContain(
      "Mastered",
    );
    await act(async () => root.unmount());
  });

  it("never shows when storage says the guide already finished", async () => {
    globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "done");
    const { container, root } = await render(<GuideHarness {...ON_ORIGINAL} />);
    expect(container.querySelector(".hint-chip")).toBeNull();
    await act(async () => root.unmount());
  });

  it("never shows for a fast user already on Mastered (and self-finishes)", async () => {
    const { container, root } = await render(<GuideHarness {...ON_MASTER} />);
    expect(container.querySelector(".hint-chip")).toBeNull();
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBe("done");
    await act(async () => root.unmount());
  });

  it("× dismisses the guide permanently", async () => {
    const { container, root } = await render(<GuideHarness {...ON_ORIGINAL} />);
    const x = container.querySelector<HTMLButtonElement>(".hint-chip-x")!;
    await act(async () => { x.click(); });
    expect(container.querySelector(".hint-chip")).toBeNull();
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBe(
      "dismissed",
    );
    await act(async () => root.unmount());
  });

  it("flip → send-off chip; after the window, the Advanced pointer", async () => {
    vi.useFakeTimers();
    try {
      const { container, root } = await render(<GuideHarness {...ON_ORIGINAL} />);
      expect(container.querySelector(".hint-chip-flip")).not.toBeNull();
      await act(async () => {
        root.render(
          <GuideHarness
            hasAnalyzedTrack
            playbackKind="master"
            isPlaying
          />,
        );
      });
      expect(container.querySelector(".hint-chip-sendoff")?.textContent).toContain(
        "Presets and Intensity",
      );
      expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBe("done");
      await act(async () => { vi.advanceTimersByTime(6000); });
      expect(container.querySelector(".hint-chip-sendoff")).toBeNull();
      expect(container.querySelector(".hint-chip-advanced")?.textContent).toContain(
        "Advanced",
      );
      await act(async () => root.unmount());
    } finally {
      vi.useRealTimers();
    }
  });

  it("a flip while paused does not complete the aha (nothing was heard)", async () => {
    const { container, root } = await render(<GuideHarness {...ON_ORIGINAL} />);
    expect(container.querySelector(".hint-chip-flip")).not.toBeNull();
    await act(async () => {
      root.render(<GuideHarness {...ON_MASTER} />);
    });
    expect(container.querySelector(".hint-chip-sendoff")).toBeNull();
    expect(globalThis.localStorage?.getItem(FIRST_RUN_GUIDE_KEY)).toBeNull();
    await act(async () => {
      root.render(<GuideHarness {...ON_ORIGINAL} />);
    });
    expect(container.querySelector(".hint-chip-flip")).not.toBeNull();
    await act(async () => root.unmount());
  });

  it("Settings reset revives the guide in the running session", async () => {
    globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "done");
    const { container, root } = await render(<GuideHarness {...ON_ORIGINAL} />);
    expect(container.querySelector(".hint-chip")).toBeNull();
    await act(async () => {
      globalThis.localStorage?.removeItem(FIRST_RUN_GUIDE_KEY);
      window.dispatchEvent(new Event(FIRST_RUN_GUIDE_RESET_EVENT));
    });
    expect(container.querySelector(".hint-chip-flip")).not.toBeNull();
    await act(async () => root.unmount());
  });

  it("live reset on Mastered playback advances to the send-off, never silently dies", async () => {
    globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "done");
    const { container, root } = await render(<GuideHarness {...ON_MASTER} />);
    expect(container.querySelector(".hint-chip")).toBeNull();
    await act(async () => {
      globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "reset");
      window.dispatchEvent(new Event(FIRST_RUN_GUIDE_RESET_EVENT));
    });
    expect(container.querySelector(".hint-chip-sendoff")).not.toBeNull();
    await act(async () => root.unmount());
  });

  it("mount with a pending reset marker on Mastered shows the send-off too", async () => {
    globalThis.localStorage?.setItem(FIRST_RUN_GUIDE_KEY, "reset");
    const { container, root } = await render(<GuideHarness {...ON_MASTER} />);
    expect(container.querySelector(".hint-chip-sendoff")).not.toBeNull();
    await act(async () => root.unmount());
  });
});
