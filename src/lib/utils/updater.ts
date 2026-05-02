import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

/**
 * Wraps the updater plugin so the UI doesn't have to think about plugin
 * specifics. Returns a small status object the Settings pane can render.
 */

export type UpdateStatus =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'up-to-date' }
  | { kind: 'available'; version: string; notes: string | null }
  | { kind: 'downloading'; downloaded: number; total: number | null }
  | { kind: 'ready' }
  | { kind: 'error'; message: string };

export async function checkForUpdate(): Promise<Update | null> {
  try {
    return await check();
  } catch (e) {
    const msg = String(e);
    // The updater plugin throws when the endpoint returns a non-2xx
    // (typical case: no published release yet, GitHub serves a 404
    // for the `latest/download/latest.json` URL). Treat that as
    // "nothing to install" rather than surfacing a scary error to
    // the user. Real failures (DNS, malformed JSON, signature
    // mismatch) still propagate.
    if (
      msg.includes('did not respond with a successful status code') ||
      msg.includes('status code: 404') ||
      msg.includes('Network Error')
    ) {
      console.warn('updater endpoint not reachable; treating as up-to-date:', msg);
      return null;
    }
    throw e;
  }
}

export async function downloadAndInstall(
  update: Update,
  onProgress: (downloaded: number, total: number | null) => void,
) {
  let downloaded = 0;
  let total: number | null = null;
  await update.downloadAndInstall((event) => {
    if (event.event === 'Started') {
      total = event.data.contentLength ?? null;
      onProgress(0, total);
    } else if (event.event === 'Progress') {
      downloaded += event.data.chunkLength;
      onProgress(downloaded, total);
    }
  });
}

export async function restart() {
  await relaunch();
}
