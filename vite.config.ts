import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';

const host = process.env.TAURI_DEV_HOST;

// Multi-page setup: index.html is the controller window (default entry — Vite
// auto-serves it on `/`); note.html is the per-note window template, opened
// via `WebviewUrl::App("note.html?id=…")` from Rust.
//
// Both HTML files live at the project root because Vite's `rollupOptions.input`
// resolves entries relative to the project root by default — moving them under
// src/pages/ would force a `root: 'src/pages'` change that complicates asset
// resolution. See docs/architecture.md.
export default defineConfig(async () => ({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  build: {
    rollupOptions: {
      input: {
        index: resolve(__dirname, 'index.html'),
        note: resolve(__dirname, 'note.html'),
      },
    },
  },
}));
