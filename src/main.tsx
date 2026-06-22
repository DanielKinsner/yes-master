import React from "react";
import ReactDOM from "react-dom/client";
import { isTauri } from "./lib/tauri-runtime";

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
  const { default: LandingPage } = await import("./LandingPage");
  root.render(
    <React.StrictMode>
      <LandingPage />
    </React.StrictMode>,
  );
}

void boot();
