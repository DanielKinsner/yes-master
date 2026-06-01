import react from "@vitejs/plugin-react";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const root = dirname(fileURLToPath(import.meta.url));
const iphoneDevHost = process.env.TAURI_DEV_HOST ?? "0.0.0.0";
const iphoneDevPort = 5174;

export default defineConfig({
  root,
  plugins: [react()],
  server: {
    headers: {
      "Cache-Control": "no-store, max-age=0",
    },
    hmr: process.env.TAURI_DEV_HOST
      ? {
          host: process.env.TAURI_DEV_HOST,
          port: iphoneDevPort,
          protocol: "ws",
        }
      : undefined,
    host: iphoneDevHost,
    port: iphoneDevPort,
    strictPort: true,
  },
  build: {
    outDir: resolve(root, "dist"),
    emptyOutDir: true,
  },
});
