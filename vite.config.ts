import react from "@vitejs/plugin-react-swc";
import path from "path";
import { defineConfig, searchForWorkspaceRoot } from "vite";

// What this build adds to the app (src/extensions/types.ts): none, unless
// VIBECHECK_EXTENSIONS names a module whose default export is AppExtensions.
// That module can import the app's own code through "@/" and "@bindings/".
const extensions = process.env.VIBECHECK_EXTENSIONS
  ? path.resolve(process.env.VIBECHECK_EXTENSIONS)
  : null;

export default defineConfig({
  plugins: [react()],
  resolve: {
    // One copy of React, even for libraries an extensions module brings
    // from its own folder
    dedupe: ["react", "react-dom"],
    alias: {
      "@extensions":
        extensions ??
        path.resolve(import.meta.dirname, "./src/extensions/none.ts"),
      "@bindings": path.resolve(import.meta.dirname, "./src-tauri/bindings"),
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    // The extensions module may live outside this folder
    fs: extensions
      ? {
          allow: [
            searchForWorkspaceRoot(process.cwd()),
            path.dirname(extensions),
          ],
        }
      : undefined,
  },
  // to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
