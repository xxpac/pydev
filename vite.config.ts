import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [svelte()],

  // Tauri expects a fixed port and fails if it is not available.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Don't watch the Rust side.
      ignored: ["**/src-tauri/**", "**/crates/**", "**/target/**"],
    },
  },

  // Produce assets compatible with the system WebViews Tauri targets.
  build: {
    target: ["es2021", "chrome100", "safari15"],
    minify: "esbuild",
    sourcemap: false,
  },
}));
