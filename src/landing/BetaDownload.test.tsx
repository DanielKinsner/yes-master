import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, describe, expect, it } from "vitest";

import BetaDownload from "./BetaDownload";
import { DURING_BETA, validRelease } from "./release-fixture";
import {
  RELEASES_INDEX_URL,
  REPO_URL,
  resolveRelease,
  type DetectedPlatform,
  type ReleaseMetadata,
} from "./release-config";

(globalThis as unknown as { IS_REACT_ACT_ENVIRONMENT: boolean })
  .IS_REACT_ACT_ENVIRONMENT = true;

let root: Root | null = null;
let container: HTMLDivElement | null = null;

afterEach(async () => {
  await act(async () => {
    root?.unmount();
  });
  root = null;
  container = null;
  document.body.innerHTML = "";
});

async function renderFor(
  metadata: ReleaseMetadata | null,
  platform: DetectedPlatform = "other",
  now: Date = DURING_BETA,
): Promise<HTMLDivElement> {
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
  await act(async () => {
    root?.render(
      <BetaDownload
        release={resolveRelease(metadata, now)}
        platform={platform}
      />,
    );
  });
  return container;
}

function downloadLinks(el: HTMLElement): HTMLAnchorElement[] {
  return Array.from(el.querySelectorAll<HTMLAnchorElement>("a[data-platform]"));
}

// ---------------------------------------------------------------------------
// The unavailable states. All four render the same thing to a visitor — which
// state we are in is internal — but each must reach it for its own reason.
// ---------------------------------------------------------------------------

