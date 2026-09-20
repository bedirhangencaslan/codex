import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  server: { port: 7877 },
  // The repo pins esbuild 0.28 (root package.json overrides), which drops
  // syntax lowering for the legacy default targets. Every host we ship to
  // (local browser, Tauri webview, VS Code webview) is evergreen, so build
  // untransformed.
  build: { target: "esnext" },
});
