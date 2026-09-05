import React from "react";
import ReactDOM from "react-dom/client";
import { isTauri } from "./lib/tauri-runtime";
import { isPublicWebsiteAnalyticsUrl } from "./lib/web-analytics";

function wantsAppShell(): boolean {
  if (isTauri()) return true;
  const url = new URL(window.location.href);
  return url.pathname === "/app" || url.searchParams.has("app");
}

async function boot() {
  const root = ReactDOM.createRoot(document.getElementById("root")!);
  if (wantsAppShell()) {
    await import("./App.css");
    const { default: App } = await import("./App");
    root.render(
      <React.StrictMode>
        <App />
      </React.StrictMode>,
    );
    return;
  }

  await import("./LandingPage.css");
  const analyticsModule = isPublicWebsiteAnalyticsUrl(
    new URL(window.location.href),
  )
    ? await import("@vercel/analytics/react")
    : null;
  const { default: LandingPage } = await import("./LandingPage");
  const Analytics = analyticsModule?.Analytics;
  root.render(
    <React.StrictMode>
      <LandingPage />
      {Analytics ? (
        <Analytics
          beforeSend={(event) => {
            try {
              return isPublicWebsiteAnalyticsUrl(new URL(event.url))
                ? event
                : null;
            } catch {
              return null;
            }
          }}
        />
      ) : null}
    </React.StrictMode>,
  );
}

void boot();
