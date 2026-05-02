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
  return check();
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
