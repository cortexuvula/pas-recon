import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import pkg from "./package.json";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  // Only expose VITE_-prefixed env vars to the client bundle. Exposing TAURI_
  // (e.g. TAURI_SIGNING_PRIVATE_KEY) would risk leaking build-time secrets into
  // the shipped webview. Explicitly allowlist any var the frontend needs instead.
  envPrefix: ["VITE_"],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  build: {
    target: "es2021",
    outDir: "dist",
  },
});
