import { describe, expect, it } from "vitest";

import { isPublicWebsiteAnalyticsUrl } from "./web-analytics";

describe("public website analytics boundary", () => {
  it.each([
    "https://yesdsp.com/",
    "https://www.yesdsp.com/",
    "https://www.yesdsp.com/#advanced",
  ])("permits aggregate page analytics on the public site: %s", (value) => {
    expect(isPublicWebsiteAnalyticsUrl(new URL(value))).toBe(true);
  });

  it.each([
    "http://www.yesdsp.com/",
    "https://www.yesdsp.com/app",
    "https://www.yesdsp.com/app/",
    "https://www.yesdsp.com/app/session",
    "https://www.yesdsp.com/?app",
    "https://yes-master.vercel.app/",
    "http://localhost:5173/",
  ])("keeps analytics out of app, preview, and local surfaces: %s", (value) => {
    expect(isPublicWebsiteAnalyticsUrl(new URL(value))).toBe(false);
  });
});
