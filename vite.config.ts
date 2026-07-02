/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [react()],
  // Prevent vite from obscuring rust errors
  clearScreen: false,
  // Provide NODE_ENV for any dependency that reads it at runtime.
  define: {
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
