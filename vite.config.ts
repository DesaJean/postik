import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';

const host = process.env.TAURI_DEV_HOST;

// Multi-page setup: controller.html (main window) + note.html (per-note windows).
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
        controller: resolve(__dirname, 'controller.html'),
        note: resolve(__dirname, 'note.html'),
      },
    },
  },
}));
