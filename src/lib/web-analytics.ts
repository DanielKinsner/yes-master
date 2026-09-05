const PUBLIC_ANALYTICS_HOSTS = new Set(["yesdsp.com", "www.yesdsp.com"]);

/**
 * Vercel Web Analytics belongs to the public marketing site only. The desktop
 * app and the browser-based /app preview intentionally remain telemetry-free.
 */
export function isPublicWebsiteAnalyticsUrl(url: URL): boolean {
  const isAppPath = url.pathname === "/app" || url.pathname.startsWith("/app/");

  return (
    url.protocol === "https:" &&
    PUBLIC_ANALYTICS_HOSTS.has(url.hostname.toLowerCase()) &&
    !isAppPath &&
    !url.searchParams.has("app")
  );
}
