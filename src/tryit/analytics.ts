import { isPublicWebsiteAnalyticsUrl } from "../lib/web-analytics";

type Props = Record<string, string | number | boolean | null>;

/** Vercel Web Analytics custom events for the demo. Public marketing site
 *  only (same gate as the page-view script); the desktop app and `/app`
 *  preview stay telemetry-free. Never throws, never blocks audio. */
export function trackTryIt(event: string, props?: Props) {
  try {
    if (typeof window === "undefined" || !isPublicWebsiteAnalyticsUrl(new URL(window.location.href))) return;
    void import("@vercel/analytics").then(({ track }) => track(`tryit_${event}`, props)).catch(() => {});
  } catch { /* analytics must never affect the demo */ }
}

/** Coarse buckets so events stay anonymous and comparable. */
export const minutesBucket = (seconds: number) =>
  seconds < 60 ? "<1m" : seconds < 180 ? "1-3m" : seconds < 360 ? "3-6m" : seconds < 600 ? "6-10m" : "10m+";