describe("BetaDownload — no downloadable release", () => {
  const closedStates: Array<[string, ReleaseMetadata | null, string]> = [
    ["missing config", null, "unavailable"],
    [
      "malformed config",
      { version: "", publication: "published" } as unknown as ReleaseMetadata,
      "unavailable",
    ],
    ["draft release", validRelease({ publication: "draft" }), "draft-proof"],
    [
      "prerelease",
      validRelease({ publication: "prerelease" }),
      "candidate-published",
    ],
    [
      "published but unverified",
      validRelease({ verifiedAt: null }),
      "candidate-published",
    ],
    [
      "withdrawn release",
      validRelease({ publication: "withdrawn" }),
      "withdrawn",
    ],
  ];

  for (const [name, metadata, expectedState] of closedStates) {
    it(`renders no download action for a ${name}`, async () => {
      const el = await renderFor(metadata);

      expect(downloadLinks(el)).toHaveLength(0);

      // The single most important assertion in this file: nothing anywhere in
      // the component may point at /releases/latest, which 404s until a full
      // release exists. That dead CTA is the defect U5 exists to remove.
      const hrefs = Array.from(el.querySelectorAll("a")).map((a) =>
        a.getAttribute("href"),
      );
      for (const href of hrefs) {
        expect(href).not.toContain("/releases/latest");
      }

      // Internal state is carried for tests and owner debugging, never as copy.
      expect(el.firstElementChild?.getAttribute("data-release-state")).toBe(
        expectedState,
      );
      expect(el.textContent).not.toContain("draft");
      expect(el.textContent).not.toContain("unverified");
      expect(el.textContent).not.toContain("candidate");
    });
  }

  it("gives the inactive action a visible, programmatically linked reason", async () => {
    const el = await renderFor(null);

    const button = el.querySelector<HTMLButtonElement>("button");
    expect(button).not.toBeNull();
    expect(button?.disabled).toBe(true);

    // Programmatic: the reason is associated, not just nearby.
    const describedBy = button?.getAttribute("aria-describedby");
    expect(describedBy).toBeTruthy();
    const reason = el.querySelector(`#${describedBy}`);
    expect(reason).not.toBeNull();

    // Visible: real text in the document, not a title attribute or tooltip.
    expect(reason?.textContent?.trim().length ?? 0).toBeGreaterThan(20);
    expect(button?.getAttribute("title")).toBeNull();
    expect(reason?.getAttribute("role")).toBe("status");

    // And it must not look like a live CTA.
    expect(button?.className).not.toContain("cta-light");
    expect(button?.className).not.toContain("bg-gradient-to-b");
  });

  it("explains beta status in present tense without a date or version (S-A1)", async () => {
    const el = await renderFor(null);
    const text = el.textContent ?? "";

    expect(text).toContain("free public beta");
    expect(text).toContain("The download is not open");

    // docs/landing-brief.md hard rules: present tense, no roadmap, no version
    // numbers, no build detail.
    expect(text).not.toMatch(/coming soon/i);
    expect(text).not.toMatch(/\bwill be\b/i);
    expect(text).not.toMatch(/\b\d+\.\d+\.\d+\b/);
  });

  it("routes to guidance that resolves today (S-A1)", async () => {
    const el = await renderFor(null);
    const links = Array.from(el.querySelectorAll<HTMLAnchorElement>("a"));
    const hrefs = links.map((a) => a.getAttribute("href"));

    // The releases index resolves whether or not a release exists; /latest
    // does not.
    expect(hrefs).toContain(RELEASES_INDEX_URL);
    expect(el.textContent).toContain("Watch the releases page on GitHub");
    for (const link of links) {
      expect(link.getAttribute("href")).toMatch(/^https:\/\//);
      expect(link.getAttribute("rel")).toContain("noreferrer");
    }
  });

  it("distinguishes a withdrawn build from one that never opened", async () => {
    const withdrawn = await renderFor(
      validRelease({ publication: "withdrawn" }),
    );
    expect(withdrawn.textContent).toContain("no build available to download");

    document.body.innerHTML = "";
    const never = await renderFor(null);
    expect(never.textContent).toContain("The download is not open");
  });

  it("closes the download when the beta window has passed", async () => {
    const el = await renderFor(
      validRelease(),
      "other",
      new Date("2027-01-01T00:00:00Z"),
    );
    expect(downloadLinks(el)).toHaveLength(0);
    expect(el.textContent).toContain("public beta is closed");
  });
});

// ---------------------------------------------------------------------------
// The available state.
// ---------------------------------------------------------------------------

describe("BetaDownload — verified public release", () => {
  it("offers explicit Windows and Mac actions pointing at the artifacts (S-A2)", async () => {
    const el = await renderFor(validRelease());
    const links = downloadLinks(el);

    expect(links).toHaveLength(2);
    const [windows, mac] = links;

    expect(windows.textContent).toContain("Download for Windows");
    expect(windows.getAttribute("href")).toMatch(/\.exe$/);
    expect(mac.textContent).toContain("Download for Mac");
    expect(mac.getAttribute("href")).toMatch(/universal.*\.dmg$/);

    // Direct artifact URLs, not the release page and not /latest.
    for (const link of links) {
      const href = link.getAttribute("href") ?? "";
      expect(href.startsWith(`${REPO_URL}/releases/download/`)).toBe(true);
      expect(href).not.toContain("/releases/latest");
    }

    // Size is stated up front. A 100 MB download on a phone tether is a
    // decision, not a surprise.
    expect(el.textContent).toContain("96 MB");
    expect(el.textContent).toContain("118 MB");
  });

  it("keeps the release page as a secondary destination, not the download", async () => {
    const el = await renderFor(validRelease());
    const secondary = Array.from(
      el.querySelectorAll<HTMLAnchorElement>("a:not([data-platform])"),
    );
    const hrefs = secondary.map((a) => a.getAttribute("href"));

    expect(el.textContent).toContain(
      "All downloads, checksums, and release notes",
    );
    expect(hrefs).toContain(`${REPO_URL}/releases/tag/v0.9.1`);
  });

  it("uses external-link semantics on every outbound action", async () => {
    const el = await renderFor(validRelease());
    for (const link of Array.from(el.querySelectorAll<HTMLAnchorElement>("a"))) {
      expect(link.getAttribute("target")).toBe("_blank");
      expect(link.getAttribute("rel")).toContain("noreferrer");
    }
  });

  it("is operable by keyboard because the actions are real links", async () => {
    const el = await renderFor(validRelease());
    for (const link of downloadLinks(el)) {
      // An anchor with href is in the tab order and activates on Enter without
      // any JavaScript. A div with an onClick handler is not, and that is the
      // failure this pins.
      expect(link.tagName).toBe("A");
      expect(link.getAttribute("href")).toBeTruthy();
      expect(link.getAttribute("tabindex")).not.toBe("-1");
      expect(link.getAttribute("aria-disabled")).toBeNull();
      expect(link.hasAttribute("disabled")).toBe(false);
    }
  });

  it("emphasises the likely platform without hiding or choosing", async () => {
    const el = await renderFor(validRelease(), "mac");
    const links = downloadLinks(el);

    // Both actions still exist and are equally usable.
    expect(links).toHaveLength(2);
    const windows = links.find((l) => l.dataset.platform === "windows");
    const mac = links.find((l) => l.dataset.platform === "mac");
    expect(windows?.getAttribute("href")).toMatch(/\.exe$/);
    expect(mac?.getAttribute("href")).toMatch(/\.dmg$/);

    // Emphasis is visual plus an assistive-tech note, not a redirect.
    expect(mac?.className).toContain("ring-2");
    expect(windows?.className).not.toContain("ring-2");
    expect(mac?.textContent).toContain("matches the system you are browsing");
  });

  it("offers both platforms when detection is wrong or unknown", async () => {
    for (const platform of ["other", "windows"] as DetectedPlatform[]) {
      document.body.innerHTML = "";
      const el = await renderFor(validRelease(), platform);
      const links = downloadLinks(el);
      expect(links.map((l) => l.dataset.platform).sort()).toEqual([
        "mac",
        "windows",
      ]);
      // Nothing is pre-selected or auto-triggered.
      expect(el.querySelector("[autofocus]")).toBeNull();
      const emphasised = links.filter((l) => l.className.includes("ring-2"));
      expect(emphasised.length).toBeLessThanOrEqual(1);
    }
  });
});

// ---------------------------------------------------------------------------
// Claims that must hold in every state.
// ---------------------------------------------------------------------------

describe("BetaDownload — invariant copy", () => {
  it("states platform support honestly for mobile and Linux (S-I1)", async () => {
    for (const metadata of [null, validRelease()]) {
      document.body.innerHTML = "";
      const el = await renderFor(metadata);
      const text = el.textContent ?? "";

      expect(text).toContain("Windows + universal Mac");
      expect(text).toContain("no Linux build");
      expect(text).toContain("iPhone and Android are not currently available");

      // The mobile sentence is the landing brief's one roadmap exception and
      // is allowed to exist only if it stays date-free.
      expect(text).not.toMatch(/coming soon/i);
      expect(text).not.toMatch(/download for (iphone|android|ios)/i);
      expect(text).not.toMatch(/app store|google play/i);
    }
  });

  it("keeps the download ungated and links install help", async () => {
    for (const metadata of [null, validRelease()]) {
      document.body.innerHTML = "";
      const el = await renderFor(metadata);
      expect(el.textContent).toContain("No email required");
      expect(el.textContent).toContain("Install help");
      expect(el.textContent).toContain("paid publisher certificates");
    }
  });
});
