/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [react()],
  // Prevent vite from obscuring rust errors
  clearScreen: false,
  // react-draggable (under react-rnd) references `process.env.*` at runtime, which
  // is undefined in the browser and throws `process is not defined`. Provide the
  // values it reads without clobbering anything else.
  define: {
    "process.env.DRAGGABLE_DEBUG": "false",
    "process.env.NODE_ENV": JSON.stringify(mode),
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: [],
  },
}));
