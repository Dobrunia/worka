import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    // Strip the render-blocking Google Fonts @import from dobruniaui-vue/styles.css.
    // Without this, WebView2 holds the entire stylesheet until fonts.googleapis.com
    // responds — in restricted networks the page stays white forever.
    // CSS variables and utility classes still load; font falls back to Segoe UI.
    {
      name: "strip-google-fonts",
      transform(code: string, id: string) {
        if (id.includes("dobruniaui-vue") && id.includes("styles.css")) {
          return {
            code: code.replace(
              /@import\s*["']https?:\/\/fonts\.googleapis\.com[^"']*["']\s*;/g,
              ""
            ),
          };
        }
      },
    },
  ],

  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
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
      ignored: ["**/src-tauri/**"],
    },
  },

  // Pre-bundle Tauri API so Vite doesn't discover it at runtime and trigger a reload.
  optimizeDeps: {
    include: ["@tauri-apps/api/window"],
  },

  // Test options
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: ["./src/tests/setup.ts"],
    include: ["**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"],
    coverage: {
      reporter: ["text", "json", "html"],
    },
  },
}));
