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
  envPrefix: ["VITE_", "TAURI_"],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  build: {
    target: "es2021",
    outDir: "dist",
  },
});
